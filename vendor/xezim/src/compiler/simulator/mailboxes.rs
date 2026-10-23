//! Simulator mailboxes.

use super::*;
#[allow(unused_imports)]
use crate::{log_eprintln as eprintln, log_println as println};

impl Simulator {
    /// Hand a put/try_put value to a parked mailbox get/peek waiter. The assign
    /// to the waiter's destination MUST run in the WAITER's process context
    /// (its local_stack holds the get/peek's output formal), not the putter's,
    /// so the continuation's `ScopePop` output writebacks propagate the value
    /// back to the original caller. Assigning in the putter's context writes
    /// the wrong frame, so the waiter reads a stale/null handle (item data=0).
    /// Swap to the waiter's saved context, assign, re-save, restore the putter's
    /// context, then schedule the continuation.
    /// Wake processes parked in `semaphore.get()` now that the count rose,
    /// FIFO and in order (§15.3.3): a waiter is served only when the FULL
    /// count it needs is available, and it is NOT skipped in favour of a
    /// later, smaller request — that would starve a large getter.
    pub(super) fn wake_semaphore_waiters(&mut self, handle: usize) {
        loop {
            let count = self.semaphores.get(&handle).copied().unwrap_or(0);
            let next_n = self
                .semaphore_get_waiters
                .get(&handle)
                .and_then(|q| q.front())
                .map(|w| w.n);
            match next_n {
                Some(n) if count >= n => {
                    let w = self
                        .semaphore_get_waiters
                        .get_mut(&handle)
                        .unwrap()
                        .pop_front()
                        .unwrap();
                    *self.semaphores.get_mut(&handle).unwrap() = count - n;
                    if w.cont.is_empty() {
                        self.child_finished(w.pid);
                    } else {
                        self.event_queue.schedule(self.time, w.pid, w.cont);
                    }
                }
                _ => break,
            }
        }
    }

    /// §15.4.1 — record a bounded mailbox's capacity from its `new(N)` arg.
    /// N absent or 0 leaves it unbounded (no entry).
    /// Allocate a built-in `mailbox` / `semaphore` object (§15.3, §15.4) on
    /// the class heap and register its queue or key count; `args` are the
    /// constructor arguments (bound / initial key count).
    pub(super) fn alloc_builtin_container(&mut self, kind: &str, args: &[Expression]) -> usize {
        let handle = self.heap.len();
        self.heap.push(Some(ClassInstance {
            class_name: kind.to_string(),
            properties: PropMap::default(),
            type_bindings: HashMap::default(),
            spec: None,
            creation_scope: self.active_instance_scope(),
        }));
        if kind == "semaphore" {
            let n = args
                .first()
                .map(|a| self.eval_expr(a).to_u64().unwrap_or(0) as i64)
                .unwrap_or(0);
            self.semaphores.insert(handle, n);
        } else {
            self.mailboxes
                .insert(handle, std::collections::VecDeque::new());
            self.record_mailbox_bound(handle, args);
        }
        handle
    }

    pub(super) fn record_mailbox_bound(&mut self, handle: usize, args: &[Expression]) {
        let bound = args
            .first()
            .map(|a| self.eval_expr(a).to_u64().unwrap_or(0) as usize)
            .unwrap_or(0);
        if bound > 0 {
            self.mailbox_bound.insert(handle, bound);
        }
    }

    /// §15.4.1 — a slot just freed on a bounded mailbox: admit one parked `put`
    /// (store its value, resume the producer). No-op for an unbounded mailbox or
    /// when no producer is waiting.
    pub(super) fn admit_mailbox_put_waiter(&mut self, handle: usize) {
        let bound = self.mailbox_bound.get(&handle).copied().unwrap_or(0);
        if bound == 0 {
            return;
        }
        let len = self.mailboxes.get(&handle).map(|q| q.len()).unwrap_or(0);
        if len >= bound {
            return;
        }
        if let Some(w) = self
            .mailbox_put_waiters
            .get_mut(&handle)
            .and_then(|q| q.pop_front())
        {
            self.mailboxes.get_mut(&handle).unwrap().push_back(w.value);
            if w.cont.is_empty() {
                self.child_finished(w.pid);
            } else {
                self.event_queue.schedule(self.time, w.pid, w.cont);
            }
        }
    }

    /// §15.4: after any put, deliver queued items to parked get/peek
    /// waiters until one side runs dry. A peek delivery leaves the item in
    /// the box (so a subsequent parked GET can consume the same item — the
    /// sequencer's get_next_item peek followed by item_done's try_get); a
    /// get delivery consumes it.
    pub(super) fn drain_mailbox_get_waiters(&mut self, handle: usize) {
        loop {
            let front = match self.mailboxes.get(&handle).and_then(|q| q.front().cloned()) {
                Some(v) => v,
                None => break,
            };
            let waiter = self
                .mailbox_get_waiters
                .get_mut(&handle)
                .and_then(|q| q.pop_front());
            let Some(w) = waiter else { break };
            let MailboxGetWaiter {
                pid,
                lvalue,
                cont,
                is_peek,
            } = w;
            if !is_peek {
                self.mailboxes.get_mut(&handle).unwrap().pop_front();
                // A consumed slot may admit a parked bounded-put producer.
                self.admit_mailbox_put_waiter(handle);
            }
            if env_set_cached!("XEZIM_MBX_DBG") {
                eprintln!(
                    "[MBX] t={} DRAIN deliver to pid={} is_peek={} handle={}",
                    self.time, pid, is_peek, handle
                );
            }
            self.deliver_to_mailbox_waiter(pid, &lvalue, front, cont, !is_peek);
        }
    }

    /// The member-wise struct an aggregate mailbox message rides in, when
    /// this argument names one. `None` for a packed struct or any scalar —
    /// those round-trip through the queued `Value` as before.
    pub(super) fn mbx_struct_arg(
        &mut self,
        arg: &Expression,
    ) -> Option<(String, crate::ast::types::StructUnionType)> {
        let name = Self::plain_ident_name(arg)?;
        let (resolved, su) = self.struct_copy_target(name.clone());
        if let Some(s) = su {
            return Some((resolved, s));
        }
        // A formal typed by a class TYPE PARAMETER — `mailbox #(T)` with
        // `put(T t)` / `get(output T t)` inside `class box #(type T)`. Its
        // declared type is the parameter NAME, which `struct_copy_target`
        // cannot resolve, so the message fell back to the scalar path and the
        // members never travelled. Resolve the binding under the active
        // specialization first.
        let dt = self.module.var_decl_types.get(&name).cloned()?;
        let DataType::TypeReference { name: tn, .. } = &dt else {
            return None;
        };
        let concrete = self.resolve_type_param_binding(&tn.name.name)?;
        let base = concrete.split('#').next().unwrap_or(&concrete).to_string();
        let tdt = self.module.typedef_types.get(&base).cloned()?;
        self.unpacked_struct_of(&tdt).map(|su| (name, su))
    }

    /// Copy an unpacked-struct message into a shadow variable; the returned
    /// value is the id that names it, to be queued in the message's place.
    pub(super) fn mbx_stash_struct(
        &mut self,
        src: &str,
        su: &crate::ast::types::StructUnionType,
    ) -> Value {
        let id = match self.mbx_free_ids.pop() {
            Some(id) => id,
            None => {
                self.mbx_msg_seq += 1;
                self.mbx_msg_seq
            }
        };
        let shadow = format!("__mbx_msg_{id}");
        self.copy_unpacked_struct(&shadow, src, su);
        Value::from_u64(id, 64)
    }

    /// Copy a stashed message out into `dst`. Read-only in the stash, so a
    /// `peek` can run any number of times before the message is consumed.
    pub(super) fn mbx_unstash_struct(
        &mut self,
        dst: &str,
        su: &crate::ast::types::StructUnionType,
        tok: &Value,
    ) -> bool {
        let Some(id) = tok.to_u64() else {
            return false;
        };
        if id == 0 {
            return false;
        }
        let shadow = format!("__mbx_msg_{id}");
        self.copy_unpacked_struct(dst, &shadow, su);
        true
    }

    /// A consumed slot releases its shadow id for the next `put`.
    pub(super) fn mbx_release_msg(&mut self, tok: &Value) {
        if let Some(id) = tok.to_u64() {
            if id != 0 && !self.mbx_free_ids.contains(&id) {
                self.mbx_free_ids.push(id);
            }
        }
    }

    /// The value a mailbox `put` queues for `arg`: an unpacked struct is
    /// stashed member-wise and queued as its shadow id.
    pub(super) fn mbx_message(&mut self, arg: &Expression) -> Value {
        match self.mbx_struct_arg(arg) {
            Some((src, su)) => self.mbx_stash_struct(&src, &su),
            None => self.eval_expr(arg),
        }
    }

    /// Write one delivered message into a parked waiter's lvalue, taking the
    /// aggregate path when the destination is an unpacked struct. Must run
    /// with the waiter's process context restored, so the lvalue's frame-local
    /// name resolves to that process's storage. A `consume`d (`get`, not
    /// `peek`) struct message frees its stash id; only a struct destination
    /// says the value IS one — a scalar message's value is not an id.
    pub(super) fn mbx_assign_delivered(&mut self, lvalue: &Expression, v: &Value, consume: bool) {
        if let Some((dst, su)) = self.mbx_struct_arg(lvalue) {
            self.mbx_unstash_struct(&dst, &su, v);
            if consume {
                self.mbx_release_msg(v);
            }
            return;
        }
        let width = self.infer_lhs_width(lvalue);
        self.assign_value(lvalue, &v.resize(width));
    }

    pub(super) fn deliver_to_mailbox_waiter(
        &mut self,
        pid: usize,
        lvalue: &Expression,
        v: Value,
        cont: ProcCont,
        consume: bool,
    ) {
        if let Some(ctx) = self.process_contexts.remove(&pid) {
            let saved = self.take_process_context();
            self.restore_process_context(ctx);
            self.mbx_assign_delivered(lvalue, &v, consume);
            let ctx = self.take_process_context();
            self.process_contexts.insert(pid, ctx);
            self.restore_process_context(saved);
        } else {
            self.mbx_assign_delivered(lvalue, &v, consume);
        }
        if !cont.is_empty() {
            self.event_queue.schedule(self.time, pid, cont);
        } else {
            self.child_finished(pid);
        }
    }
}
