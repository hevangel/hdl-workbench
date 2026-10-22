//! Two fixes found chasing the "specify path delays are never applied" gap —
//! which turned out to be two separate bugs, neither quite that.
//!
//! 1. §30.4 — a specify path delay (`(a => y) = 4`) reaches the runtime as an
//!    `sdf_delays[dst]` entry, and the DirectCopy / fused-gate paths already
//!    refuse a delayed destination — but a cell whose output is computed
//!    (`assign y = a & b;`, i.e. every behavioral gate model) was bytecode-
//!    compiled, and the compiled writeback commits immediately. The compile
//!    branch now also requires a delay-free destination, so those fall to the
//!    AST path, which schedules through the delayed-update queue.
//!
//! 2. §9.4.2 — `always @(a) t = $time;` fired exactly once. A level-only
//!    event-controlled block is routed to the comb-settle path, whose
//!    sensitivity is re-derived from the body's READS; the faithfulness check
//!    only verified reads ⊆ list, which passes VACUOUSLY for a body that
//!    reads no signal — the list was dropped and the entry had no
//!    sensitivity at all. The check now requires the id sets to be equal;
//!    blocks whose list names signals the body doesn't read go to the edge
//!    path, which fires on the list exactly.
//!
//! Both verified byte-identical against a reference simulator.

use xezim::simulate;

fn u(sim: &xezim::compiler::Simulator, n: &str) -> u64 {
    sim.get_signal(n)
        .or_else(|| sim.get_signal(&format!("tb.{}", n)))
        .unwrap_or_else(|| panic!("signal not found: {}", n))
        .to_u64()
        .unwrap_or_else(|| panic!("{} not u64-able (x/z?)", n))
}

/// The full path-delay picture: an assign-bodied cell and a gate-bodied cell,
/// each with a specify path, sampled by posedge observers.
#[test]
fn specify_path_delays_apply_to_assign_and_gate_cells() {
    let src = r#"
`timescale 1ns/1ns
module buf_cell (input a, output y);
  assign y = a;
  specify
    (a => y) = 4;
  endspecify
endmodule
module gate_cell (input a, b, output y);
  and g1(y, a, b);
  specify
    (a => y) = 3;
    (b => y) = 3;
  endspecify
endmodule
module tb;
  reg a, b;
  wire y1, y2;
  buf_cell  u1(.a(a), .y(y1));
  gate_cell u2(.a(a), .b(b), .y(y2));
  int t_y1, t_y2;
  always @(posedge y1) t_y1 = $time;
  always @(posedge y2) t_y2 = $time;
  initial begin
    a = 0; b = 1;
    #10 a = 1;
    #10 $finish;
  end
endmodule
"#;
    let sim = simulate(src, 100).expect("simulate failed");
    assert_eq!(u(&sim, "t_y1"), 14, "assign-cell path delay (a => y) = 4");
    assert_eq!(
        u(&sim, "t_y2"),
        13,
        "gate-cell path delay (a => y) = 3 — was 10 (dropped)"
    );
}

/// A computed-RHS cell output still works with NO specify — the compile gate
/// must not change plain behavior.
#[test]
fn undelayed_computed_cell_output_still_immediate() {
    let src = r#"
`timescale 1ns/1ns
module and_cell (input a, b, output y);
  assign y = a & b;
endmodule
module tb;
  reg a, b;
  wire y;
  and_cell u1(.a(a), .b(b), .y(y));
  int t_y;
  always @(posedge y) t_y = $time;
  initial begin
    a = 0; b = 1;
    #10 a = 1;
    #10 $finish;
  end
endmodule
"#;
    let sim = simulate(src, 100).expect("simulate failed");
    assert_eq!(u(&sim, "t_y"), 10, "no specify, no delay");
}

/// §9.4.2: an explicit level-sensitivity list fires the block even when the
/// body reads none of the listed signals.
#[test]
fn event_list_fires_a_body_that_reads_nothing() {
    let src = r#"
`timescale 1ns/1ns
module tb;
  reg a;
  int t_pos, t_any, t_neg, runs;
  always @(posedge a) t_pos = $time;
  always @(a)         t_any = $time;
  always @(negedge a) t_neg = $time;
  always @(a)         runs  = runs + 1;
  initial begin
    a = 0;
    #10 a = 1;
    #10 a = 0;
    #10 $finish;
  end
endmodule
"#;
    let sim = simulate(src, 100).expect("simulate failed");
    assert_eq!(u(&sim, "t_pos"), 10, "posedge capture");
    assert_eq!(u(&sim, "t_any"), 20, "any-edge capture — was stuck at 0");
    assert_eq!(u(&sim, "t_neg"), 20, "negedge capture");
    assert!(
        u(&sim, "runs") >= 2,
        "the counter body re-fires per listed change"
    );
}

/// A list wider than the body's reads must fire on the extra signal too —
/// `@(a or b)` with a body reading only `a` still triggers on `b`.
#[test]
fn event_list_superset_of_reads_triggers_on_the_extra_signal() {
    let src = r#"
`timescale 1ns/1ns
module tb;
  reg a, b;
  int last_t;
  always @(a or b) last_t = $time;
  initial begin
    a = 0; b = 0;
    #10 a = 1;
    #10 b = 1;      // body reads neither — must still capture t=20
    #10 $finish;
  end
endmodule
"#;
    let sim = simulate(src, 100).expect("simulate failed");
    assert_eq!(u(&sim, "last_t"), 20, "the b edge fires the block");
}
