//! §4c: class storage crossed with MODULE-SCOPE paths — reference-validated.
//!
//! Three stacked gaps, found while validating the #109 event fix:
//! 1. `h.prop = new` / `h.prop` READ through a MODULE-SCOPE handle variable
//!    landed on a phantom flattened name (writes vanished, reads returned x).
//!    Both parse shapes route to the heap object now, guarded so a REAL
//!    hierarchical signal (`u1.sig`) always wins.
//! 2. `h.assoc[k] = new` could not resolve the element class (the resolver
//!    only saw module collections and the current class context) — resolved
//!    from the receiver's runtime class now.
//! 3. `tb.counter++` from a class method was a silent no-op — `<top>.<sig>`
//!    now strips the top-module prefix on both read and write when no
//!    composed signal exists.

use xezim::simulate;

fn outs(sim: &xezim::compiler::Simulator) -> Vec<String> {
    sim.output.iter().map(|o| o.message.clone()).collect()
}

/// Reference: tag=11 null=0 (plain property store + read through module h).
#[test]
fn property_store_through_module_scope_handle() {
    let src = r#"
module tb;
  class A; int tag; endclass
  class B; A direct; endclass
  B h; A m;
  initial begin
    h = new; h.direct = new; h.direct.tag = 11;
    m = h.direct;
    $display("T|tag=%0d null=%0d", m.tag, (m == null));
  end
endmodule
"#;
    let sim = simulate(src, 10).expect("simulate failed");
    assert!(
        outs(&sim).contains(&"T|tag=11 null=0".to_string()),
        "{:?}",
        outs(&sim)
    );
}

/// Reference: tags 11/22/33/44 both in-class and at module scope — plain
/// property, assoc keyed by class handle, assoc keyed by int, queue.
#[test]
fn collection_stores_through_module_scope_handle() {
    let src = r#"
module tb;
  class events_t; int tag; endclass
  class holder_t;
    events_t direct;
    events_t by_obj[holder_t];
    events_t by_int[int];
    events_t q[$];
    task show(holder_t k);
      $display("T|in=%0d %0d %0d %0d", direct.tag, by_obj[k].tag, by_int[7].tag, q[0].tag);
    endtask
  endclass
  holder_t h, key;
  events_t tmp;
  initial begin
    h = new; key = new;
    h.direct = new;      h.direct.tag = 11;
    h.by_obj[key] = new; h.by_obj[key].tag = 22;
    h.by_int[7] = new;   h.by_int[7].tag = 33;
    tmp = new; tmp.tag = 44; h.q.push_back(tmp);
    h.show(key);
    $display("T|mod=%0d %0d %0d %0d", h.direct.tag, h.by_obj[key].tag, h.by_int[7].tag, h.q[0].tag);
  end
endmodule
"#;
    let sim = simulate(src, 10).expect("simulate failed");
    let o = outs(&sim);
    assert!(o.contains(&"T|in=11 22 33 44".to_string()), "{o:?}");
    assert!(o.contains(&"T|mod=11 22 33 44".to_string()), "{o:?}");
}

/// Reference: created=1 — a hierarchical `<top>.<var>` write from a class
/// method reaches the top module's variable.
#[test]
fn hier_write_from_class_method_reaches_top_module() {
    let src = r#"
module tb;
  int counter = 0;
  class worker_t;
    task bump();
      tb.counter++;
      tb.counter++;
    endtask
  endclass
  worker_t w;
  initial begin
    w = new;
    w.bump();
    $display("T|counter=%0d", counter);
  end
endmodule
"#;
    let sim = simulate(src, 10).expect("simulate failed");
    assert!(
        outs(&sim).contains(&"T|counter=2".to_string()),
        "{:?}",
        outs(&sim)
    );
}

/// Module-scope event triggers through selected receivers — the full event
/// matrix that these storage bugs originally invalidated.
/// Reference: module=10 direct=20 by_obj=30 by_int=40 queue=50.
#[test]
fn module_scope_event_matrix_end_to_end() {
    let src = r#"
module tb;
  int t_m = -1, t_d = -1, t_o = -1, t_i = -1, t_q = -1;
  class events_t; event ev; endclass
  class holder_t;
    events_t direct;
    events_t by_obj[holder_t];
    events_t by_int[int];
    events_t q[$];
    task wait_direct;             @(direct.ev);    endtask
    task wait_by_obj(holder_t k); @(by_obj[k].ev); endtask
    task wait_by_int;             @(by_int[7].ev); endtask
    task wait_q;                  @(q[0].ev);      endtask
  endclass
  holder_t h, key;
  events_t m_ev, tmp;
  initial begin
    h = new; key = new;
    h.direct = new;
    h.by_obj[key] = new;
    h.by_int[7] = new;
    tmp = new; h.q.push_back(tmp);
    m_ev = new;
    fork
      begin @(m_ev.ev);          t_m = $time; end
      begin h.wait_direct();     t_d = $time; end
      begin h.wait_by_obj(key);  t_o = $time; end
      begin h.wait_by_int();     t_i = $time; end
      begin h.wait_q();          t_q = $time; end
      begin
        #10 ->m_ev.ev;
        #10 ->h.direct.ev;
        #10 ->h.by_obj[key].ev;
        #10 ->h.by_int[7].ev;
        #10 ->h.q[0].ev;
      end
    join
    $display("T|%0d %0d %0d %0d %0d", t_m, t_d, t_o, t_i, t_q);
  end
endmodule
"#;
    let sim = simulate(src, 1000).expect("simulate failed");
    assert!(
        outs(&sim).contains(&"T|10 20 30 40 50".to_string()),
        "{:?}",
        outs(&sim)
    );
}
