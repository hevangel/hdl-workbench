//! §4.4/§4.5 — the NBA region of a timestamp commits only after EVERY
//! active-region process at that timestamp has evaluated its RHS.
//!
//! `exec_statement`'s `#delay` arm called `apply_nba()` inline before
//! suspending. Inside an edge block that is premature: the other blocks on the
//! same edge have not sampled yet, so committing early lets them read the
//! POST-edge value. The trigger is ordinary testbench furniture — a single
//!
//!     always @(posedge clk) begin #1; ...checks... end
//!
//! observer (any monitor/scoreboard that samples a bit after the edge) broke
//! every OTHER clocked block that sampled a signal written by a sibling block
//! on that same edge. A one-cycle pipeline reference `d <= src;` recorded the
//! NEW value, so `d` showed no delay at all and every pipeline comparison
//! failed — while the DUT's own equivalent register was correct, which made it
//! look like the DUT was wrong.
//!
//! The flush is now skipped while `in_edge_block`; the scheduler's own NBA
//! point (`run_events_until`) commits at the right moment. A `#delay` outside
//! an edge block still flushes, since the active region there is just that
//! process.

use xezim::simulate;

fn u(sim: &xezim::compiler::Simulator, n: &str) -> u64 {
    sim.get_signal(n)
        .or_else(|| sim.get_signal(&format!("top.{}", n)))
        .unwrap_or_else(|| panic!("signal not found: {}", n))
        .to_u64()
        .unwrap_or_else(|| panic!("{} not u64-able (x/z?)", n))
}

/// Two clocked blocks and a `#1` observer. `d_hier` must lag `src` by one
/// cycle exactly as `d_local` does; before the fix the observer's presence made
/// the hierarchical sample land on the same cycle.
#[test]
fn delayed_observer_does_not_commit_nbas_mid_edge() {
    let src = r#"
module producer(input logic clk, input logic rst_n, output logic q);
  always_ff @(posedge clk) begin
    if (!rst_n) q <= 0;
    else        q <= ~q;
  end
endmodule
module top;
  logic clk = 0, rst_n = 0;
  always #5 clk = ~clk;
  logic s;
  producer p(.clk(clk), .rst_n(rst_n), .q(s));

  // Same-cycle sample of a signal a SIBLING block writes on this edge.
  logic d_hier;
  always_ff @(posedge clk) begin
    if (!rst_n) d_hier <= 0; else d_hier <= p.q;
  end

  // The observer whose `#1` used to force the NBA region early.
  int ticks;
  always @(posedge clk) begin
    #1;
    ticks++;
  end

  // The same sample taken through the LOCAL port-connected net. Both read
  // the identical signal on the identical edge, so they must always agree —
  // and neither may equal the post-edge value.
  logic d_local;
  always_ff @(posedge clk) begin
    if (!rst_n) d_local <= 0; else d_local <= s;
  end

  int mismatches, no_lag, samples;
  always @(posedge clk) begin
    #2;
    if (rst_n) begin
      samples++;
      if (d_hier !== d_local) mismatches++;
      // `s` toggles every cycle, so a correctly-delayed copy never equals it.
      // Skip the first post-reset sample: reset held both at 0, so agreeing
      // there is legitimate rather than a missing delay.
      if (samples > 1 && d_hier === s) no_lag++;
    end
  end

  initial begin
    repeat (3) @(posedge clk);
    #1 rst_n = 1;
    repeat (12) @(posedge clk);
    $finish;
  end
endmodule
"#;
    let sim = simulate(src, 400).expect("simulate failed");
    assert!(u(&sim, "samples") >= 8, "the checker must have run");
    assert_eq!(
        u(&sim, "mismatches"),
        0,
        "hierarchical and local samples of the same signal must agree"
    );
    assert_eq!(
        u(&sim, "no_lag"),
        0,
        "the sample must lag by a cycle, not track the post-edge value"
    );
    assert!(u(&sim, "ticks") >= 8, "the observer itself still runs");
}

/// A `#delay` OUTSIDE any edge block must still make the process's own
/// previously-scheduled NBA visible after the delay (§4.4) — the flush was
/// only narrowed, not removed.
#[test]
fn delay_outside_an_edge_block_still_sees_its_own_nba() {
    let src = r#"
module top;
  logic [7:0] v;
  int seen_after, seen_before;
  initial begin
    v = 8'h11;
    v <= 8'hEE;      // NBA scheduled here
    seen_before = v; // still the old value in this active region
    #1;
    seen_after = v;  // must be the committed NBA value
  end
endmodule
"#;
    let sim = simulate(src, 20).expect("simulate failed");
    assert_eq!(
        u(&sim, "seen_before"),
        0x11,
        "NBA is not visible in the same region"
    );
    assert_eq!(
        u(&sim, "seen_after"),
        0xEE,
        "NBA committed before the delay resumed"
    );
}

/// The multi-block clocked pipeline the interface testbench exercised: a DUT
/// register and a testbench register that sample the same signal on the same
/// edge must agree, with a delayed observer in the mix.
#[test]
fn sibling_clocked_samplers_agree_with_an_observer_present() {
    let src = r#"
interface bus_if;
  logic req_valid;
  logic rsp_valid;
  modport src (output req_valid, input  rsp_valid);
  modport sink(input  req_valid, output rsp_valid);
endinterface
module ctrl(input logic clk, input logic rst_n, bus_if.src b);
  always_ff @(posedge clk) begin
    if (!rst_n) b.req_valid <= 0; else b.req_valid <= ~b.req_valid;
  end
endmodule
module router(input logic clk, input logic rst_n, bus_if.sink b);
  always_ff @(posedge clk) begin
    if (!rst_n) b.rsp_valid <= 0; else b.rsp_valid <= b.req_valid;
  end
endmodule
module dutm(input logic clk, input logic rst_n);
  bus_if bus();
  ctrl   u_c(.clk(clk), .rst_n(rst_n), .b(bus));
  router u_r(.clk(clk), .rst_n(rst_n), .b(bus));
endmodule
module top;
  logic clk = 0, rst_n = 0;
  always #5 clk = ~clk;
  dutm dut(.clk(clk), .rst_n(rst_n));

  logic req_d;
  always_ff @(posedge clk) begin
    if (!rst_n) req_d <= 0; else req_d <= dut.bus.req_valid;
  end

  int ticks;
  always @(posedge clk) begin
    #1;
    ticks++;
  end

  int mismatches, samples;
  always @(posedge clk) begin
    #2;
    if (rst_n) begin
      samples++;
      // The DUT's register and the TB's register sampled the same signal on
      // the same edge — they must hold the same value.
      if (dut.bus.rsp_valid !== req_d) mismatches++;
    end
  end

  initial begin
    repeat (3) @(posedge clk);
    #1 rst_n = 1;
    repeat (12) @(posedge clk);
    $finish;
  end
endmodule
"#;
    let sim = simulate(src, 400).expect("simulate failed");
    assert!(u(&sim, "samples") >= 8, "the checker must have run");
    assert_eq!(u(&sim, "mismatches"), 0, "DUT and TB registers must agree");
}
