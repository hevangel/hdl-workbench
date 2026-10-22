//! §9.2.2.2 always_comb audit vs the reference — four sensitivity/scoping
//! defects, each reference-validated. A comb block that misses a re-fire
//! keeps its time-0 value forever, which reads as "the output is stuck at
//! reset" in a real design.
//!
//! 1. **Queue reads had no dependency at all.** A queue's elements and size
//!    live in the runtime map with no signal ids, so `always_comb sz =
//!    q.size();` never re-fired on push/pop/delete. Every dynamic array now
//!    materializes a real `<q>.size` signal that doubles as its change proxy:
//!    mutations mark it dirty, and element reads register it as their dep.
//! 2. **§6.21 — a comb block-local shadowed INTO the module variable.** The
//!    frameless VarDecl exec landed on the module signal, so `begin logic
//!    [7:0] v; v = 8'hF0; end` overwrote module `v`. The bytecode compiler now
//!    hard-fails a shadowing declaration so the whole block interprets, and
//!    the SeqBlock exec pushes a shadow frame (like for-loop variables).
//! 3. **`arr[i].field` registered no per-element FIELD reads.** The write
//!    dirties `arr[0].field`; the dep set only had the elementless base, so a
//!    field write never re-fired the reader.
//! 4. **§11.4.13 — `inside` had no read-collection arm.** A comb or
//!    continuous assign with an `inside` RHS collected nothing and kept its
//!    time-0 value.

use xezim::simulate;

fn u(sim: &xezim::compiler::Simulator, n: &str) -> u64 {
    sim.get_signal(n)
        .or_else(|| sim.get_signal(&format!("tb.{}", n)))
        .unwrap_or_else(|| panic!("signal not found: {}", n))
        .to_u64()
        .unwrap_or_else(|| panic!("{} not u64-able (x/z?)", n))
}

#[test]
fn comb_refires_on_queue_mutations() {
    let src = r#"
module tb;
  int q[$];
  int sz, head;
  int a_sz, a_head, b_sz, b_head, c_sz, d_sz, d_head;
  always_comb sz = q.size();
  always_comb head = (q.size() > 0) ? q[0] : -1;
  initial begin
    #1; a_sz = sz; a_head = head;
    q.push_back(42);
    #1; b_sz = sz; b_head = head;
    q.push_back(7);
    void'(q.pop_front());
    #1; c_sz = sz; d_head = head;
    q.delete();
    #1; d_sz = sz;
  end
endmodule
"#;
    let sim = simulate(src, 50).expect("simulate failed");
    assert_eq!(u(&sim, "a_sz"), 0);
    assert_eq!(u(&sim, "a_head") as i32, -1);
    assert_eq!(u(&sim, "b_sz"), 1, "push must re-fire the size reader");
    assert_eq!(u(&sim, "b_head"), 42);
    assert_eq!(u(&sim, "c_sz"), 1, "push+pop nets one element");
    assert_eq!(u(&sim, "d_head"), 7, "pop re-selects the new head");
    assert_eq!(u(&sim, "d_sz"), 0, "delete must re-fire too");
}

#[test]
fn comb_block_local_shadows_module_var() {
    let src = r#"
module tb;
  logic [7:0] v;
  logic [7:0] o1;
  int vv, oo;
  always_comb begin
    logic [7:0] v;          // shadows the module's v (§6.21)
    v = 8'hF0;
    o1 = v;
  end
  initial begin
    v = 8'h0F;
    #1;
    vv = v;                 // module v must be untouched
    oo = o1;
  end
endmodule
"#;
    let sim = simulate(src, 50).expect("simulate failed");
    assert_eq!(u(&sim, "oo"), 0xF0, "block sees its own local");
    assert_eq!(u(&sim, "vv"), 0x0F, "module v must NOT be clobbered");
}

#[test]
fn comb_refires_on_struct_array_field_write() {
    let src = r#"
module tb;
  typedef struct { logic [3:0] tag; logic [3:0] val; } e_t;
  e_t arr [3];
  logic [1:0] pick;
  logic [3:0] o;
  int a, b, c, d;
  always_comb o = arr[pick].val;
  initial begin
    for (int i = 0; i < 3; i++) begin arr[i].tag = i[3:0]; arr[i].val = 4'h4 + i[3:0]; end
    pick = 0;
    #1; a = o;
    arr[0].val = 4'hA;      // selected element's field
    #1; b = o;
    pick = 2;
    #1; c = o;
    arr[2].val = 4'h1;
    #1; d = o;
  end
endmodule
"#;
    let sim = simulate(src, 50).expect("simulate failed");
    assert_eq!(u(&sim, "a"), 4);
    assert_eq!(u(&sim, "b"), 0xA, "field write must re-fire the reader");
    assert_eq!(u(&sim, "c"), 6);
    assert_eq!(u(&sim, "d"), 1);
}

#[test]
fn inside_operator_collects_reads() {
    let src = r#"
module tb;
  logic [7:0] v;
  logic [7:0] lo;
  logic o1, o2;
  int a1, a2, b1, b2, c1;
  always_comb o1 = v inside {8'h10, [8'h20:8'h2F]};
  assign o2 = v inside {[lo:8'h2F]};
  initial begin
    v = 8'h00; lo = 8'h20;
    #1; a1 = o1; a2 = o2;
    v = 8'h25;
    #1; b1 = o1; b2 = o2;
    lo = 8'h26;             // set-member expression changes
    #1; c1 = o2;
  end
endmodule
"#;
    let sim = simulate(src, 50).expect("simulate failed");
    assert_eq!(u(&sim, "a1"), 0);
    assert_eq!(u(&sim, "a2"), 0);
    assert_eq!(u(&sim, "b1"), 1, "comb inside must re-fire on the operand");
    assert_eq!(u(&sim, "b2"), 1, "cont-assign inside must re-fire too");
    assert_eq!(u(&sim, "c1"), 0, "range bound change re-evaluates");
}
