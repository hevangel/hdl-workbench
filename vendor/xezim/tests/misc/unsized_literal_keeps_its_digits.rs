//! §5.7.1 — an UNSIZED based literal (`'h1234…`) was parsed at a flat 32 bits,
//! silently discarding every digit above bit 31.
//!
//!     assign wide = 'h123456789ABCDEF0;   // kept only 9abcdef0
//!
//! An unsized number takes its size from the context (at least 32 bits), but it
//! must never DROP digits the source actually wrote. The natural width now
//! comes from the digit string — `max(32, bits implied by the digits)` — and the
//! usual context resize widens or truncates from there, so small literals are
//! unaffected.
//!
//! This one is nastier than a plain wrong value: a self-checking testbench can
//! still report PASS, because the literal it compares against is truncated
//! identically. A real testbench did exactly that — every check passed while the
//! bus carried the low 32 bits of a 64-bit constant.

use xezim::simulate;

fn get(sim: &xezim::compiler::Simulator, name: &str) -> u64 {
    sim.get_signal(name)
        .or_else(|| sim.get_signal(&format!("tb.{}", name)))
        .unwrap_or_else(|| panic!("signal not found: {}", name))
        .to_u64()
        .unwrap_or_else(|| panic!("signal {} not u64-able", name))
}

const SRC: &str = r#"
module tb;
  logic [63:0] hex_unsized;
  logic [63:0] hex_sized;
  logic [63:0] oct_unsized;
  logic [63:0] bin_unsized;
  logic [63:0] dec_unsized;
  logic [31:0] small_unsized;   // unchanged by the fix
  logic        eq_hex;
  initial begin
    hex_unsized   = 'h123456789ABCDEF0;
    hex_sized     = 64'h123456789ABCDEF0;
    oct_unsized   = 'o1234567012345670123;
    bin_unsized   = 'b1010101010101010101010101010101010101010;
    dec_unsized   = 'd1234567890123456789;
    small_unsized = 'hDEAD;
    #1;
    eq_hex = (hex_unsized === hex_sized);
  end
endmodule
"#;

#[test]
fn unsized_literal_keeps_digits_beyond_32_bits() {
    let sim = simulate(SRC, 100).expect("simulate failed");
    // The whole point: unsized must equal the explicitly sized form.
    assert_eq!(get(&sim, "hex_unsized"), 0x123456789ABCDEF0);
    assert_eq!(get(&sim, "hex_sized"), 0x123456789ABCDEF0);
    assert_eq!(get(&sim, "eq_hex") & 1, 1);
    // Other radices carry their digits too.
    assert_eq!(get(&sim, "oct_unsized"), 0o1234567012345670123);
    assert_eq!(
        get(&sim, "bin_unsized"),
        0b1010101010101010101010101010101010101010
    );
    assert_eq!(get(&sim, "dec_unsized"), 1234567890123456789);
    // A literal that already fit in 32 bits behaves exactly as before.
    assert_eq!(get(&sim, "small_unsized") & 0xFFFF_FFFF, 0xDEAD);
}
