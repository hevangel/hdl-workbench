//! §21.2.1.7: `%m` names the scope of the statement that CONTAINS it.
//!
//! `$monitor`/`$fmonitor` only ARM at their call site — the format string is
//! re-rendered at the end of every later time slot by `check_monitor`, after
//! the arming statement has returned. Nothing carried the arming scope across,
//! so `%m` resolved against whatever was executing at that point. For the
//! FIRST render — the slot-end print that follows arming — that is nothing at
//! all, and it degraded to the top module: a monitor armed inside an instance
//! reported the TOP while a `$display("%m")` on the line above reported the
//! instance.
//!
//! A BOUND instance is what makes this visible. With a plain instantiation the
//! lingering `current_scope` of the arming process happened to be right, so the
//! defect hid; the first print under `bind` did not.

use xezim::simulate;

fn notes(src: &str) -> Vec<String> {
    let sim = simulate(src, 1_000_000).expect("simulate failed");
    sim.output
        .iter()
        .map(|o| o.message.trim().to_string())
        .filter(|l| l.starts_with("NOTE:"))
        .collect()
}

const BOUND_PROBE: &str = r#"
`timescale 1ns/1ps
module testbench;
  logic clk = 0;
  always #5 clk = ~clk;
  initial #40 $finish;
endmodule
module probe;
  logic tick = 0;
  initial begin
    $display("NOTE: inline %m");
    $monitor("NOTE: monitor %m tick=%b", tick);
    #10 tick = 1;
  end
endmodule
bind testbench probe u_probe ();
"#;

/// EVERY monitor render names the arming instance — including the first, which
/// is the one that used to say `testbench`.
#[test]
fn monitor_percent_m_names_the_arming_instance_from_the_first_render() {
    let n = notes(BOUND_PROBE);
    assert_eq!(
        n.first().map(String::as_str),
        Some("NOTE: inline testbench.u_probe"),
        "inline %m must name the bound instance; got {:?}",
        n
    );
    let monitors: Vec<&String> = n
        .iter()
        .filter(|l| l.starts_with("NOTE: monitor "))
        .collect();
    assert!(
        monitors.len() >= 2,
        "expected repeated monitor renders, got {:?}",
        n
    );
    for line in &monitors {
        assert!(
            line.starts_with("NOTE: monitor testbench.u_probe "),
            "every monitor render must name the arming instance; got {:?}",
            monitors
        );
    }
}

/// `$fmonitor` shares the same slot and must carry the scope the same way —
/// this is the shape the original report used (capture `%m` to a file and
/// compare it against `$sformatf("%m")`).
#[test]
fn fmonitor_percent_m_matches_sformatf_percent_m() {
    let src = r#"
`timescale 1ns/1ps
module testbench;
  logic clk = 0;
  always #5 clk = ~clk;
  initial #40 $finish;
endmodule
module probe;
  string armed_scope;
  string seen_scope;
  int fd, status;
  initial begin
    armed_scope = $sformatf("%m");
    fd = $fopen("xezim_monitor_scope_probe.log", "w");
    $fmonitor(fd, "%m");
    #10;
    $fclose(fd);
    fd = $fopen("xezim_monitor_scope_probe.log", "r");
    status = $fscanf(fd, "%s", seen_scope);
    $fclose(fd);
    $display("NOTE: armed=%s seen=%s", armed_scope, seen_scope);
  end
endmodule
bind testbench probe u_probe ();
"#;
    let n = notes(src);
    assert_eq!(
        n,
        vec!["NOTE: armed=testbench.u_probe seen=testbench.u_probe"],
        "$fmonitor's %m must agree with $sformatf(\"%m\") in the same scope"
    );
    let _ = std::fs::remove_file("xezim_monitor_scope_probe.log");
}

/// A monitor armed at the top keeps naming the top — the fix must not make
/// every `%m` report an instance.
#[test]
fn monitor_percent_m_at_top_still_names_top() {
    let src = r#"
`timescale 1ns/1ps
module top;
  logic tick = 0;
  initial begin
    $monitor("NOTE: monitor %m tick=%b", tick);
    #10 tick = 1;
    #10 $finish;
  end
endmodule
"#;
    let n = notes(src);
    assert!(
        n.iter().all(|l| l.starts_with("NOTE: monitor top ")),
        "a top-level monitor must name the top, got {:?}",
        n
    );
}

/// §21.2.1.7 again, one level deeper: `m_scope_stack` — the task / function /
/// named-block chain that follows the instance path — was a single GLOBAL
/// stack. A task that suspends at a `#delay` leaves its own name installed,
/// so the next process to run inherited it.
///
/// Here `clkgen`'s `drive()` suspends at `#100` before `probe`'s initial block
/// arms its monitor. Every `%m` then read `testbench.u_tb_binder.drive` — an
/// instance path glued to an unrelated module's task. Under `bind`, while
/// chasing a timestamp discrepancy, that names a scope that does not exist.
const SUSPENDED_TASK_LEAK: &str = r#"
`timescale 1ns/1ps
module clkgen(output logic clk, output logic rst);
  task drive();
    clk = 1'bx; rst = 1'bx;
    #100 rst = 0; clk = 0;
    #500 clk = 1;
  endtask
  initial drive();
endmodule
module probe(input logic clk, input logic rst);
  initial $monitor("NOTE: %m clk=%b rst=%b", clk, rst);
endmodule
module testbench;
  logic clk, rst;
  clkgen u_clkgen(.clk(clk), .rst(rst));
  initial #700 $finish;
endmodule
bind testbench probe u_tb_binder(.clk(testbench.clk), .rst(testbench.rst));
"#;

#[test]
fn monitor_m_ignores_unrelated_suspended_task() {
    let got = notes(SUSPENDED_TASK_LEAK);
    assert_eq!(
        got,
        vec![
            "NOTE: testbench.u_tb_binder clk=x rst=x",
            "NOTE: testbench.u_tb_binder clk=0 rst=0",
            "NOTE: testbench.u_tb_binder clk=1 rst=0",
        ],
        "%m must name the arming instance only; `.drive` belongs to a \
         suspended task in a different module"
    );
}
