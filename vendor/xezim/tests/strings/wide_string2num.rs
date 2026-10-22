//! Regression tests for wide string-to-number conversion.
//!
//! Decimal, hex, octal, and binary strings ≥ 2^63 must correctly parse into
//! arbitrarily wide destination variables, with truncation to the destination
//! width per LRM §6.16.8 (string methods), §11.4.12 (formatted I/O), and
//! §20.13.2 ($readmemd).
//!
//! Before the fix: $fscanf/$sscanf used i64::from_str_radix() → 0 for ≥ 2^63;
//! $value$plusargs hard-coded width=64; string.atoi() returned 0 for ≥ 2^64.
//! All now pass through Value::from_str_radix() with the destination width.

use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};
use xezim::simulate;

fn out(sim: &xezim::compiler::Simulator, tag: &str) -> String {
    sim.output
        .iter()
        .map(|o| o.message.trim().to_string())
        .find(|l| l.starts_with(&format!("NOTE: {}", tag)))
        .unwrap_or_else(|| panic!("missing output for tag: {}", tag))
}

/// A scratch file unique to this process and instant. `$fopen` here resolves
/// relative to the test harness's working directory — the crate root — so a
/// fixed name collides across the tests that link together into this one
/// binary and leaves debris behind if an assertion trips first.
fn temp_file_path(tag: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let mut p = std::env::temp_dir();
    p.push(format!("xezim_{}_{}_{}", tag, std::process::id(), nanos));
    p
}

/// Run `src` with the given plusargs visible to `$value$plusargs`.
fn sim_with_plusargs(src: &str, plusargs: &[&str]) -> xezim::compiler::Simulator {
    let source = src.to_string();
    let plusargs: Vec<String> = plusargs.iter().map(|s| (*s).to_string()).collect();
    xezim::simulate_multi(
        &[source],
        100_000,
        None,
        &[],
        &[],
        None,
        false,
        None,
        None,
        &[],
        &plusargs,
        None,
        &[],
        0,
        u64::MAX,
        None,
        &[],
        None,
        None,
        None,
        None,
        false,
    )
    .expect("simulate failed")
}

// ---------------------------------------------------------------------------
// $sscanf with wide hex / dec / bin / oct into wide vectors (§11.4.12)
// ---------------------------------------------------------------------------

const SSCANF_WIDE: &str = r#"
module tb;
  logic [159:0] w;
  logic [63:0]  v64;
  logic [31:0]  v32;
  int           cnt;
  initial begin
    w = '0; cnt = $sscanf("0123456789abcdef0123456789abcdef01234567", "%x", w);
    $display("NOTE: s_hex160 %0d %h", cnt, w);
    v64 = '0; cnt = $sscanf("8000000000000000", "%x", v64);
    $display("NOTE: s_hex64 %0d %h", cnt, v64);
    v32 = '0; cnt = $sscanf("deadbeefdeadbeef", "%x", v32);
    $display("NOTE: s_hex32 %0d %h", cnt, v32);
    w = '0; cnt = $sscanf("3ab4901f2c3d5e60", "%x", w);
    $display("NOTE: s_hexsmall %0d %h", cnt, w);
    w = '0; cnt = $sscanf("11111111111111111111111111111111111111111111111111111111111111111111111111111111", "%b", w);
    $display("NOTE: s_bin160 %0d %h", cnt, w);
    w = '0; cnt = $sscanf("1000000000000000000000", "%o", w);
    $display("NOTE: s_oct160 %0d %h", cnt, w);
    v64 = '0; cnt = $sscanf("12345678901234567890", "%d", v64);
    $display("NOTE: s_posdec64 %0d %h", cnt, v64);
    v64 = '0; cnt = $sscanf("-12345678901234567890", "%d", v64);
    $display("NOTE: s_negdec64 %0d %h", cnt, v64);
    w = '0; cnt = $sscanf("-12345678901234567890", "%d", w);
    $display("NOTE: s_negdec160 %0d %h", cnt, w);
    w = '0; cnt = $sscanf("-1", "%d", w);
    $display("NOTE: s_negone160 %0d %h", cnt, w);
    v64 = '0; cnt = $sscanf("0123456789abcdef", "%16x", v64);
    $display("NOTE: s_w16x %0d %h", cnt, v64);
    w = '0; cnt = $sscanf("8000000000000000", "%16x", w);
    $display("NOTE: s_w16xwide %0d %h", cnt, w);
    $finish;
  end
endmodule
"#;

#[test]
fn sscanf_wide_hex_preserves_full_width() {
    let sim = simulate(SSCANF_WIDE, 100).expect("simulate failed");
    // 40-hex → 160-bit: all 160 bits survive (was truncated to low-32)
    assert_eq!(
        out(&sim, "s_hex160"),
        "NOTE: s_hex160 1 0123456789abcdef0123456789abcdef01234567"
    );
    // 8000000000000000 → 64-bit: full 64-bit value (was 0)
    assert_eq!(out(&sim, "s_hex64"), "NOTE: s_hex64 1 8000000000000000");
    // deadbeefdeadbeef → 32-bit: low 32 = deadbeef (was 0)
    assert_eq!(out(&sim, "s_hex32"), "NOTE: s_hex32 1 deadbeef");
    // 3ab4901f2c3d5e60 < 2^63 → 160-bit: full value (was low-32 only)
    assert_eq!(
        out(&sim, "s_hexsmall"),
        "NOTE: s_hexsmall 1 0000000000000000000000003ab4901f2c3d5e60"
    );
}

#[test]
fn sscanf_wide_bin_and_oct() {
    let sim = simulate(SSCANF_WIDE, 100).expect("simulate failed");
    // 80 ones → 160-bit: low 80 bits all 1
    assert_eq!(
        out(&sim, "s_bin160"),
        "NOTE: s_bin160 1 00000000000000000000ffffffffffffffffffff"
    );
    // 2^63 via octal → 160-bit
    assert_eq!(
        out(&sim, "s_oct160"),
        "NOTE: s_oct160 1 0000000000000000000000008000000000000000"
    );
}

#[test]
fn sscanf_wide_decimal_signed() {
    let sim = simulate(SSCANF_WIDE, 100).expect("simulate failed");
    // 12345678901234567890 mod 2^64 = 0xab54a98ceb1f0ad2
    assert_eq!(
        out(&sim, "s_posdec64"),
        "NOTE: s_posdec64 1 ab54a98ceb1f0ad2"
    );
    // Two's complement: 2^64 - 0xab54a98ceb1f0ad2 = 0x54ab567314e0f52e
    assert_eq!(
        out(&sim, "s_negdec64"),
        "NOTE: s_negdec64 1 54ab567314e0f52e"
    );
    // Same payload into 160 bits: the two's complement belongs to the
    // destination, so everything above the magnitude has to come back ones.
    // A `Value::add`-based negation stops carrying at bit 127 and answers
    // 0x00000000ffffffff_ffffffff_ffffffff54ab567314e0f52e instead.
    assert_eq!(
        out(&sim, "s_negdec160"),
        "NOTE: s_negdec160 1 ffffffffffffffffffffffff54ab567314e0f52e"
    );
    assert_eq!(
        out(&sim, "s_negone160"),
        "NOTE: s_negone160 1 ffffffffffffffffffffffffffffffffffffffff"
    );
}

#[test]
fn sscanf_field_width_16x() {
    let sim = simulate(SSCANF_WIDE, 100).expect("simulate failed");
    assert_eq!(out(&sim, "s_w16x"), "NOTE: s_w16x 1 0123456789abcdef");
    assert_eq!(
        out(&sim, "s_w16xwide"),
        "NOTE: s_w16xwide 1 0000000000000000000000008000000000000000"
    );
}

// ---------------------------------------------------------------------------
// $fscanf with wide hex (file-based — production pattern, §11.4.12)
// ---------------------------------------------------------------------------

fn fscanf_wide_src(path_sv: &str) -> String {
    format!(
        r#"
module tb;
  logic [160:0] w;
  int           fd, cnt;
  string        tok;
  initial begin
    fd = $fopen("{}", "w");
    $fdisplay(fd, "attribute 3ab4901f2c3d5e60");
    $fdisplay(fd, "data deadbeefdeadbeef");
    $fclose(fd);
    fd = $fopen("{}", "r");
    cnt = $fscanf(fd, "%s", tok);
    w = '0; cnt = $fscanf(fd, "%x", w);
    $display("NOTE: f_hexsmall %0d %h", cnt, w);
    cnt = $fscanf(fd, "%s", tok);
    w = '0; cnt = $fscanf(fd, "%x", w);
    $display("NOTE: f_hexlarge %0d %h", cnt, w);
    cnt = $fscanf(fd, "%s", tok);
    $display("NOTE: f_eof %0d", cnt);
    $fclose(fd);
    $finish;
  end
endmodule
"#,
        path_sv, path_sv
    )
}

#[test]
fn fscanf_wide_advances_past_large_token() {
    // §11.4.12: a token >= 2^63 must scan 1 conversion and advance the
    // file position. Before the fix it scanned 0 and re-read the token
    // (production "wrong cmd" fatal path).
    let path = temp_file_path("wide_fscanf.txt");
    let path_sv = path.to_string_lossy().replace('\\', "\\\\");
    let sim = simulate(&fscanf_wide_src(&path_sv), 100).expect("simulate failed");
    assert_eq!(
        out(&sim, "f_hexsmall"),
        "NOTE: f_hexsmall 1 00000000000000000000000003ab4901f2c3d5e60"
    );
    assert_eq!(
        out(&sim, "f_hexlarge"),
        "NOTE: f_hexlarge 1 0000000000000000000000000deadbeefdeadbeef"
    );
    // After consuming all tokens, $fscanf returns -1 or 0 (clean EOF)
    let eof = out(&sim, "f_eof");
    let n: i32 = eof.split_whitespace().last().unwrap().parse().unwrap();
    assert!(n <= 0, "clean EOF expected, got: {}", eof);
    let _ = std::fs::remove_file(&path);
}

// ---------------------------------------------------------------------------
// §6.16.8 string methods: atoi, atohex, atooct, atobin with wide inputs
// ---------------------------------------------------------------------------

const STRING_METHODS_WIDE: &str = r#"
module tb;
  int  si;
  real rv;
  initial begin
    // 50-digit decimal (>= 2^64) wraps to low-32 signed int
    si = "12345678901234567890123456789012345678901234567890".atoi();
    $display("NOTE: atoi50 %0d", si);
    // 20-digit (>= 2^63) wraps to low-32
    si = "12345678901234567890".atoi();
    $display("NOTE: atoi20 %0d", si);
    // 2^63-1 → low 32 = 0xFFFFFFFF = -1
    si = "9223372036854775807".atoi();
    $display("NOTE: atoi63 %0d", si);
    // atohex 16-hex = deadbeefdeadbeef → low 32 = deadbeef = -559038737
    si = "deadbeefdeadbeef".atohex();
    $display("NOTE: atohex16 %0d", si);
    // atohex 32-hex → low 32 = 89abcdef = -1985872337
    si = "0123456789abcdef0123456789abcdef".atohex();
    $display("NOTE: atohex32 %0d", si);
    // atooct 37777777777 = 0xFFFFFFFF → low 32 = 0xFFFFFFFF = -1
    si = "37777777777".atooct();
    $display("NOTE: atooct %0d", si);
    // atobin 32 ones → 0xFFFFFFFF = -1
    si = "11111111111111111111111111111111".atobin();
    $display("NOTE: atobin %0d", si);
    // atoreal negative
    rv = "-2.71828".atoreal();
    $display("NOTE: atoreal %0.5f", rv);
    $finish;
  end
endmodule
"#;

#[test]
fn string_atoi_wide_wraps_to_low32() {
    // §6.16.8: atoi returns a 32-bit signed int — wide decimals wrap.
    let sim = simulate(STRING_METHODS_WIDE, 100).expect("simulate failed");
    // 50-digit mod 2^32 = 0xCE3F0AD2 = -834729262 signed
    assert_eq!(out(&sim, "atoi50"), "NOTE: atoi50 -834729262");
    // 20-digit mod 2^32 = 0xEB1F0AD2 = -350287150 signed
    assert_eq!(out(&sim, "atoi20"), "NOTE: atoi20 -350287150");
    // 2^63-1 = 0x7FFFFFFFFFFFFFFF → low 32 = 0xFFFFFFFF = -1
    assert_eq!(out(&sim, "atoi63"), "NOTE: atoi63 -1");
}

#[test]
fn string_atohex_atooct_atobin_wide() {
    let sim = simulate(STRING_METHODS_WIDE, 100).expect("simulate failed");
    assert_eq!(out(&sim, "atohex16"), "NOTE: atohex16 -559038737");
    assert_eq!(out(&sim, "atohex32"), "NOTE: atohex32 -1985229329");
    assert_eq!(out(&sim, "atooct"), "NOTE: atooct -1");
    assert_eq!(out(&sim, "atobin"), "NOTE: atobin -1");
    assert_eq!(out(&sim, "atoreal"), "NOTE: atoreal -2.71828");
}

// ---------------------------------------------------------------------------
// $value$plusargs with %d into wide destinations (§21.8)
// ---------------------------------------------------------------------------

const VALUE_PLUSARGS_WIDE: &str = r#"
module tb;
  logic [159:0]        wp;
  logic signed [159:0] sp;
  logic [63:0]         u64;
  int                  cnt;
  initial begin
    wp = '0; cnt = $value$plusargs("WPOS=%d", wp);
    $display("NOTE: vp_wpos %0d %h", cnt, wp);
    wp = '0; cnt = $value$plusargs("WNEG=%d", wp);
    $display("NOTE: vp_wneg %0d %h", cnt, wp);
    wp = '0; cnt = $value$plusargs("WALL=%d", wp);
    $display("NOTE: vp_wall %0d %h", cnt, wp);
    sp = '0; cnt = $value$plusargs("SNEG=%d", sp);
    $display("NOTE: vp_sneg %0d %h", cnt, sp);
    u64 = 64'hDEAD_BEEF_DEAD_BEEF;
    cnt = $value$plusargs("BAD=%d", u64);
    $display("NOTE: vp_bad %0d %h", cnt, u64);
    u64 = '0; cnt = $value$plusargs("N64=%d", u64);
    $display("NOTE: vp_n64 %0d %h", cnt, u64);
    $finish;
  end
endmodule
"#;

const VALUE_PLUSARGS: &[&str] = &[
    "WPOS=12345678901234567890",
    "WNEG=-12345678901234567890",
    "WALL=-1",
    "SNEG=-98765432109876543210",
    "BAD=abc",
    "N64=18446744073709551621",
];

#[test]
fn value_plusargs_wide_decimal_keeps_all_160_bits() {
    // Before the fix the width was hard-coded to 64, so everything above the
    // low 64 bits of a 160-bit destination was lost regardless of the payload.
    let sim = sim_with_plusargs(VALUE_PLUSARGS_WIDE, VALUE_PLUSARGS);
    // 12345678901234567890 = 0xAB54A98CEB1F0AD2, zero-extended to 160.
    assert_eq!(
        out(&sim, "vp_wpos"),
        "NOTE: vp_wpos 1 000000000000000000000000ab54a98ceb1f0ad2"
    );
    // 2^64+5 truncated to a 64-bit destination.
    assert_eq!(out(&sim, "vp_n64"), "NOTE: vp_n64 1 0000000000000005");
}

#[test]
fn value_plusargs_wide_decimal_applies_sign_at_destination_width() {
    // The sign was being dropped: `to_digit` returned None for '-', so the
    // minus was skipped and the magnitude reported — `-1` and `1` produced the
    // same bits. Two's complement has to be taken at the destination width.
    let sim = sim_with_plusargs(VALUE_PLUSARGS_WIDE, VALUE_PLUSARGS);
    // 2^160 - 12345678901234567890
    assert_eq!(
        out(&sim, "vp_wneg"),
        "NOTE: vp_wneg 1 ffffffffffffffffffffffff54ab567314e0f52e"
    );
    // -1 into 160 bits is all ones, not 1.
    assert_eq!(
        out(&sim, "vp_wall"),
        "NOTE: vp_wall 1 ffffffffffffffffffffffffffffffffffffffff"
    );
    // 2^160 - 98765432109876543210, into a signed destination.
    assert_eq!(
        out(&sim, "vp_sneg"),
        "NOTE: vp_sneg 1 fffffffffffffffffffffffaa55ab2c71ad98116"
    );
}

#[test]
fn value_plusargs_rejects_non_numeric_payload() {
    // `+BAD=abc` matched the prefix and then accumulated nothing, which summed
    // to zero — the register was overwritten with 0 and the call reported a
    // successful match. A payload that is not a number is not a match: the
    // destination keeps its old value and $value$plusargs returns 0.
    let sim = sim_with_plusargs(VALUE_PLUSARGS_WIDE, VALUE_PLUSARGS);
    assert_eq!(out(&sim, "vp_bad"), "NOTE: vp_bad 0 deadbeefdeadbeef");
}
