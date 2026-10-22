//! §23.6 / §23.7 — downward hierarchical references (`child.grandchild.sig`)
//! inside a module that is instantiated more than once. Reference-validated.
//!
//! Child INSTANCE names were not in the inliner's local-name set (only a
//! typedef-disambiguation corner added them), so a reference rooted at one
//! kept its bare spelling in EVERY copy of the module — and all copies
//! resolved to a single instance's signal. The field shape: two sibling
//! hierarchies each wait on their own done-strobe via
//! `@(posedge clk iff (sub.core.busy === 0))`; both fired when the FIRST
//! sibling's strobe fell, and the slower sibling sampled its result while
//! still x. A single-instance MWE with identical stimulus cannot reproduce
//! it, because with one copy the bare path happens to resolve correctly.

use xezim::simulate;

fn u(sim: &xezim::compiler::Simulator, n: &str) -> u64 {
    sim.get_signal(n)
        .or_else(|| sim.get_signal(&format!("tb.{}", n)))
        .unwrap_or_else(|| panic!("signal not found: {}", n))
        .to_u64()
        .unwrap_or_else(|| panic!("{} not u64-able (x/z?)", n))
}

/// Each wrapper waits on ITS OWN grandchild's busy flag; the two drop at
/// different times, and each result must be sampled at its own drop.
#[test]
fn iff_guard_through_instances_is_per_copy() {
    let src = r#"
module core(input logic clk, input logic go, input logic [7:0] delay,
            output logic busy, output logic [7:0] result);
  logic [7:0] cnt;
  initial begin
    busy = 1; result = 8'hxx; cnt = 0;
    wait (go);
    repeat (delay) @(posedge clk);
    result = 8'h40 + delay;
    busy = 0;
  end
endmodule
module sub(input logic clk, input logic go, input logic [7:0] delay);
  logic busy;
  logic [7:0] result;
  core u_core(clk, go, delay, busy, result);
endmodule
module wrapper(input logic clk, input logic go, input logic [7:0] delay,
               output logic [7:0] sampled, output logic ok);
  sub u_sub(clk, go, delay);
  initial begin
    ok = 0; sampled = 8'hxx;
    @(posedge clk iff (u_sub.u_core.busy === 1'b0));
    sampled = u_sub.u_core.result;      // downward ref in an expression too
    ok = !$isunknown(sampled);
  end
endmodule
module tb;
  logic clk = 0;
  logic go = 0;
  always #5 clk = ~clk;
  logic [7:0] s_fast, s_slow;
  logic ok_fast, ok_slow;
  wrapper w_fast(clk, go, 8'd3,  s_fast, ok_fast);
  wrapper w_slow(clk, go, 8'd11, s_slow, ok_slow);
  int r_fast, r_slow, r_okf, r_oks;
  initial begin
    #12 go = 1;
    wait (ok_fast === 1'b1 && ok_slow === 1'b1);
    #1;
    r_fast = s_fast; r_slow = s_slow;
    r_okf = ok_fast; r_oks = ok_slow;
  end
endmodule
"#;
    let sim = simulate(src, 1000).expect("simulate failed");
    assert_eq!(u(&sim, "r_okf"), 1, "the fast copy sampled a known value");
    assert_eq!(
        u(&sim, "r_oks"),
        1,
        "the slow copy did NOT fire on the fast copy's strobe"
    );
    assert_eq!(
        u(&sim, "r_fast"),
        0x43,
        "each copy sees its own grandchild's result"
    );
    assert_eq!(u(&sim, "r_slow"), 0x4B);
}
