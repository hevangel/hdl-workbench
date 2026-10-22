//! `force u.internal = v` on a signal INSIDE an instance parses as a
//! MemberAccess, not a dotted Ident, and `force_target` matched only the
//! Ident and array-element shapes. Hierarchical forces therefore resolved to
//! no target at all and degraded to a plain write: the value appeared for an
//! instant and the target's own driver overwrote it on its next evaluation
//! (continuous assign) or its next clock edge (NBA). A force onto a signal
//! feeding an output port never became visible at all.
//!
//! §10.6.2 requires the override to supersede all drivers until `release`.
//! Reference-verified line-for-line, including the post-release resumption.

use std::process::Command;

fn run(src: &str) -> String {
    // Unique per call: these tests run as parallel threads of ONE process,
    // so a pid-only directory name lets them clobber each other's source.
    static SEQ: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);
    let dir = std::env::temp_dir().join(format!(
        "xezim_hierforce_{}_{}",
        std::process::id(),
        SEQ.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
    ));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("tb.sv");
    std::fs::write(&path, src).unwrap();
    let out = Command::new(env!("CARGO_BIN_EXE_xezim"))
        .args([
            "--simulate",
            "-s",
            "tb_shape",
            path.to_str().unwrap(),
            "--no-cache",
        ])
        .output()
        .expect("run xezim");
    let mut text = String::from_utf8_lossy(&out.stdout).to_string();
    text.push_str(&String::from_utf8_lossy(&out.stderr));
    let _ = std::fs::remove_dir_all(&dir);
    text
}

#[test]
fn hierarchical_force_survives_driver_reevaluation() {
    let text = run(
        r#"module dut (input clk, input [7:0] a, output [7:0] oport);
  wire [7:0] internal;
  reg  [7:0] flop;
  assign internal = a + 8'd1;
  assign oport    = a + 8'd2;
  always @(posedge clk) flop <= a + 8'd3;
  wire [7:0] probe_int = internal;
  wire [7:0] probe_flp = flop;
endmodule
module tb_shape;
  reg clk = 0; always #5 clk = ~clk;
  reg [7:0] a = 8'd10;
  wire [7:0] oport;
  dut u (.clk(clk), .a(a), .oport(oport));
  initial begin
    #12;
    force u.internal = 8'hAA;
    force u.oport    = 8'hBB;
    force u.flop     = 8'hCC;
    #0;
    $display("T1 int=%02x oport=%02x flop=%02x", u.internal, oport, u.flop);
    a = 8'd20;                     // re-drives both continuous assigns
    #1 $display("T2 int=%02x oport=%02x flop=%02x", u.internal, oport, u.flop);
    @(posedge clk);                // re-drives the clocked block
    #1 $display("T3 int=%02x oport=%02x flop=%02x  probes int=%02x flp=%02x",
                u.internal, oport, u.flop, u.probe_int, u.probe_flp);
    release u.internal; release u.oport; release u.flop;
    #10 $display("T4 int=%02x oport=%02x flop=%02x", u.internal, oport, u.flop);
    $finish;
  end
endmodule
"#,
    );
    for expect in [
        "T1 int=aa oport=bb flop=cc",
        "T2 int=aa oport=bb flop=cc",
        "T3 int=aa oport=bb flop=cc  probes int=aa flp=cc",
        "T4 int=15 oport=16 flop=17",
    ] {
        assert!(text.contains(expect), "missing `{expect}` in:\n{text}");
    }
}

#[test]
fn hierarchical_force_reaches_generate_and_array_targets() {
    // A force through a generate-block index (`G[0].u_gen.q`) and one onto a
    // fixed-array element, both re-driven every clock. Reference-verified.
    let text = run(
        r#"module blk (input clk, input [31:0] din, output reg [31:0] q);
  initial q = 0;
  always @(posedge clk) q <= (din ^ 32'h5A5A_1234) + 32'd7;
endmodule
module tb_shape;
  reg clk = 0; always #5 clk = ~clk;
  reg [31:0] lfsr = 32'hACE1_1234;
  wire [31:0] qg;
  reg [7:0] mem [4];
  integer k;
  initial for (k = 0; k < 4; k = k + 1) mem[k] = 0;
  genvar g;
  generate for (g = 0; g < 1; g = g + 1) begin : G
    blk u_gen (.clk(clk), .din(lfsr), .q(qg));
  end endgenerate
  always @(posedge clk) begin
    lfsr   <= {lfsr[30:0], lfsr[31] ^ lfsr[21] ^ lfsr[1] ^ lfsr[0]};
    mem[1] <= lfsr[7:0];
  end
  initial begin
    #103; force G[0].u_gen.q = 32'hFEED_FACE;
          force mem[1]       = 8'hC3;
    #100; $display("F qg=%08x mem1=%02x", qg, mem[1]);
          $finish;
  end
endmodule
"#,
    );
    assert!(
        text.contains("F qg=feedface mem1=c3"),
        "generate-indexed or array-element force did not hold:\n{text}"
    );
}
