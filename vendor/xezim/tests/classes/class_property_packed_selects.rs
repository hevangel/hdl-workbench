//! §8.25 / §7.4.1 — packed class properties: parameter-dependent widths, and
//! bit/part selects on them. Reference-validated.
//!
//! Three defects, all silent:
//!
//! 1. A property whose packed range references a class PARAMETER was seeded
//!    from the class's elaborated signature — i.e. at the class's DEFAULT
//!    parameter value — so `bit [W-1:0] pw` in a `P#(16)` was created 8 bits
//!    wide. A later WHOLE assignment re-widened it, so the damage was confined
//!    to bit/part-select writes above the default width, which vanished, and
//!    to `$bits` until the first whole write.
//! 2. A single-bit WRITE to any packed property was dropped entirely: the
//!    element resolver handled only multi-dimensional properties, so a plain
//!    `bit [15:0]` had no handler. Reads fell through to a generic bit-select
//!    and were fine, and the part-select `v[15:8] = ...` beside it worked,
//!    which is what made this hard to see.
//! 3. An ASCENDING property (`bit [0:15]`) labels its bits from the MSB end,
//!    but a part-select WRITE spliced the bits a descending vector would have
//!    — while a single-bit write to the same property was correct.

use xezim::simulate;

fn u(sim: &xezim::compiler::Simulator, n: &str) -> u64 {
    sim.get_signal(n)
        .or_else(|| sim.get_signal(&format!("tb.{}", n)))
        .unwrap_or_else(|| panic!("signal not found: {}", n))
        .to_u64()
        .unwrap_or_else(|| panic!("{} not u64-able (x/z?)", n))
}

/// A parameter-sized property is allocated at the SPECIALIZATION's width, so
/// selects above the class's default width land.
#[test]
fn parameter_sized_property_uses_the_specialization_width() {
    let src = r#"
class P #(int W = 8);
  bit [W-1:0] pw;
endclass
module tb;
  P #(16) a, b, c;
  P #(4)  d;
  int part, bit_hi, bits16, bits4;
  initial begin
    a = new(); b = new(); c = new(); d = new();
    a.pw = '0; a.pw[15:8] = 8'hAA; a.pw[7:0] = 8'h55;
    b.pw = '0; b.pw[15] = 1'b1;
    part   = a.pw;
    bit_hi = b.pw;
    bits16 = $bits(c.pw);
    bits4  = $bits(d.pw);
  end
endmodule
"#;
    let sim = simulate(src, 50).expect("simulate failed");
    assert_eq!(
        u(&sim, "part"),
        0xAA55,
        "part-writes above the default width land"
    );
    assert_eq!(u(&sim, "bit_hi"), 0x8000, "so does a bit-write above it");
    assert_eq!(u(&sim, "bits16"), 16, "$bits before any write");
    assert_eq!(
        u(&sim, "bits4"),
        4,
        "a narrower specialization is independent"
    );
}

/// Bit and part selects across descending, ascending and multi-dimensional
/// properties.
#[test]
fn packed_property_bit_and_part_selects() {
    let src = r#"
class P;
  bit [15:0]     d;
  bit [0:15]     a;
  bit [1:0][7:0] m;
endclass
module tb;
  P p;
  int r_d, r_a, r_m1, r_m0;
  initial begin
    p = new();
    p.d = '0; p.a = '0;
    p.d[15]  = 1'b1;
    p.d[3:0] = 4'hF;
    p.a[0]   = 1'b1;      // ascending: label 0 is the MSB
    p.a[4:7] = 4'hF;      // labels 4..7 from the MSB end
    p.m[1]   = 8'hAB;
    p.m[0]   = 8'hCD;
    #1;
    r_d = p.d; r_a = p.a; r_m1 = p.m[1]; r_m0 = p.m[0];
  end
endmodule
"#;
    let sim = simulate(src, 50).expect("simulate failed");
    assert_eq!(u(&sim, "r_d"), 0x800F, "descending bit and part writes");
    assert_eq!(u(&sim, "r_a"), 0x8F00, "ascending writes map by label");
    assert_eq!(
        (u(&sim, "r_m1"), u(&sim, "r_m0")),
        (0xAB, 0xCD),
        "multi-dim elements"
    );
}
