//! §11.5.1 / §7.4.1 — indexed part-selects on an ASCENDING packed vector.
//! Reference-validated.
//!
//! On a `logic [0:15]` the bit labels run from the MSB end. The constant form
//! `v[a:b]` was mapped accordingly, and so was a single-bit select — but the
//! indexed forms `v[b +: n]` / `v[b -: n]` were not, so they read and wrote the
//! bits a DESCENDING vector would have. The two views of the same vector
//! therefore disagreed about what index 4 meant.

use xezim::simulate;

fn u(sim: &xezim::compiler::Simulator, n: &str) -> u64 {
    sim.get_signal(n)
        .or_else(|| sim.get_signal(&format!("tb.{}", n)))
        .unwrap_or_else(|| panic!("signal not found: {}", n))
        .to_u64()
        .unwrap_or_else(|| panic!("{} not u64-able (x/z?)", n))
}

#[test]
fn ascending_indexed_part_selects_read_and_write_by_label() {
    let src = r#"
module tb;
  logic [0:15] av;
  logic [15:0] dv;
  logic [0:15] aw;
  logic [15:0] dw;
  int a_up, a_dn, a_bit, a_const, a_wr;
  int d_up, d_dn, d_bit, d_wr;
  initial begin
    av = 16'h1234;
    dv = 16'h1234;
    #1;
    a_up = av[4 +: 4];
    a_dn = av[7 -: 4];
    a_bit = av[4];
    a_const = av[4:7];
    d_up = dv[4 +: 4];
    d_dn = dv[7 -: 4];
    d_bit = dv[4];
    aw = '0; aw[4 +: 4] = 4'hF; a_wr = aw;
    dw = '0; dw[4 +: 4] = 4'hF; d_wr = dw;
  end
endmodule
"#;
    let sim = simulate(src, 50).expect("simulate failed");
    // 16'h1234 on [0:15] is 0001 0010 0011 0100 read from the MSB end.
    assert_eq!(u(&sim, "a_up"), 0x2, "ascending [4 +: 4] takes labels 4..7");
    assert_eq!(u(&sim, "a_dn"), 0x2, "ascending [7 -: 4] takes labels 4..7");
    assert_eq!(u(&sim, "a_const"), 0x2, "the constant form agrees");
    assert_eq!(u(&sim, "a_bit"), 0, "and so does a single-bit select");
    assert_eq!(
        u(&sim, "a_wr"),
        0x0f00,
        "an ascending indexed part-WRITE lands by label"
    );
    assert_eq!(u(&sim, "d_up"), 0x3, "descending is unchanged");
    assert_eq!(u(&sim, "d_dn"), 0x3);
    assert_eq!(u(&sim, "d_bit"), 1);
    assert_eq!(u(&sim, "d_wr"), 0x00f0);
}
