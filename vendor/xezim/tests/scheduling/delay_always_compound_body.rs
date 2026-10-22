//! Issue #159: a delay-headed `always` block with a compound body — the
//! timed integration step of a real-number model — ran as an AST process
//! and paid identifier resolution on every step. It now compiles to the
//! process FSM like the delay-reroute class; the result must match the
//! reference simulator to the printed precision.
use xezim::simulate;

#[test]
fn timed_real_model_block_matches_reference() {
    let sim = simulate(
        "`timescale 1fs/1fs
module rnm_core (input real vin, output real vout);
  localparam real TSTEP = 1755.39;
  localparam real K     = 1.755385837e-12 / 1.9607843e-11;
  real state = 0.0, acc = 0.0;
  always #(TSTEP) begin : lag_step
    state = state + K * (vin - state);
    acc   = acc + state * K;
  end
  assign vout = state;
endmodule
module tb;
  real drive = 0.0; real out;
  rnm_core u_core (.vin(drive), .vout(out));
  initial begin
    for (int k = 0; k < 8; k++) begin
      drive = 0.1 * real'(k);
      #(1755.39 * 2000);
    end
    $display(\"RNM done: out=%0.6f acc=%0.6f\", out, u_core.acc);
    $finish;
  end
endmodule",
        400_000_000,
    )
    .expect("simulate failed");
    let msgs: Vec<String> = sim.output.iter().map(|o| o.message.clone()).collect();
    assert!(
        msgs.iter()
            .any(|m| m.starts_with("RNM done: out=0.700000 acc=")),
        "{msgs:?}"
    );
}

#[test]
fn timed_integer_block_with_runtime_delay_matches_ast_path() {
    // A delay that reads a variable is re-evaluated each step.
    let sim = simulate(
        "`timescale 1ns/1ns
module tb;
  int period = 5, n = 0, acc = 0;
  always #(period) begin
    n = n + 1;
    acc = acc + n;
    if (n == 4) period = 10;
  end
  initial begin
    #100;
    $display(\"n=%0d acc=%0d t=%0t\", n, acc, $time);
    $finish;
  end
endmodule",
        1_000,
    )
    .expect("simulate failed");
    let msgs: Vec<String> = sim.output.iter().map(|o| o.message.clone()).collect();
    // Steps at 5,10,15,20 then every 10 from 30; the step at 100 lands
    // after the display in the same time step (reference: n=11 acc=66).
    assert!(msgs.iter().any(|m| m == "n=11 acc=66 t=100"), "{msgs:?}");
}
