//! `instance_assoc_member` resolves a class-member collection against the
//! RUNTIME type of `this` (the heap's `class_name`) rather than the lexical
//! `class_context_stack`.
//!
//! A bare, unqualified method call (`body()` from an inherited `start()`) is
//! inlined without pushing a new class context, so the stack can still name
//! the CALLER's class. Resolving a member array through it then searches the
//! wrong class, finds nothing, and `foreach_materialize_keys_1d` returns
//! `None` — which the async `ForeachTail` path reads as "not ready" and
//! replays, so the loop repeats index 0 forever instead of erroring.
//!
//! The lookup is a pure existence check (the value returned is always
//! `<handle>#<member>`), and the runtime class is always a descendant of the
//! lexical one, so walking from the runtime class covers a strict SUPERSET of
//! the old chain — the change can only resolve more names, never fewer.
//!
//! CHARACTERIZATION, not a red-then-green regression: the divergence could not
//! be provoked here. Nine call shapes (bare call from an inherited method, a
//! base-typed handle, a cross-class caller, `fork`/`join`, a three-level
//! hierarchy, non-virtual dispatch, and assoc/queue members) plus all seven
//! AVIP UVM testbenches instrumented on the exact predicate produced zero
//! divergences. These cases pin the resolution behaviour so a future change to
//! the context stack cannot quietly regress subclass member lookup.

use xezim::simulate;

fn notes(src: &str) -> Vec<String> {
    let sim = simulate(src, 1_000_000).expect("simulate failed");
    sim.output
        .iter()
        .map(|o| o.message.trim().to_string())
        .filter(|l| l.starts_with("NOTE:"))
        .collect()
}

const BARE_CALL_FIXED: &str = r#"
`timescale 1ns/1ps
module top;
  logic clk = 0;
  always #5 clk = ~clk;
  class seq_base;
    virtual task body(); endtask
    task start(); body(); endtask   // bare + unqualified
  endclass
  class my_seq extends seq_base;
    int items[3];
    function new(); items[0]=10; items[1]=20; items[2]=30; endfunction
    virtual task body();
      foreach (items[i]) begin
        @(posedge clk);             // blocking -> async ForeachTail path
        $display("NOTE: item[%0d]=%0d", i, items[i]);
      end
    endtask
  endclass
  initial begin
    seq_base sb; my_seq ms;
    ms = new(); sb = ms;
    sb.start();                     // dispatch through a BASE-typed handle
    $finish;
  end
endmodule
"#;

/// §8.14 — a member array declared in the concrete subclass resolves when the
/// enclosing method is reached by bare dispatch from an inherited method.
#[test]
fn subclass_member_array_resolves_through_inherited_bare_call() {
    assert_eq!(
        notes(BARE_CALL_FIXED),
        vec!["NOTE: item[0]=10", "NOTE: item[1]=20", "NOTE: item[2]=30",],
        "foreach must complete all three iterations, not replay index 0"
    );
}

const BARE_CALL_ASSOC_QUEUE: &str = r#"
`timescale 1ns/1ps
module top;
  logic clk = 0;
  always #5 clk = ~clk;
  class seq_base;
    virtual task body(); endtask
    task start(); body(); endtask
  endclass
  class my_seq extends seq_base;
    int    amap[string];
    string q[$];
    function new();
      amap["a"] = 1; amap["b"] = 2;
      q.push_back("x"); q.push_back("y");
    endfunction
    virtual task body();
      foreach (amap[k]) begin
        @(posedge clk);
        $display("NOTE: amap[%s]=%0d", k, amap[k]);
      end
      foreach (q[i]) begin
        @(posedge clk);
        $display("NOTE: q[%0d]=%s", i, q[i]);
      end
    endtask
  endclass
  initial begin
    my_seq ms; ms = new(); ms.start(); $finish;
  end
endmodule
"#;

/// The same resolution covers associative and queue members — the other two
/// maps `instance_assoc_member` consults.
#[test]
fn subclass_assoc_and_queue_members_resolve_through_bare_call() {
    assert_eq!(
        notes(BARE_CALL_ASSOC_QUEUE),
        vec![
            "NOTE: amap[a]=1",
            "NOTE: amap[b]=2",
            "NOTE: q[0]=x",
            "NOTE: q[1]=y",
        ]
    );
}
