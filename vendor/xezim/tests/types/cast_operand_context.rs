//! §6.24.1: a cast returns the value a variable of the cast type would hold
//! after being ASSIGNED the operand, so an integral cast type is the
//! operand's context width (§11.6.1) — `int'(a + b)` over two 8-bit operands
//! sums at 32 bits (300), where xezim summed at 8 bits and gave 44. A
//! narrowing cast, and `signed'` / `unsigned'`, leave the operand's own width
//! in charge. Covered on every path that evaluates a cast: the interpreter,
//! compiled continuous assigns and `always_comb`, the elaborator's constant
//! folder (including a cast in a packed dimension), and the randomize solver.
//!
//! Every expectation was cross-checked against the reference simulator.

use xezim::simulate;

fn outs(src: &str, max_time: u64) -> Vec<String> {
    let sim = simulate(src, max_time).expect("simulate failed");
    sim.output.iter().map(|o| o.message.clone()).collect()
}

fn expect_lines(o: &[String], want: &[&str]) {
    for w in want {
        assert!(o.iter().any(|l| l.trim() == *w), "missing {w:?} in {o:?}");
    }
}

#[test]
fn procedural_casts_widen_their_operand() {
    let src = r#"
typedef int myint_t;
typedef logic [15:0] u16_t;
typedef enum logic [9:0] {E0, E1, E300 = 300} e10_t;
typedef struct packed { logic [11:0] f; } s12_t;
module top;
  localparam int W = 12;
  typedef logic [3:0] nib_t;
  bit [7:0] a = 200, b = 100;
  logic [7:0] la = 200, lb = 100;
  bit signed [7:0] sa = -100, sb = -100;
  int ia = 100000, ib = 100000;
  real r = 3.3;
  function automatic int ff(bit [7:0] x, bit [7:0] y);
    return int'(x + y);
  endfunction
  initial begin
    $display("A %0d %0d %0d %0d %0d", int'(a + b), longint'(a + b), myint_t'(a + b), u16_t'(a + b), int'(la + lb));
    $display("B %0d %0d %0d %0d", int'(a * b), int'(a << 1), int'(sa + sb), shortint'(a * b));
    $display("C %0d %0d %0d %0d", int'(a > b ? a + b : a - b), int'({a} + b), int'(a + b) + 0, int'(a + b) >> 1);
    $display("D %0d %0d %0d %0d", 16'(a + b), 9'(a + b), 4'(a + b), 8'(a + b));
    $display("E %0d %0d %0d %0d", signed'(a + b), unsigned'(a + b), byte'(a + b), logic'(a + b));
    $display("F %0d %0d %0d %0d %0d", e10_t'(a + b), s12_t'(a + b), W'(a + b), nib_t'(a + b), integer'(a + b));
    $display("G %0d %0d %0d", int'(r * 2.5), int'(r), int'(a + r));
    $display("H %0d %0d %0d %0d", int'(byte'(a + b) + 1), shortint'(sa * sb), longint'(ia * ib), ff(a, b));
    $display("I %0d %0d %h", $bits(int'(a + b)), int'(a + b) == 300, int'(~a));
  end
endmodule
"#;
    let o = outs(src, 10);
    expect_lines(
        &o,
        &[
            "A 300 300 300 300 300",
            "B 20000 400 -200 20000",
            "C 300 300 300 150",
            "D 300 300 12 44",
            "E 44 44 44 0",
            "F 300 300 300 12 300",
            "G 8 3 203",
            "H 45 10000 10000000000 300",
            "I 32 1 ffffff37",
        ],
    );
}

/// The same casts on the compiled paths: continuous assigns and an
/// `always_comb`, re-evaluated when an operand changes.
#[test]
fn compiled_casts_widen_their_operand() {
    let src = r#"
typedef int myint2_t;
typedef enum logic [9:0] {F0, F1, F300 = 300} f10_t;
module top;
  localparam int W = 12;
  bit [7:0] a = 200, b = 100;
  wire [31:0] w1 = int'(a + b);
  wire [31:0] w2 = 16'(a + b);
  wire [31:0] w3 = myint2_t'(a + b);
  wire [31:0] w4 = f10_t'(a + b);
  wire [31:0] w5 = W'(a + b);
  wire [15:0] w6 = int'(a + b) >> 4;
  logic [31:0] cb;
  always_comb cb = int'(a + b) + 1;
  initial begin
    #1 $display("V %0d %0d %0d %0d %0d %0d %0d", w1, w2, w3, w4, w5, w6, cb);
    a = 250;
    #1 $display("W %0d %0d %0d %0d %0d %0d %0d", w1, w2, w3, w4, w5, w6, cb);
  end
endmodule
"#;
    let o = outs(src, 10);
    expect_lines(
        &o,
        &[
            "V 300 300 300 300 300 18 301",
            "W 350 350 350 350 350 21 351",
        ],
    );
}

/// Constant casts folded by the elaborator, including one sizing a packed
/// dimension (which left the range unresolved, one bit wide).
#[test]
fn constant_casts_widen_their_operand() {
    let src = r#"
typedef int myint3_t;
typedef logic [15:0] u16b_t;
typedef enum logic [9:0] {G0, G1, G300 = 300} g10_t;
module top;
  localparam bit [7:0] PA = 200;
  localparam bit [7:0] PB = 100;
  typedef logic [3:0] nib_t;
  localparam int L1 = int'(PA + PB);
  localparam int L2 = myint3_t'(PA + PB);
  localparam int L3 = 16'(PA + PB);
  localparam int L4 = longint'(PA + PB) + 1;
  localparam int L5 = u16b_t'(PA + PB);
  localparam int L6 = int'(PA * PB);
  localparam int L7 = 4'((PA + PB) / 3);
  localparam int L8 = g10_t'(PA + PB);
  localparam int L10 = nib_t'(PA + PB);
  localparam int L11 = 16'(PA + PB) >> 4;
  localparam int L12 = int'(PA - PB - 150);
  localparam int L13 = int'((PA - 250) / 2);
  localparam longint L14 = longint'(PA * PB * PA);
  localparam int L15 = int'(~PA);
  localparam int L16 = int'(PA > PB ? PA + PB : PB);
  localparam int L17 = signed'(PA + PB);
  localparam int L18 = int'(PA << 1);
  logic [int'(PA + PB) - 1:0] big;
  logic [16'(PA + PB) - 1:0] big2;
  initial begin
    $display("K %0d %0d %0d %0d %0d %0d %0d %0d", L1, L2, L3, L4, L5, L6, L7, L8);
    $display("L %0d %0d %0d %0d %0d %0d %0d %0d %0d", L10, L11, L12, L13, L14, L15, L16, L17, L18);
    $display("M %0d %0d", $bits(big), $bits(big2));
  end
endmodule
"#;
    let o = outs(src, 10);
    expect_lines(
        &o,
        &[
            "K 300 300 300 301 300 20000 4 300",
            "L 12 18 -50 2147483623 4000000 -201 300 44 400",
            "M 300 300",
        ],
    );
}

/// §18.5: a cast inside a constraint widens the same way — `int'(x + y) ==
/// 300` over 8-bit rand variables is satisfiable (x = 200, y = 100), and a
/// typedef cast likewise.
#[test]
fn constraint_casts_widen_their_operand() {
    let src = r#"
typedef int myint4_t;
class rc;
  rand bit [7:0] x, y;
  constraint c { x == 200; int'(x + y) == 300; }
endclass
class rc2;
  rand bit [7:0] x, y;
  constraint c { x == 250; y > 0; 16'(x + y) < 256; }
endclass
class rc3;
  rand bit [7:0] x, y;
  constraint c { x == 150; myint4_t'(x + y) == 400; }
endclass
module top;
  initial begin
    automatic rc o = new;
    automatic rc2 o2 = new;
    automatic rc3 o3 = new;
    automatic int r, bad = 0;
    r = o.randomize();
    $display("R1 %0d %0d %0d", r, o.x, o.y);
    for (int i = 0; i < 20; i++) begin
      r = o2.randomize();
      if (r != 1 || o2.y == 0 || o2.y > 5) bad++;
    end
    $display("R2 bad=%0d", bad);
    r = o2.randomize() with { int'(x + y) == 260; };
    $display("R3 %0d", r);
    r = o3.randomize();
    $display("R4 %0d %0d %0d", r, o3.x, o3.y);
  end
endmodule
"#;
    let o = outs(src, 10);
    expect_lines(&o, &["R1 1 200 100", "R2 bad=0", "R3 0", "R4 1 150 250"]);
}
