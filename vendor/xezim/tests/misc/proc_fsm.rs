//! Roadmap steps 11-12 (opt-in XEZIM_PROC_FSM=1): blocking always bodies
//! compile into bytecode FSMs with wait insns; a resume re-enters at the
//! saved pc with per-process registers instead of re-walking the AST chain.
//!
//! The multi-wait expected values below are REFERENCE-VERIFIED: a body that
//! consumes time past the next clock edge MISSES that edge (§9.2.2 process
//! semantics — the process is not at its event control). xezim's legacy
//! edge path fires such bodies on every posedge instead; the FSM is the
//! conforming behavior, which is why the values differ from a plain run.

use std::process::Command;

fn run_fsm(src: &str, tag: &str) -> String {
    let dir = std::env::temp_dir().join(format!("xezim_proc_fsm_{}_{}", tag, std::process::id()));
    let _ = std::fs::create_dir_all(&dir);
    let f = dir.join("t.sv");
    std::fs::write(&f, src).unwrap();
    let out = Command::new(env!("CARGO_BIN_EXE_xezim"))
        .args(["--no-cache", "-s", "tb", "--max-time", "1000"])
        .arg(&f)
        .env("XEZIM_PROC_FSM", "1")
        .env("XEZIM_PROC_LOOP_STATS", "1")
        .output()
        .expect("run xezim");
    format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    )
}

#[test]
fn fsm_multiwait_bodies_match_reference() {
    let src = r#"
`timescale 1ns/1ps
module tb;
  reg clk = 0;
  reg [7:0] cnt = 0, mirror = 0, tick = 0;
  always begin
    #5 clk = ~clk;
  end
  always @(posedge clk) begin
    cnt = cnt + 1;
    #2;
    mirror = cnt ^ 8'h0f;
    repeat (2) @(negedge clk);
    mirror = mirror + 1;
  end
  always begin
    @(posedge clk);
    tick <= tick + 3;
    #1 tick <= tick + 1;
  end
  initial begin
    #103 $display("R cnt=%0d mirror=%0d tick=%0d clk=%b", cnt, mirror, tick, clk);
    #40 $display("R2 cnt=%0d mirror=%0d tick=%0d", cnt, mirror, tick);
    $finish;
  end
endmodule
"#;
    let text = run_fsm(src, "multiwait");
    assert!(
        text.contains("[PROC-FSM] registered"),
        "FSM must engage:\n{}",
        text
    );
    // Reference-simulator values (edge missed while the body is mid-flight).
    assert!(
        text.contains("R cnt=5 mirror=11 tick=40 clk=0"),
        "multi-wait always semantics:\n{}",
        text
    );
    assert!(
        text.contains("R2 cnt=7 mirror=9 tick=56"),
        "later window:\n{}",
        text
    );
}

#[test]
fn fsm_blocking_task_calls_match_reference() {
    // Step 12: blocking task bodies (waits inside) inline into the FSM —
    // formals live in frame registers across suspensions. Expected values
    // are reference-verified; the legacy edge path loses these bodies
    // entirely (a1=0) or over-fires them.
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
    let text = run_fsm(src, "taskcall");
    assert!(
        text.contains("U a1=3 a2=6 beat=15"),
        "task-call FSM must match the reference:\n{}",
        text
    );
}

#[test]
fn fsm_task_with_repeat_and_delay_matches_reference() {
    let src = r#"
`timescale 1ns/1ps
module tb;
  reg clk = 0;
  always #5 clk = ~clk;
  reg [7:0] acc = 0, phase = 0, beat = 0;
  task automatic pulse(input [7:0] n);
    repeat (2) @(posedge clk);
    acc = acc + n;
    #3;
    acc = acc ^ 8'h21;
  endtask
  always @(posedge clk) begin
    phase <= phase + 1;
    pulse(phase);
  end
  initial begin
    #7;
    forever begin
      #10 beat <= beat + 3;
    end
  end
  initial begin
    #103 $display("S1 acc=%0d phase=%0d beat=%0d", acc, phase, beat);
    #50  $display("S2 acc=%0d phase=%0d beat=%0d", acc, phase, beat);
    $finish;
  end
endmodule
"#;
    let text = run_fsm(src, "taskrpt");
    assert!(
        text.contains("S1 acc=36 phase=4 beat=27"),
        "reference S1:\n{}",
        text
    );
    assert!(
        text.contains("S2 acc=43 phase=5 beat=42"),
        "reference S2:\n{}",
        text
    );
}

#[test]
fn fsm_native_aot_matches_reference() {
    // Step 13: the same multi-wait body through the rustc-native FSM path
    // (XEZIM_AOT=1) — enrollment must happen and values must stay
    // reference-exact. Skips silently when rustc is unavailable (the AOT
    // library fails to build and the bytecode executor serves the FSM with
    // identical values, so the value assertions still hold).
    let src = r#"
`timescale 1ns/1ps
module tb;
  reg clk = 0;
  reg [7:0] cnt = 0, mirror = 0, tick = 0;
  always begin
    #5 clk = ~clk;
  end
  always @(posedge clk) begin
    cnt = cnt + 1;
    #2;
    mirror = cnt ^ 8'h0f;
    repeat (2) @(negedge clk);
    mirror = mirror + 1;
  end
  always begin
    @(posedge clk);
    tick <= tick + 3;
    #1 tick <= tick + 1;
  end
  initial begin
    #103 $display("R cnt=%0d mirror=%0d tick=%0d clk=%b", cnt, mirror, tick, clk);
    #40 $display("R2 cnt=%0d mirror=%0d tick=%0d", cnt, mirror, tick);
    $finish;
  end
endmodule
"#;
    let dir = std::env::temp_dir().join(format!("xezim_proc_fsm_native_{}", std::process::id()));
    let _ = std::fs::create_dir_all(&dir);
    let f = dir.join("t.sv");
    std::fs::write(&f, src).unwrap();
    let out = Command::new(env!("CARGO_BIN_EXE_xezim"))
        .args(["--no-cache", "-s", "tb", "--max-time", "1000"])
        .arg(&f)
        .env("XEZIM_PROC_FSM", "1")
        .env("XEZIM_JIT", "1")
        .env("XEZIM_AOT", "1")
        .env("XEZIM_JIT_VERBOSE", "1")
        .output()
        .expect("run xezim");
    let text = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(
        text.contains("R cnt=5 mirror=11 tick=40 clk=0"),
        "native FSM values:\n{}",
        text
    );
    assert!(
        text.contains("R2 cnt=7 mirror=9 tick=56"),
        "native FSM later window:\n{}",
        text
    );
}

/// A delay-bearing monitor that reads `<top>.<path>` (a testbench macro such
/// as `tb.dut.core.retire`) compiles into a process FSM by default, as one
/// reading the unprefixed path does. The prefixed read used to fall back,
/// which kept the whole body on the AST process path for every clock.
#[test]
fn fsm_monitor_reads_top_prefixed_path() {
    let src = r#"
module leaf(input clk, output o);
  reg r = 0;
  always @(posedge clk) r <= ~r;
  assign o = r;
endmodule
module mid(input clk);
  wire w;
  leaf u2(.clk(clk), .o(w));
endmodule
module tb;
  reg clk = 0;
  always #5 clk = ~clk;
  mid u1(.clk(clk));
  integer n = 0;
  always @(posedge clk) begin
    if (tb.u1.u2.o) n <= n + 1;
    if (n == 3) begin
      #1;
      $display("n=%0d t=%0t", n, $time);
      $finish;
    end
  end
endmodule
"#;
    let dir = std::env::temp_dir().join(format!("xezim_proc_fsm_top_{}", std::process::id()));
    let _ = std::fs::create_dir_all(&dir);
    let f = dir.join("t.sv");
    std::fs::write(&f, src).unwrap();
    let out = Command::new(env!("CARGO_BIN_EXE_xezim"))
        .args(["--no-cache", "-s", "tb", "--max-time", "1000"])
        .arg(&f)
        .env("XEZIM_PROC_LOOP_STATS", "1")
        .output()
        .expect("run xezim");
    let text = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(
        text.contains("[PROC-FSM] registered"),
        "monitor not compiled:\n{}",
        text
    );
    assert!(!text.contains("gated out"), "monitor gated out:\n{}", text);
    assert!(text.contains("n=3 t=66"), "monitor result:\n{}", text);
}

/// A run that stops at --max-time with a compiled process parked still says
/// where that process is: its waiter has no AST continuation to point at.
#[test]
fn hang_report_names_parked_fsm_process() {
    let src = r#"
module tb;
  reg clk = 0;
  always #5 clk = ~clk;
  integer n = 0;
  always @(posedge clk) begin
    n <= n + 1;
    if (n == 1000) begin
      #1;
      $display("never");
    end
  end
endmodule
"#;
    let dir = std::env::temp_dir().join(format!("xezim_proc_fsm_hang_{}", std::process::id()));
    let _ = std::fs::create_dir_all(&dir);
    let f = dir.join("t.sv");
    std::fs::write(&f, src).unwrap();
    let out = Command::new(env!("CARGO_BIN_EXE_xezim"))
        .args(["--no-cache", "-s", "tb", "--max-time", "40"])
        .arg(&f)
        .env("XEZIM_PROC_LOOP_STATS", "1")
        .output()
        .expect("run xezim");
    let text = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(
        text.contains("[PROC-FSM] registered"),
        "process not compiled:\n{}",
        text
    );
    assert!(
        text.contains("resumes at a wait inside the process at ") && text.contains("t.sv:6"),
        "hang report location:\n{}",
        text
    );
}
