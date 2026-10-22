//! Task enables written without parentheses through a hierarchical path or a
//! package scope (§13.5): `za.delay;`, `ai.bi.tick;`, `pk::runner;`.
//!
//! These used to stay bare reads. Inside a task they were dropped, so an
//! `always` calling that task looped at time 0 and the run hung; directly in
//! an `always` the no-timing-control lint rejected the design. A three-level
//! enable inside a task (`ai.bi.tick` parses as `(ai.bi).tick` there) was
//! never inlined either, so it ran synchronously and kept advancing time
//! after `$finish`. The reference simulator prints TEST_PASS and stops at 31.

use std::process::Command;

const SRC: &str = r#"
`ifndef SVTEST_DEFS_SVH
`define SVTEST_DEFS_SVH

`define SVTEST_INIT \
int failures = 0;

`define SVTEST_CHECK(expr, msg) \
if (!(expr)) begin \
  failures++; \
  $display("FAIL @%0t : %s", $time, msg); \
end

`define SVTEST_PASSFAIL \
if (failures == 0) begin \
  $display("TEST_PASS"); \
end else begin \
  $display("TEST_FAIL count=%0d", failures); \
  $fatal(1); \
end

`endif

package pk;
  task runner;          // package-qualified parenless enable target
    #2;
  endtask
endpackage

module zcell;           // plain hierarchical task targets
  task delay;           // leaf name collides with arm A's wrapper (on purpose)
    #2;
  endtask
  task tick;            // clean leaf name for arm G
    #2;
  endtask
endmodule

module wcell (input logic clk);  // event-control task target
  task waiter;
    @(posedge clk);
  endtask
endmodule

module deep;            // 3-segment path target: top.ai.bi.tick
  task tick;
    #2;
  endtask
endmodule

module mid;
  deep bi();
endmodule

module top;
  logic clk = 0;
  int    pulses    = 0;  // clock posedges observed (arm B evidence)
  int    cnt_a     = 0;  // arm A completed wrapper iterations
  int    cnt_g     = 0;  // arm G completed iterations
  int    cnt_c     = 0;  // arm C completed iterations
  int    cnt_d     = 0;  // arm D completed iterations
  int    oneshot_t  = -1;    // control: time at which the one-shot returned

  zcell za();           // arm A target (delay)
  zcell zg();           // arm G target (tick)
  wcell zi(clk);        // arm B target (waiter)
  mid   ai();           // arm C target (ai.bi.tick)

  import pk::*;         // for the bare-imported control arm

  task delay;
    za.delay;
    cnt_a = cnt_a + 1;
  endtask
  always begin
    delay;
  end

  task call_tick;
    zg.tick;
    cnt_g = cnt_g + 1;
  endtask
  always begin
    call_tick;
  end

  task call_deep;
    ai.bi.tick;
    cnt_c = cnt_c + 1;
  endtask
  always begin
    call_deep;
  end

  task call_pkg;
    pk::runner;
    cnt_d = cnt_d + 1;
  endtask
  always begin
    call_pkg;
  end

  task call_wait;
    zi.waiter;
  endtask
  always begin
    call_wait;
  end

  always begin
    za.delay;
  end

  always begin
    pk::runner;
  end

  initial begin
    za.delay;
    oneshot_t = $time;
    $display("CTRL_one_shot_hier_ok t=%0t", $realtime);
  end

  always begin
    za.delay();
  end

  always begin
    runner;
  end

  task inner_local;
    #2;
  endtask
  task outer_local;
    inner_local;
  endtask
  always begin
    outer_local;
  end

  task call_wait_p;
    zi.waiter();
  endtask
  always begin
    call_wait_p;
  end

  always #5 clk = ~clk;
  always @(posedge clk) pulses = pulses + 1;

  initial begin
    `SVTEST_INIT
    #21;
    $display("ARM_A_hier_wrapper_delay_ok t=%0t cnt=%0d", $realtime, cnt_a);
    `SVTEST_CHECK($time == 21, "arm_A_scheduler_not_starved")
    `SVTEST_CHECK(cnt_a == 10, "arm_A_wrapper_completed_10_iterations")
    $display("ARM_G_clean_two_segment_ok t=%0t cnt=%0d", $realtime, cnt_g);
    `SVTEST_CHECK(cnt_g == 10, "arm_G_completed_10_iterations")
    $display("ARM_C_three_segment_ok t=%0t cnt=%0d", $realtime, cnt_c);
    `SVTEST_CHECK(cnt_c == 10, "arm_C_completed_10_iterations")
    $display("ARM_D_package_qualified_ok t=%0t cnt=%0d", $realtime, cnt_d);
    `SVTEST_CHECK(cnt_d == 10, "arm_D_completed_10_iterations")
    `SVTEST_CHECK(oneshot_t == 2, "control_one_shot_returned_at_2ns")
    #10;                       // t = 31
    $display("ARM_B_event_control_ok t=%0t pulses=%0d", $realtime, pulses);
    `SVTEST_CHECK($time == 31, "arm_B_time_advanced_to_31ns")
    `SVTEST_CHECK(pulses == 3, "arm_B_waiter_suspended_on_clock_edges")
    `SVTEST_PASSFAIL
    $finish;
  end
endmodule
"#;

#[test]
fn parenless_hierarchical_and_package_task_enables() {
    let dir = std::env::temp_dir().join(format!("xezim_parenless_{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let file = dir.join("t.sv");
    std::fs::write(&file, SRC).unwrap();
    let out = Command::new(env!("CARGO_BIN_EXE_xezim"))
        .args(["--no-cache", "-s", "top"])
        .arg(&file)
        .output()
        .expect("failed to run xezim");
    let text = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    let _ = std::fs::remove_dir_all(&dir);
    assert!(out.status.success(), "{text}");
    for line in [
        "CTRL_one_shot_hier_ok t=2",
        "ARM_A_hier_wrapper_delay_ok t=21 cnt=10",
        "ARM_G_clean_two_segment_ok t=21 cnt=10",
        "ARM_C_three_segment_ok t=21 cnt=10",
        "ARM_D_package_qualified_ok t=21 cnt=10",
        "ARM_B_event_control_ok t=31 pulses=3",
        "TEST_PASS",
        "Simulation finished at time 31",
    ] {
        assert!(text.contains(line), "missing `{line}`:\n{text}");
    }
}
