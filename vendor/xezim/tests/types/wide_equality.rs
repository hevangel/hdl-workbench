//! §11.4.5: equality of operands wider than 64 bits compares every bit.
//! `==`/`!=` read only the low 64 bits of each operand, so two wide values
//! that differ only above bit 63 compared equal — every self-check of a
//! wide `randomize()` result passed whatever it held.
//!
//! Expected values come from the reference simulator.

use xezim::simulate;

#[test]
fn wide_equality_compares_all_bits() {
    let src = r#"
module top;
  bit [71:0] y, z;
  logic [99:0] p, q;
  bit [127:0] u;
  bit [63:0] s64;
  initial begin
    y = 72'h400000000000000000; z = 72'h010000000000000000;
    p = 100'h8_0000_0000_0000_0000_0000_0001; q = 100'h0_0000_0000_0000_0000_0000_0001;
    u = 128'h1_0000_0000_0000_0005; s64 = 64'h5;
    $display("T|eq %0d %0d %0d %0d", y == z, y != z, y == 72'h400000000000000000, z == 72'h010000000000000001);
    $display("T|mix %0d %0d %0d %0d", p == q, p != q, u == s64, u[63:0] == s64);
    $display("T|ins %0d %0d", y inside {z, 72'h400000000000000000}, z inside {y});
    $display("T|if %0d", (y == z) ? 1 : 2);
  end
endmodule
"#;
    let got: Vec<String> = simulate(src, 100)
        .expect("simulate failed")
        .output
        .iter()
        .map(|o| o.message.clone())
        .filter(|m| m.starts_with("T|"))
        .collect();
    assert_eq!(
        got,
        ["T|eq 0 1 1 0", "T|mix 0 1 0 1", "T|ins 1 0", "T|if 2"]
    );
}
