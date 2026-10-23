//! Two-state islands (P5) were inert on real designs for two reasons this
//! pins:
//!
//! 1. ANY active `force` disabled every island in the design. The §9.3.1
//!    filter is per DESTINATION, so a block is only unsafe when it writes a
//!    forced signal; a force elsewhere is irrelevant. A single `force` — the
//!    norm in a real bench — was collapsing island coverage to zero.
//! 2. `clean_const` rejected every SIGNED constant, and bare decimal
//!    literals are signed (§5.7.1), so `x + 1` / `x >> 3` / `x == 5` kept
//!    their blocks out. Constant-add and logical shifts had no lowered form
//!    at all, the former dismissed as "rare in comb cones" before the edge
//!    hook made counters the common case.
//!
//! Both fixes must not change results: each case is checked against a
//! reference-verified checksum AND against the same run with islands off.

use std::process::Command;

fn run(src: &str, two_state: bool) -> String {
    // Unique per call: these tests run as parallel threads of ONE process,
    // so a pid-only directory name lets them clobber each other's source.
    static SEQ: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);
    let dir = std::env::temp_dir().join(format!(
        "xezim_tscov_{}_{}_{}",
        u8::from(two_state),
        std::process::id(),
        SEQ.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
    ));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("tb.sv");
    std::fs::write(&path, src).unwrap();
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_xezim"));
    cmd.args([
        "--simulate",
        "-s",
        "tb_shape",
        path.to_str().unwrap(),
        "--no-cache",
    ])
    .env("XEZIM_PROFILE_TIMING", "1");
    // These tests assert island-engagement STATISTICS; an ambient
    // XEZIM_JIT=1 would route the same blocks to the native backend
    // (checksums stay right, the island counters read zero).
    cmd.env_remove("XEZIM_JIT");
    cmd.env_remove("XEZIM_AOT");
    if !two_state {
        cmd.env("XEZIM_TWO_STATE", "0");
    }
    let out = cmd.output().expect("run xezim");
    let mut text = String::from_utf8_lossy(&out.stdout).to_string();
    text.push_str(&String::from_utf8_lossy(&out.stderr));
    let _ = std::fs::remove_dir_all(&dir);
    text
}

fn evals(text: &str) -> u64 {
    text.split("two_state_evals=")
        .nth(1)
        .and_then(|t| t.split_whitespace().next())
        .and_then(|n| n.parse().ok())
        .unwrap_or(0)
}

/// A body built from bare (signed) literals, a constant add and both logical
/// shifts — the shapes that previously blocked lowering outright.
const SHIFT_ADD_DUT: &str = r#"module blk (input clk, input [31:0] din, output reg [31:0] q, output reg [31:0] r);
  initial begin q = 0; r = 0; end
  always @(posedge clk) begin
    q <= ((din ^ 32'h5A5A_1234) + 7) ^ ((din + 11) & 32'h00FF_FF00)
         ^ ((din >> 3) + 5) ^ ((din << 2) & 32'hFFF0_0000);
    r <= ((din ^ 32'h1234_5A5A) + 3) ^ ((din + 17) & 32'h0F0F_0F0F);
  end
endmodule
module tb_shape;
  reg clk = 0; always #5 clk = ~clk;
  reg [31:0] lfsr = 32'hACE1_1234;
  wire [31:0] qa, ra, qb, rb;
  blk u_a (.clk(clk), .din(lfsr),         .q(qa), .r(ra));
  blk u_b (.clk(clk), .din(lfsr ^ 32'd9), .q(qb), .r(rb));
  reg [31:0] csum = 0;
  integer cyc = 0;
  always @(posedge clk) begin
    lfsr <= {lfsr[30:0], lfsr[31] ^ lfsr[21] ^ lfsr[1] ^ lfsr[0]};
    csum <= csum ^ qa ^ ra ^ qb ^ rb;
    cyc  <= cyc + 1;
    if (cyc == 300) begin $display("CSUM=%08x qa=%08x", csum, qa); $finish; end
  end
endmodule
"#;

#[test]
fn signed_literals_shifts_and_const_add_lower() {
    let on = run(SHIFT_ADD_DUT, true);
    let off = run(SHIFT_ADD_DUT, false);
    // Reference-verified for this stimulus.
    assert!(on.contains("CSUM=00007dfc"), "checksum changed:\n{on}");
    assert!(
        off.contains("CSUM=00007dfc"),
        "4-state checksum changed:\n{off}"
    );
    assert!(
        evals(&on) > 500,
        "islands did not engage on shift/const-add bodies (evals={}):\n{on}",
        evals(&on)
    );
}

#[test]
fn unrelated_force_does_not_disable_islands() {
    // The forced signal is written by nothing the islands touch, so every
    // island must keep running.
    let src = SHIFT_ADD_DUT.replace(
        "  reg [31:0] csum = 0;",
        "  reg unrelated_probe;\n  initial begin #37; force unrelated_probe = 1'b1; end\n  reg [31:0] csum = 0;",
    );
    let on = run(&src, true);
    assert!(on.contains("CSUM=00007dfc"), "checksum changed:\n{on}");
    assert!(
        evals(&on) > 500,
        "an unrelated force disabled the islands (evals={}):\n{on}",
        evals(&on)
    );
}

#[test]
fn force_on_an_island_target_is_honored() {
    // The force lands ON a signal an island writes: the two-state stores
    // bypass the §9.3.1 filter, so the block must decline while forced and
    // resume after release. Reference-verified with the force in place.
    let src = SHIFT_ADD_DUT.replace(
        "  integer cyc = 0;",
        "  integer cyc = 0;\n  initial begin #103; force u_a.q = 32'hDEAD_BEEF; #500; release u_a.q; end",
    );
    let on = run(&src, true);
    let off = run(&src, false);
    assert!(
        on.contains("CSUM=20f894e8"),
        "forced-target checksum changed:\n{on}"
    );
    assert!(
        off.contains("CSUM=20f894e8"),
        "4-state forced-target checksum changed:\n{off}"
    );
    let forced_write: u64 = on
        .split("forced_write=")
        .nth(1)
        .and_then(|t| t.split_whitespace().next())
        .and_then(|n| n.parse().ok())
        .unwrap_or(0);
    assert!(
        forced_write > 0,
        "the per-block force guard never engaged:\n{on}"
    );
}
