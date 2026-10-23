//! An `always @(posedge clk)` body that CALLS a blocking task must classify
//! as a process, not an edge block. On the edge path the callee's first
//! `#delay` re-enters the time loop synchronously while this slot's NBAs
//! are still queued (`apply_nba` is skipped inside edge blocks to protect
//! sibling sampling), so `q <= d;` scheduled before the call committed
//! picoseconds LATE — a reset-release pipeline flop stamped at t+6ps in a
//! BFM. Both expected values below are REFERENCE-VERIFIED process
//! semantics (§9.2.2): NBAs commit in their own slot, and edges arriving
//! while the body is mid-call are missed.

use std::process::Command;

fn run_default(src: &str, tag: &str) -> String {
    let dir = std::env::temp_dir().join(format!(
        "xezim_edge_taskcall_{}_{}",
        tag,
        std::process::id()
    ));
    let _ = std::fs::create_dir_all(&dir);
    let f = dir.join("t.sv");
    std::fs::write(&f, src).unwrap();
    let out = Command::new(env!("CARGO_BIN_EXE_xezim"))
        .args(["--no-cache", "-s", "tb", "--max-time", "1000"])
        .arg(&f)
        .output()
        .expect("run xezim");
    format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    )
}

#[test]
fn nba_before_blocking_call_commits_in_slot() {
    let src = r#"
`timescale 1ps/1ps
module tb;
  reg clk = 0;
  always #50 clk = ~clk;
  reg rst_l = 0;
  reg rst_l_d1 = 0;
  reg [7:0] mode = 0;
  task slow_mode;
    begin
      #6;
      mode = mode + 1;
    end
  endtask
  always @(posedge clk) begin
    rst_l_d1 <= rst_l;
    if (rst_l) slow_mode;
  end
  initial begin
    #120 rst_l = 1;
    @(rst_l_d1);
    $display("D1RISE t=%0t mode=%0d", $time, mode);
    #300 $finish;
  end
endmodule
"#;
    let out = run_default(src, "nba_slot");
    // The posedge at t=150 samples rst_l=1 and queues rst_l_d1<=1; the
    // commit must land at 150, not after the callee's #6 (156).
    assert!(
        out.contains("D1RISE t=150 "),
        "rst_l_d1 NBA must commit in its own slot (t=150): {}",
        out
    );
}

#[test]
fn call_blocking_body_misses_busy_edges() {
    let src = r#"
`timescale 1ns/1ps
module tb;
  reg clk = 0;
  always #5 clk = ~clk;
  reg [7:0] a1 = 0, a2 = 0, beat = 0;
  task automatic pulse(input [7:0] n);
    @(posedge clk);
    a2 = a2 + n;
  endtask
  task pulse0;
    @(posedge clk);
    a1 = a1 + 1;
  endtask
  always @(posedge clk) pulse0();
  always @(posedge clk) pulse(2);
  initial begin
    #7;
    forever begin
      #10 beat <= beat + 3;
    end
  end
  initial begin
    #63 $display("U a1=%0d a2=%0d beat=%0d", a1, a2, beat);
    $finish;
  end
endmodule
"#;
    let out = run_default(src, "busy_edges");
    // Reference: each call consumes the NEXT edge inside the task, so the
    // wrapping process re-arms only every other edge (a1 counts half the
    // edges). The legacy edge path either overlapped bodies or lost the
    // callee's statements entirely (a1 stayed 0).
    assert!(
        out.contains("U a1=3 a2=6 beat=15"),
        "call-blocking posedge bodies must follow process semantics: {}",
        out
    );
}
