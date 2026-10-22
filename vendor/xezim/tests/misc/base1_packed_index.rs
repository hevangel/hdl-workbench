//! §7.4.1: a DYNAMIC index into a packed array whose outer dimension has a
//! non-zero LSB (`logic [4:1][4:0] src; ... src[i]`) must normalize the
//! index against the declared range. The compiled read path used
//! `idx * elem_w` directly, so slice i read element i+1 and the top element
//! read out of range — injecting X. In a lane-expander DUT this turned every
//! per-lane status flop X on the first multi-lane advance ($isunknown
//! checker caught it; the reference passes). Constant indices and the write
//! path were already correct.

use std::process::Command;

fn run(name: &str, top: &str, src_path: Option<&str>, src_inline: Option<&str>) -> String {
    let path = if let Some(p) = src_path {
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(p)
    } else {
        let dir = std::env::temp_dir().join(format!("xezim_b1pi_{}_{}", name, std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let p = dir.join(format!("{name}.sv"));
        std::fs::write(&p, src_inline.unwrap()).unwrap();
        p
    };
    let out = Command::new(env!("CARGO_BIN_EXE_xezim"))
        .args([
            "--simulate",
            "-s",
            top,
            path.to_str().unwrap(),
            "--no-cache",
            "--max-time",
            "10000",
        ])
        .output()
        .expect("run xezim");
    format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    )
}

#[test]
fn base1_outer_dim_dynamic_read() {
    let text = run(
        "micro",
        "tb",
        None,
        Some(
            r#"module tb;
  logic clk = 0; always #5 clk = ~clk;
  logic [4:1][4:0] src;
  logic [3:0] en;
  logic [4:0] out;
  always_ff @(posedge clk) begin
    for (int i = 1; i <= 4; i++) begin
      if (en[i-1]) out <= src[i];
    end
  end
  initial begin
    src[1] = 5'h01; src[2] = 5'h02; src[3] = 5'h03; src[4] = 5'h04;
    en = 4'b0001; @(posedge clk); #1 $display("T|i1 out=%h", out);
    en = 4'b0010; @(posedge clk); #1 $display("T|i2 out=%h", out);
    en = 4'b0100; @(posedge clk); #1 $display("T|i3 out=%h", out);
    en = 4'b1000; @(posedge clk); #1 $display("T|i4 out=%h", out);
    $finish;
  end
endmodule
"#,
        ),
    );
    for want in ["T|i1 out=01", "T|i2 out=02", "T|i3 out=03", "T|i4 out=04"] {
        assert!(text.contains(want), "{text}");
    }
}

#[test]
fn lane_expander_lfsr_no_x_leak() {
    // The full self-checking lane-expander TB (renamed): pipe+skid input
    // stage, base-1 packed status vectors, LFSR stimulus with $isunknown
    // checks every cycle. Reference-verified ALL-PASS.
    let text = run(
        "lanex",
        "tb_lane_exp",
        Some("tests/misc/svtb/lane_expander.sv"),
        None,
    );
    assert!(
        text.contains(">>> ALL TESTS PASSED SUCCESSFULLY <<<"),
        "{text}"
    );
    assert!(!text.contains("SVCHECK FAILED"), "{text}");
}
