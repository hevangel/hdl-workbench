//! The register-backed compiled path for `for (int i = ...)` loops in clocked
//! blocks was gated by a body filter that rejected `case` statements, nested
//! `for` loops, and any element update reading its own array (`cnt[b] <=
//! cnt[b] + 1`). Arbiter/datapath loops are exactly banks-by-entries nests
//! with a case and per-bank counters, so whole hot blocks fell back to AST
//! interpretation per iteration.
//!
//! The self-read exclusion existed for alias skew: an elaboration-rewritten
//! dotted lvalue paired with a bare-name read could resolve the array through
//! different aliases. Same-form self-reads resolve through the identical
//! lookup and cannot skew, so the audit now only rejects reads whose dotted
//! form differs from the lvalue's.
//!
//! Pins a two-instance arb-style bench (case + nested for + self-read
//! counters) against a reference-verified checksum, and asserts the compiled
//! path engages (no For_init_vardecl fallback reported).

use std::process::Command;

#[test]
fn clocked_loop_with_case_and_nested_for_compiles_and_matches() {
    let src = r#"module arb_dp #(parameter NB=4, NE=8) (
  input clk, input [3:0] gnt, input [31:0] din,
  output reg [31:0] dout
);
  reg [31:0] q [NB*NE-1:0];
  reg [7:0] cnt [NB-1:0];
  integer k;
  initial begin dout = 0; for (k=0;k<NB*NE;k=k+1) q[k]=0; for(k=0;k<NB;k=k+1) cnt[k]=0; end
  always @(posedge clk) begin
    for (int b = 0; b < NB; b = b + 1) begin
      case (gnt)
        4'd0: cnt[b] <= cnt[b] + 8'd1;
        4'd1: cnt[b] <= cnt[b] - 8'd1;
        default: begin
          for (int e = 0; e < NE; e = e + 1) begin
            if (cnt[b][2:0] == e[2:0])
              q[b*NE + e] <= din ^ {24'd0, cnt[b]};
          end
        end
      endcase
    end
    dout <= q[{28'd0, gnt} % (NB*NE)] ^ din;
  end
endmodule
module tb;
  reg clk = 0; always #5 clk = ~clk;
  reg [31:0] lfsr = 32'hABCD_0123;
  wire [31:0] d0, d1;
  arb_dp u_a0 (.clk(clk), .gnt(lfsr[3:0]), .din(lfsr), .dout(d0));
  arb_dp u_a1 (.clk(clk), .gnt(lfsr[7:4]), .din(~lfsr), .dout(d1));
  reg [31:0] csum = 0; integer cyc = 0;
  always @(posedge clk) begin
    lfsr <= {lfsr[30:0], lfsr[31] ^ lfsr[21] ^ lfsr[1] ^ lfsr[0]};
    if (cyc >= 4) csum <= csum ^ d0 ^ d1;
    cyc <= cyc + 1;
    if (cyc == 400) begin $display("CSUM=%08x", csum); $finish; end
  end
endmodule
"#;
    let dir = std::env::temp_dir().join(format!("xezim_loopcase_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("tb.sv");
    std::fs::write(&path, src).unwrap();
    let out = Command::new(env!("CARGO_BIN_EXE_xezim"))
        .args([
            "--simulate",
            "-s",
            "tb",
            path.to_str().unwrap(),
            "--no-cache",
        ])
        .env("XEZIM_PROFILE_TIMING", "1")
        .output()
        .expect("run xezim");
    let mut text = String::from_utf8_lossy(&out.stdout).to_string();
    text.push_str(&String::from_utf8_lossy(&out.stderr));

    // Reference-verified checksum for this stimulus.
    assert!(
        text.contains("CSUM=a654bb05"),
        "checksum mismatch or missing:\n{text}"
    );
    // The loops must take the compiled path, not per-iteration AST fallback.
    assert!(
        !text.contains("For_init_vardecl"),
        "clocked loop fell back to AST interpretation:\n{text}"
    );
    let _ = std::fs::remove_dir_all(&dir);
}
