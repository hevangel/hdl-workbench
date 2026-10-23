//! A per-statement bytecode fallback inside a loop whose counter lives in a
//! VM REGISTER read the loop var as a (non-existent) SIGNAL: `wr[i] = s.vld`
//! inside `for (int i; …)` — the struct-member RHS isn't bytecode-compilable,
//! `compile_stmt`'s rollback branch emitted a StmtFallback without the
//! reg_var_loop_depth guard that `emit_fallback` honors, and the interpreted
//! statement indexed with x and wrote NOTHING, every iteration, while the
//! entry reported success. A multi-queue DUT built on that shape (comb write
//! enables computed from a request struct) missed every request: levels
//! froze at 0, then underflows drove x into full/ready flags.
//!
//! Pins: (1) the minimal shape, (2) a self-checking multi-slot queue vs an
//! in-TB reference model under deterministic LFSR stimulus + a directed fill
//! burst. All expected values reference-verified (TEST_PASS there too).

use std::process::Command;

fn run(name: &str, src: &str) -> String {
    let dir = std::env::temp_dir().join(format!("xezim_crlf_{}_{}", name, std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join(format!("{name}.sv"));
    std::fs::write(&path, src).unwrap();
    let out = Command::new(env!("CARGO_BIN_EXE_xezim"))
        .args([
            "--simulate",
            "-s",
            "tb",
            path.to_str().unwrap(),
            "--no-cache",
        ])
        .output()
        .expect("run xezim");
    let mut text = String::from_utf8_lossy(&out.stdout).to_string();
    text.push_str(&String::from_utf8_lossy(&out.stderr));
    text
}

#[test]
fn regvar_loop_member_read_writes_land() {
    let text = run(
        "minimal",
        r#"module tb;
  typedef struct packed { logic vld; logic [3:0] slot_id; } req_t;
  logic clk = 0, rst = 1;
  req_t req;
  logic [3:0] level [16];
  logic wr [16];
  always #5 clk = ~clk;
  always_comb begin
    for (int i = 0; i < 16; i++) wr[i] = req.vld && (req.slot_id == i);
  end
  always_ff @(posedge clk) begin
    if (rst) begin
      for (int i = 0; i < 16; i++) level[i] <= '0;
    end else begin
      for (int i = 0; i < 16; i++) if (wr[i]) level[i] <= level[i] + 1;
    end
  end
  initial begin
    req = '0;
    repeat (3) @(posedge clk);
    rst = 0;
    @(negedge clk);
    req.vld <= 1'b1; req.slot_id <= 4'd11;
    @(posedge clk);
    #1 $display("T|t=%0t level11=%0d wr11=%b", $time, level[11], wr[11]);
    @(negedge clk);
    req.vld <= 1'b0;
    @(posedge clk);
    #1 $display("T|t=%0t level11=%0d", $time, level[11]);
    $finish;
  end
endmodule
"#,
    );
    assert!(text.contains("T|t=36 level11=1 wr11=1"), "{text}");
    assert!(text.contains("T|t=46 level11=1"), "{text}");
}

#[test]
fn multi_slot_queue_lfsr_regression_passes() {
    let text = run("slotq", include_str!("comb_regvar_loop_fallback_dut.sv"));
    assert!(text.contains("TEST_PASS"), "{text}");
    assert!(!text.contains("FAIL:"), "{text}");
}
