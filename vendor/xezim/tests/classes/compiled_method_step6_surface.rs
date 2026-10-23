//! class-perf Step 6 — the WIDENED class-method compilation surface:
//!
//! * CLASS-TYPED LOCALS in a method body (`phase sched;`) hold heap handles
//!   in registers (default null, no resize);
//! * `this` as an rvalue (`sched = this;`);
//! * ENUM members and ENUM returns (the value is the enum's base width);
//! * a member CHAIN as a call receiver (`sched.m_parent.get_ptype()`) and a
//!   BARE member as a call receiver (`m_parent.ptype` — resolves to
//!   `this.m_parent` first, then reads `.ptype` off that handle);
//! * `null` literals in handle comparisons;
//! * `return this / local / m_parent / null` all round-trip handles untouched;
//! * member-chain STORES (`a.b.c = v`) use the runtime bare key, exactly like
//!   the interpreter's dotted access;
//! * a QUEUE-typed member (`int q[$]`) must stay on the AST path — a bare
//!   read of it (or `.size()` on it) silently becoming `this.<method>` caused
//!   an infinite dispatch recursion (uvm_queue::size regression).
//!
//! All expectations are reference-simulator-validated (method_vm10 shape and
//! its reference simulator run print identical values). The gate is forced ON so
//! a plain `cargo test` exercises the compiled path; the suite is also run
//! with XEZIM_COMPILE_METHODS unset (AST path) — both must pass identically.

use xezim::simulate;

fn u(sim: &xezim::compiler::Simulator, n: &str) -> u64 {
    sim.get_signal(n)
        .or_else(|| sim.get_signal(&format!("tb.{}", n)))
        .unwrap_or_else(|| panic!("signal not found: {}", n))
        .to_u64()
        .unwrap_or_else(|| panic!("{} is x/z", n))
}

fn gate_on() -> bool {
    super::compiled_method_test_env::eager()
}

/// The `uvm_phase::get_schedule` shape: class-typed local seeded from `this`,
/// a while-walk over `m_parent` with a chain-call receiver, enum property
/// reads, a bare member used as a receiver, and handle returns of every
/// flavor. Reference simulator: `TAG_SCHED pass=2` for this exact topology.
#[test]
fn class_local_walk_enum_return_and_chain_calls() {
    if !gate_on() {
        return;
    }
    let src = r#"
typedef enum int { PH_NODE = 0, PH_SCHED = 1, PH_DOM = 2 } phase_type_e;
class phase;
  phase_type_e ptype;
  phase m_parent;
  int id;
  function new(int i, phase_type_e t);
    id = i; ptype = t; m_parent = null;
  endfunction
  function phase get_parent(); return m_parent; endfunction
  function phase_type_e get_ptype(); return ptype; endfunction
  function phase get_sched(bit hier = 0);
    phase sched;
    sched = this;
    if (hier) begin
      while (sched.m_parent != null && sched.m_parent.get_ptype() == PH_SCHED) begin
        sched = sched.m_parent;
      end
    end
    if (sched.ptype == PH_SCHED) begin
      return sched;
    end
    if (sched.ptype == PH_NODE) begin
      if (m_parent != null && m_parent.ptype != PH_DOM) begin
        return m_parent;
      end
    end
    return null;
  endfunction
endclass
module tb;
  int r_hier, r_nohier, r_from_node, r_ptype, r_null;
  initial begin
    // A real schedule chain: node -> schedule -> root schedule.
    phase root = new(10, PH_SCHED);
    phase mid  = new(20, PH_SCHED);
    phase node = new(30, PH_NODE);
    phase s1, s2, s3;
    mid.m_parent  = root;
    node.m_parent = mid;
    // hier walk from the node: node -> mid -> root, return the root.
    s1 = node.get_sched(1);
    if (s1 == null) r_hier = 99; else r_hier = s1.id;
    // no walk: a NODE returns its parent when that parent is not a domain.
    s2 = node.get_sched(0);
    if (s2 == null) r_nohier = 99; else r_nohier = s2.id;
    // hier walk from the middle schedule lands on the root.
    s3 = mid.get_sched(1);
    if (s3 == null) r_from_node = 99; else r_from_node = s3.id;
    r_ptype = node.get_ptype();
    if (root.get_parent() == null) r_null = 1; else r_null = 0;
  end
endmodule
"#;
    let sim = simulate(src, 50).expect("simulate failed");
    assert_eq!(
        u(&sim, "r_hier"),
        10,
        "hier walk from the node reaches the root"
    );
    assert_eq!(
        u(&sim, "r_nohier"),
        20,
        "no-walk on a node returns its (non-domain) parent"
    );
    assert_eq!(
        u(&sim, "r_from_node"),
        10,
        "walk from the middle schedule reaches the root"
    );
    assert_eq!(u(&sim, "r_ptype"), 0, "enum return = PH_NODE");
    assert_eq!(u(&sim, "r_null"), 1, "root's parent is null");
}

/// A member chain as a STORE target (`a.b.id = v`) and as an rvalue
/// (`a.b.id`): the dotted access uses the instance's runtime bare key, so
/// the compiled Load/StoreClassMember sequence must agree with the AST path.
#[test]
fn chain_store_and_read_through_handle_member() {
    if !gate_on() {
        return;
    }
    let src = r#"
class node;
  int id;
  node next;
  function new(int i); id = i; next = null; endfunction
  function int get_id(); return id; endfunction
endclass
module tb;
  int r1, r2, r3;
  initial begin
    node a = new(1);
    node b = new(2);
    a.next = b;
    a.next.id = 55;        // chain STORE through the handle member
    r1 = a.next.get_id();  // chain receiver -> dispatch on b
    r2 = a.next.id;        // chain READ of the plain member
    b.id = 66;
    r3 = a.next.id;        // must observe the same object (bare key)
  end
endmodule
"#;
    let sim = simulate(src, 50).expect("simulate failed");
    assert_eq!(u(&sim, "r1"), 55, "store through the chain landed on b");
    assert_eq!(u(&sim, "r2"), 55, "chain read sees the stored value");
    assert_eq!(u(&sim, "r3"), 66, "a.next and b are the same object");
}

/// REGRESSION (uvm_queue::size): a queue-typed member must never be
/// bare-resolved. `return q.size();` with `int q[$]` once lowered the
/// receiver to `this`, making `size()` dispatch into itself — an infinite
/// runtime recursion (stack overflow). The compiled path must refuse the
/// whole method and the AST interpreter must answer the queue builtin.
#[test]
fn queue_member_size_stays_interpreted() {
    if !gate_on() {
        return;
    }
    let src = r#"
class iq;
  int q[$];
  function int sz(); return q.size(); endfunction
endclass
module tb;
  int r1, r2, r3;
  initial begin
    iq i = new();
    i.q.push_back(5);
    i.q.push_back(6);
    r1 = i.sz();
    i.q.pop_back();
    r2 = i.sz();
    r3 = i.q.size();
  end
endmodule
"#;
    let sim = simulate(src, 50).expect("simulate failed");
    assert_eq!(u(&sim, "r1"), 2, "queue member size via the method");
    assert_eq!(u(&sim, "r2"), 1, "size after pop");
    assert_eq!(u(&sim, "r3"), 1, "direct builtin agrees");
}
