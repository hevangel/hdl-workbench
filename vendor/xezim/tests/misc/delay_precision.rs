//! §3.14.3: a delay quantizes to its DECLARING scope's time precision.
//! Constant delays now fold-and-quantize at ELABORATION (where the declaring
//! module is unambiguous), so every execution path — hierarchical task call,
//! interface task, class method, intra-assignment form — agrees. The runtime
//! scope-resolved quantization used to miss on some task-call paths: a
//! `#0.002` inside a 1ns/1ns BFM called from a 1ps-precision testbench kept
//! a real 2 ps duration, and one such call phase-shifted the BFM's
//! delay-paced loops 6 ps off the clock grid for the rest of the run (the
//! NBA-vs-reference skew a VCD comparison flagged at t=4,000,606 vs
//! 4,000,600).

use std::process::Command;

fn run(src: &str, tag: &str) -> String {
    let dir = std::env::temp_dir().join(format!("xezim_delay_prec_{}_{}", tag, std::process::id()));
    let _ = std::fs::create_dir_all(&dir);
    let f = dir.join("t.sv");
    std::fs::write(&f, src).unwrap();
    let out = Command::new(env!("CARGO_BIN_EXE_xezim"))
        .args(["--no-cache", "-s", "tb", "--max-time", "1000"])
        .arg(&f)
        .output()
        .expect("run xezim");
    String::from_utf8_lossy(&out.stdout).to_string()
}

#[test]
fn sub_precision_delays_round_to_zero_in_coarse_scopes() {
    let src = r#"
`timescale 1ns/1ns
interface coarse_if(input logic clk);
  task tiny_if_delay;
    #0.002;
  endtask
endinterface

`timescale 1ns/1ns
package coarse_pkg;
  class CoarseC;
    task tiny_meth_delay;
      #0.002;
    endtask
  endclass
endpackage

`timescale 1ns/1ns
module coarse_bfm(input logic clk);
  logic [7:0] cyc = 0;
  always @(posedge clk) begin
    cyc <= cyc + 1;
    #0.002;
  end
  task tiny_task_delay;
    #0.002;
  endtask
  logic [7:0] beat = 0;
  initial begin
    #0.6;
    forever begin
      beat = beat + 1;
      #0.002;
      #1.0;
    end
  end
endmodule

`timescale 1ns/1ps
module tb;
  import coarse_pkg::*;
  logic clk = 0;
  always #0.5 clk = ~clk;
  coarse_bfm u_b(.clk(clk));
  coarse_if u_if(.clk(clk));
  CoarseC obj = new();
  initial begin
    #10.25;
    $display("BEAT t=%0t beat=%0d", $realtime, u_b.beat);
    u_b.tiny_task_delay;
    $display("HTASK t=%0t", $realtime);
    u_if.tiny_if_delay;
    $display("IFTASK t=%0t", $realtime);
    obj.tiny_meth_delay;
    $display("METH t=%0t", $realtime);
    $finish;
  end
endmodule
"#;
    let text = run(src, "coarse");
    // A delay-paced loop with a rounds-to-zero mid-body delay stays ON the
    // 1ns grid: 10 beats by t=10.25ns. With the 2ps kept, the phase shifts
    // and the count/time drift (the cv8s BFM symptom).
    assert!(
        text.contains("BEAT t=10250 beat=10"),
        "delay-paced loop must stay on grid:\n{}",
        text
    );
    for l in ["HTASK t=10250", "IFTASK t=10250", "METH t=10250"] {
        assert!(
            text.contains(l),
            "{} — a #0.002 in a 1ns/1ns scope must consume ZERO time on \
             every call path:\n{}",
            l,
            text
        );
    }
}

#[test]
fn fine_precision_delays_are_kept() {
    // The same #0.002 in a 1ns/1ps module is a REAL 2ps delay — the
    // quantization must not flatten it.
    let src = r#"
`timescale 1ns/1ps
module tb;
  logic mark = 0;
  initial begin
    #0.002 mark = 1;
    $display("FINE t=%0t mark=%b", $realtime, mark);
    #0.0004;
    $display("FINE2 t=%0t", $realtime);
    $finish;
  end
endmodule
"#;
    let text = run(src, "fine");
    assert!(
        text.contains("FINE t=2 mark=1"),
        "2ps must survive:\n{}",
        text
    );
    // 0.4ps rounds to the 1ps precision grid -> 0.
    assert!(
        text.contains("FINE2 t=2"),
        "0.4ps rounds to zero:\n{}",
        text
    );
}
