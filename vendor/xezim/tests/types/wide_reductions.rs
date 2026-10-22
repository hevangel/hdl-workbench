//! §11.4.9 — reduction operators on wide and four-state operands.
//! Reference-validated.
//!
//! `Value::reduce_xor` parity-counted `to_u64()`, which a value wider than
//! 64 bits cannot give, so every bit above bit 63 was dropped: `^(128'h1 <<
//! 64)` was 0 and `~^` its complement. Each reduction is checked on wide
//! and four-state values through the interpreter (`$display` operands),
//! a constant parameter, a continuous assignment and a compiled `always @*`
//! (two-state `bit` operands, so the two-state executors see them too).

use xezim::simulate;

fn t_lines(sim: &xezim::compiler::Simulator) -> Vec<String> {
    sim.output
        .iter()
        .map(|o| o.message.trim_end().to_string())
        .filter(|l| l.starts_with("T|"))
        .collect()
}

#[test]
fn reductions_on_wide_and_four_state_values() {
    const SRC: &str = r#"
module tb;
  logic [127:0] s; logic [95:0] m; logic [64:0] e; logic [63:0] h;
  bit [127:0] bs; bit [7:0] a, b;
  logic [7:0] q;
  localparam logic [127:0] P = 128'h1 << 64;
  localparam PX = ^P;
  localparam PA = &8'hFF;
  localparam PO = |(128'h1 << 100);
  localparam PS = ^5'sb10000;
  localparam PN = ~^4'b0111;
  logic [3:0] r_cmb;
  logic [7:0] cr;
  // compiled continuous / always paths
  assign r_cmb = {^s, &s, |s, ~^s};
  always @* cr = {^bs, &bs, |bs, ~&bs, ~|bs, ~^bs, ^(a + b), |(~a)};
  task show(string t);
    #1 $display("T|%s ^%b &%b |%b ~&%b ~|%b ~^%b cmb=%b cr=%b", t, ^s, &s, |s, ~&s, ~|s, ~^s, r_cmb, cr);
  endtask
  initial begin
    $display("T|params %b %b %b %b %b", PX, PA, PO, PS, PN);
    s = 128'h1 << 64;        bs = 128'h1 << 64;  a = 8'hF0; b = 8'h10; show("a");
    s = '1;                  bs = '1;  a = 8'hFF; b = 8'h01; show("b");
    s = {64'h0, 64'hFFFF_FFFF_FFFF_FFFF}; bs = {64'h1, 64'h0}; show("c");
    s = {1'bx, 127'h0};      bs = {1'b1, 127'h0}; a = 8'h00; show("d");
    s = {1'bx, 127'h1};      show("e");
    s = {1'bz, 63'h0, 64'hFFFF_FFFF_FFFF_FFFF}; show("f");
    s = {64'hFFFF_FFFF_FFFF_FFFF, 1'bx, 63'h7FFF_FFFF_FFFF_FFFF}; show("g");
    s = 128'h1f9f9f9f9f9f9f9f9f9f9f9f9f9f9f9f; show("h");
    m = 96'h1_0000_0000_0000_0000; e = 65'h1_0000_0000_0000_0000; h = 64'h8000_0000_0000_0001;
    $display("T|m %b %b %b e %b %b %b h %b", ^m, &m, |m, ^e, &e, |e, ^h);
    e = '1; $display("T|e1 %b %b", &e, ^e);
    q = 8'b1x0z_1111; $display("T|q %b %b %b %b", ^q, &q, |q, ~^q);
    $finish;
  end
endmodule"#;
    let sim = simulate(SRC, 100).expect("sim");
    assert_eq!(
        t_lines(&sim),
        [
            "T|params 1 1 1 1 0",
            "T|a ^1 &0 |1 ~&1 ~|0 ~^0 cmb=1010 cr=10110001",
            "T|b ^0 &1 |1 ~&0 ~|0 ~^1 cmb=0111 cr=01100100",
            "T|c ^0 &0 |1 ~&1 ~|0 ~^1 cmb=0011 cr=10110000",
            "T|d ^x &0 |x ~&1 ~|x ~^x cmb=x0xx cr=10110011",
            "T|e ^x &0 |1 ~&1 ~|0 ~^x cmb=x01x cr=10110011",
            "T|f ^x &0 |1 ~&1 ~|0 ~^x cmb=x01x cr=10110011",
            "T|g ^x &x |1 ~&x ~|0 ~^x cmb=xx1x cr=10110011",
            "T|h ^1 &0 |1 ~&1 ~|0 ~^0 cmb=1010 cr=10110011",
            "T|m 1 0 1 e 1 0 1 h 0",
            "T|e1 1 1",
            "T|q x 0 1 x",
        ],
    );
}

/// Reductions in constant range expressions take the operand's own width:
/// the elaborator's integer evaluator reduced the 64-bit footprint, so
/// `&4'hF` was 0 and every range built on it shrank.
#[test]
fn reductions_in_constant_ranges() {
    const SRC: &str = r#"
module tb;
  localparam logic [3:0] F = 4'hF;
  logic [(&F) ? 7 : 3 : 0] a;
  logic [(^5'b10110) ? 7 : 3 : 0] b;
  logic [(&4'hF) * 4 + 3 : 0] c;
  logic [(~^3'b011) + 1 : 0] d;
  initial $display("T|%0d %0d %0d %0d", $bits(a), $bits(b), $bits(c), $bits(d));
endmodule"#;
    let sim = simulate(SRC, 100).expect("sim");
    assert_eq!(t_lines(&sim), ["T|8 8 8 3"]);
}

/// §11.4.9: parameter and generate reductions include every bit of a wide
/// operand, including operands whose literal cannot fit the scalar evaluator.
#[test]
fn wide_reductions_in_ranges_and_generate_conditions() {
    const SRC: &str = r#"
module tb;
  localparam logic [127:0] HIGH = 128'h1 << 100;
  localparam logic [127:0] BOTH = (128'h1 << 100) | 128'h1;
  localparam logic [64:0] FULL = '1;
  localparam int WIDTH = (^HIGH) ? 9 : 3;
  logic [WIDTH-1:0] a;
  logic [(^128'h10000000000000000000000000) ? 10 : 2 : 0] b;
  logic [(^(128'h1 << 100)) ? 12 : 2 : 0] c;
  logic [(&FULL) ? 14 : 2 : 0] d;
  logic [(~^BOTH) ? 16 : 2 : 0] e;
  logic [(~|HIGH) ? 2 : 18 : 0] f;
  if ((^HIGH) && (|HIGH) && (&FULL) && !(~&FULL) && !(~^HIGH)) begin : selected
    initial $display("T|generate wide=1");
  end else begin : rejected
    initial $display("T|generate wide=0");
  end
  initial begin
    #1;
    $display("T|ranges %0d %0d %0d %0d %0d %0d", $bits(a), $bits(b), $bits(c), $bits(d), $bits(e), $bits(f));
    $finish;
  end
endmodule"#;
    let sim = simulate(SRC, 100).expect("sim");
    assert_eq!(
        t_lines(&sim),
        ["T|generate wide=1", "T|ranges 9 11 13 15 17 19"],
    );
}
