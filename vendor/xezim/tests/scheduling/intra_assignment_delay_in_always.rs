//! Issue #160: an intra-assignment delay (`q <= #5 v;`, `q = #5 v;`,
//! §9.4.5) inside an edge-triggered `always` block was silently dropped —
//! the compiled edge path evaluated the delay marker as an unknown system
//! call and assigned at once. The `initial` form was already correct.
//! Expected times are those of the reference simulator.
//!
//! Also the `assign #(rise, fall[, turnoff])` form (§10.3.3), which the
//! parser rejected.
use xezim::simulate;

fn messages(src: &str) -> Vec<String> {
    let sim = simulate(src, 1_000).expect("simulate failed");
    sim.output.iter().map(|o| o.message.clone()).collect()
}

#[test]
fn nonblocking_intra_delay_in_always_is_honoured() {
    let msgs = messages(
        "`timescale 1ns/1ns
module tb;
  parameter int  IP = 5;
  parameter real RP = 5.0;
  logic a = 0;
  logic q1, q2, q3, q4, q5, q6;
  real t0, t1, t2, t3, t4, t5, t6;
  initial begin #10; t0 = $realtime; a = 1; end
  initial begin @(posedge a); q1 <= #5 1'b1; end
  always @(posedge a) q2 <= #5 1'b1;
  always @(posedge a) begin q3 <= #5   1'b1; end
  always @(posedge a) begin q4 <= #(5) 1'b1; end
  always @(posedge a) begin q5 <= #(IP) 1'b1; end
  always @(posedge a) begin q6 <= #(RP) 1'b1; end
  always @(posedge q1) t1 = $realtime;
  always @(posedge q2) t2 = $realtime;
  always @(posedge q3) t3 = $realtime;
  always @(posedge q4) t4 = $realtime;
  always @(posedge q5) t5 = $realtime;
  always @(posedge q6) t6 = $realtime;
  initial begin
    #40;
    $display(\"1 %0.1f\", t1 - t0);
    $display(\"2 %0.1f\", t2 - t0);
    $display(\"3 %0.1f\", t3 - t0);
    $display(\"4 %0.1f\", t4 - t0);
    $display(\"5 %0.1f\", t5 - t0);
    $display(\"6 %0.1f\", t6 - t0);
    $finish;
  end
endmodule",
    );
    for want in ["1 5.0", "2 5.0", "3 5.0", "4 5.0", "5 5.0", "6 5.0"] {
        assert!(msgs.iter().any(|m| m == want), "missing {want}: {msgs:?}");
    }
}

#[test]
fn blocking_intra_delay_in_always_suspends_the_block() {
    let msgs = messages(
        "`timescale 1ns/1ns
module tb;
  logic a = 0; logic qa, qb, qc; real t0, ta, tb, tc;
  initial begin #10; t0 = $realtime; a = 1; end
  always @(posedge a) qa = #5 1'b1;
  always @(posedge a) begin qb = #5 1'b1; end
  initial begin @(posedge a); qc = #5 1'b1; end
  always @(posedge qa) ta = $realtime;
  always @(posedge qb) tb = $realtime;
  always @(posedge qc) tc = $realtime;
  initial begin #40; $display(\"a=%0.1f b=%0.1f c=%0.1f\", ta-t0, tb-t0, tc-t0); $finish; end
endmodule",
    );
    assert!(msgs.iter().any(|m| m == "a=5.0 b=5.0 c=5.0"), "{msgs:?}");
}

#[test]
fn continuous_assign_rise_fall_turnoff_delays() {
    let msgs = messages(
        "`timescale 1ns/1ns
module tb;
  localparam int RISE_FS = 3, FALL_FS = 7;
  logic nv = 0; wire z;
  assign #(RISE_FS, FALL_FS) z = nv;
  real tr, tf, t0;
  always @(posedge z) tr = $realtime - t0;
  always @(negedge z) tf = $realtime - t0;
  initial begin #10; t0 = $realtime; nv = 1; #20; t0 = $realtime; nv = 0; #20;
    $display(\"rise=%0.1f fall=%0.1f\", tr, tf); $finish; end
endmodule",
    );
    assert!(msgs.iter().any(|m| m == "rise=3.0 fall=7.0"), "{msgs:?}");

    let msgs = messages(
        "`timescale 1ns/1ns
module tb;
  logic drv = 0; logic oe = 1; wire z;
  assign #(2, 4, 6) z = oe ? drv : 1'bz;
  real t0; real tr = -1, tf = -1, tz = -1;
  always @(z) begin
    if (z === 1'b1) tr = $realtime - t0;
    else if (z === 1'b0) tf = $realtime - t0;
    else if (z === 1'bz) tz = $realtime - t0;
  end
  initial begin
    #10; t0 = $realtime; drv = 1;
    #20; t0 = $realtime; drv = 0;
    #20; t0 = $realtime; oe = 0;
    #20; $display(\"rise=%0.1f fall=%0.1f off=%0.1f\", tr, tf, tz); $finish;
  end
endmodule",
    );
    assert!(
        msgs.iter().any(|m| m == "rise=2.0 fall=4.0 off=6.0"),
        "{msgs:?}"
    );
}
