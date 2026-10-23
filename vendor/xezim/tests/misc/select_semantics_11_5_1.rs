//! IEEE 1800-2023 §11.5.1 bit- and part-select semantics, in the statement
//! interpreter (`initial`) and the compiled path (`always_comb`):
//!
//! * an index or bound carrying x/z selects nothing: reads give x, writes
//!   are discarded (previously bit 0 was read or written);
//! * a constant select below a vector's declared low bound reads x
//!   (previously clamped to the low bit);
//! * an ascending vector `logic [3:10] a` places label `p` at physical bit
//!   `10 - p` for reads AND writes (previously the write side ignored the
//!   declared low bound: `a[3] = 1` set bit 4, not the MSB);
//! * an x or out-of-range element index into a packed 2-D value reads a
//!   whole-x element.
//!
//! A PARTIALLY out-of-range select reads x per bit (`v[12:9]` on `[10:3]`
//! is `xx10`), differential-verified in `misc/operators_11_select_reduce.rs`;
//! `XEZIM_OOB_SELECT=whole` gives the whole-x form some tools use, and
//! discards a partially out-of-range WRITE. By default a partial write
//! stores the in-range bits (§11.5.1), pinned in
//! `collections/stream_justify_assoc_default_partsel.rs`.
use xezim::simulate;

const SRC: &str = r#"
module top;
  logic [7:0]  d8 = 8'hB7;
  logic [10:3] v  = 8'hA5;
  logic [7:0]  xi;            // x
  logic [3:10] a;
  logic [1:0][7:0] p;
  logic [31:0] pxi;
  int di, dx;
  // compiled (always_comb) reads
  logic       c_bit, c_zbit, c_dbit, c_nbit;
  logic [3:0] c_up, c_dn, c_drng;
  always_comb c_bit  = d8[xi];
  always_comb c_up   = d8[xi +: 4];
  always_comb c_dn   = d8[xi -: 4];
  always_comb c_dbit = v[2];
  always_comb c_drng = v[5:2];
  always_comb c_nbit = d8[0-1];
  logic [3:0] c_hrng, c_frng;
  always_comb c_hrng = v[12:9];      // past the MSB -> whole x
  always_comb c_frng = d8[9:6];
  // interpreter results
  logic [7:0] r_abit, r_a2, r_a11, r_a4_6, r_a5, r_adx, r_aw1, r_aw3, r_aw8, r_aw10, r_awdx, r_aw2;
  logic [7:0] r_pe7, r_pe4, r_pe3;
  logic [15:0] r_pe8;
  logic       r_dbit, r_xbit;
  logic [3:0] r_drng, r_xup;
  initial begin
    #1;
    r_dbit = v[2];
    r_drng = v[5:2];
    r_xbit = d8[xi];
    r_xup  = d8[xi +: 4];
    a = 8'hA5;
    r_a4_6 = {5'b0, a[4:6]};     // positions 4,5,6 -> bits 6,5,4 = 010
    r_a5   = {7'b0, a[5]};
    r_a2   = {7'b0, a[2]};       // below range -> x
    r_a11  = {7'b0, a[11]};      // above range -> x
    dx = 4'bx;
    r_adx  = {7'b0, a[dx]};      // x index -> x
    a = 8'h00; a[4:6] = 3'b101; r_aw1 = a;   // -> 8'h50
    a = 8'h00; a[3]   = 1'b1;   r_aw8 = a;   // pos 3 = MSB -> 8'h80
    a = 8'h00; a[10]  = 1'b1;   r_aw10 = a;  // pos 10 = LSB -> 8'h01
    a = 8'h00; a[2]   = 1'b1;   r_aw2 = a;   // out of range -> no write
    a = 8'h00; a[dx]  = 1'b1;   r_awdx = a;  // x index -> no write
    p = 16'hA53C;
    pxi = 32'hxxxxxxxx;
    r_pe7 = p[pxi];              // x element index -> xx
    r_pe4 = p[-1];               // below -> xx
    r_pe3 = p[2];                // above -> xx
    p = 16'h0FA5; p[pxi] = 8'hAB; r_pe8 = p;  // x index store -> unchanged
  end
endmodule
"#;

/// The value as a `%b` string, MSB first (`Value::to_string` decodes the
/// bits as ASCII, so it cannot show x).
fn s(sim: &xezim::compiler::Simulator, n: &str) -> String {
    let v = sim
        .get_signal(n)
        .or_else(|| sim.get_signal(&format!("top.{}", n)))
        .unwrap_or_else(|| panic!("signal not found: {}", n));
    (0..v.width as usize)
        .rev()
        .map(|i| match v.get_bit(i) {
            xezim_core::value::LogicBit::Zero => '0',
            xezim_core::value::LogicBit::One => '1',
            xezim_core::value::LogicBit::X => 'x',
            xezim_core::value::LogicBit::Z => 'z',
        })
        .collect()
}

#[test]
fn select_semantics_11_5_1() {
    let sim = simulate(SRC, 100).expect("simulate failed");
    // compiled path: unknown index / below-range constant -> x
    assert_eq!(s(&sim, "c_bit"), "x", "d8[xi] with xi = x");
    assert_eq!(s(&sim, "c_up"), "xxxx", "d8[xi +: 4]");
    assert_eq!(s(&sim, "c_dn"), "xxxx", "d8[xi -: 4]");
    assert_eq!(s(&sim, "c_dbit"), "x", "v[2] on [10:3]");
    assert_eq!(
        s(&sim, "c_drng"),
        "101x",
        "v[5:2] on [10:3]: label 2 is out of range, 5..3 read"
    );
    assert_eq!(s(&sim, "c_nbit"), "x", "d8[-1]");
    assert_eq!(
        s(&sim, "c_hrng"),
        "xx10",
        "v[12:9]: labels 12,11 past the MSB"
    );
    assert_eq!(s(&sim, "c_frng"), "xx10", "d8[9:6]: bits 9,8 past the MSB");
    // interpreter: same rules
    assert_eq!(s(&sim, "r_dbit"), "x");
    assert_eq!(s(&sim, "r_drng"), "101x");
    assert_eq!(s(&sim, "r_xbit"), "x");
    assert_eq!(s(&sim, "r_xup"), "xxxx");
    // ascending vector reads
    assert_eq!(s(&sim, "r_a4_6"), "00000010");
    assert_eq!(s(&sim, "r_a5"), "00000001");
    assert_eq!(s(&sim, "r_a2"), "0000000x");
    assert_eq!(s(&sim, "r_a11"), "0000000x");
    assert_eq!(s(&sim, "r_adx"), "0000000x");
    // ascending vector writes
    assert_eq!(
        s(&sim, "r_aw1"),
        "01010000",
        "a[4:6] = 101 lands on bits 6..4"
    );
    assert_eq!(s(&sim, "r_aw8"), "10000000", "a[3] is the MSB");
    assert_eq!(s(&sim, "r_aw10"), "00000001", "a[10] is the LSB");
    assert_eq!(s(&sim, "r_aw2"), "00000000", "out-of-range write discarded");
    assert_eq!(s(&sim, "r_awdx"), "00000000", "x-index write discarded");
    // packed 2-D elements
    assert_eq!(s(&sim, "r_pe7"), "xxxxxxxx", "x element index");
    assert_eq!(s(&sim, "r_pe4"), "xxxxxxxx", "element below range");
    assert_eq!(s(&sim, "r_pe3"), "xxxxxxxx", "element above range");
    assert_eq!(
        s(&sim, "r_pe8"),
        "0000111110100101",
        "x-index element store discarded"
    );
}
