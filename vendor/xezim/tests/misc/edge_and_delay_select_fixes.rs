//! Three related fixes found chasing a full-SoC CoreMark hang, all in the
//! "select expression where a whole signal was assumed" family:
//!
//! 1. A DELAYED continuous assign with a range-select LHS
//!    (`assign #1 d[8:0] = a[8:0];`) was silently dropped — the delayed
//!    scheduler resolved bare-Ident LHS only, and an `if let` swallowed the
//!    write. An AXI bridge FSM whose next-state ran through exactly this
//!    idiom stayed z forever, its rvalid X-poisoned the bus arbiter's
//!    priority chain, and the first speculative read wedged the whole read
//!    return path.
//! 2. An edge clock that SELECTS into a signal (`posedge vec[1]`, or bit j
//!    of unpacked element i via an inlined `.clk(arr[i][j])` port
//!    connection) degraded to the base signal's LSB — or was dropped
//!    outright for an unpacked base. Now rewritten to a synthesized 1-bit
//!    alias net at elaboration.
//! 3. A cont-assign RESOLVING a constant net during the time-0 settle
//!    (`wire one = 1'b1;`, X→1) never woke `always @(one)` blocks: the edge
//!    baseline snapshot was taken AFTER the settle. Decode ROMs written as
//!    `always @(const_bit)` never initialized.

use xezim::simulate;

fn u(sim: &xezim::compiler::Simulator, n: &str) -> u64 {
    sim.get_signal(n)
        .or_else(|| sim.get_signal(&format!("tb.{}", n)))
        .unwrap_or_else(|| panic!("signal not found: {}", n))
        .to_u64()
        .unwrap_or_else(|| panic!("{} has x/z bits", n))
}

/// Delayed cont-assigns with full-range, partial-range, and bit-select
/// targets must all deliver (transport the value after the delay).
#[test]
fn delayed_assign_select_lhs_delivers() {
    let src = r#"
module tb;
  reg [8:0] a = 9'b000000001;
  wire [8:0] d_bare, d_rng;
  wire [7:0] d_part;
  wire d_bit;
  assign #1 d_bare = a;
  assign #1 d_rng[8:0] = a[8:0];
  assign #1 d_part[7:0] = a[8:1];
  assign #1 d_bit = a[7];
  reg [31:0] got_rng, got_part;
  reg got_bit;
  initial begin
    #5 a = 9'b010000000;
    #10;
    got_rng  = d_rng;
    got_part = d_part;
    got_bit  = d_bit;
  end
endmodule
"#;
    let sim = simulate(src, 100).expect("simulate failed");
    assert_eq!(u(&sim, "got_rng"), 0b010000000, "full-range select LHS");
    assert_eq!(u(&sim, "got_part"), 0b01000000, "partial-range select LHS");
    assert_eq!(u(&sim, "got_bit"), 1, "bit-select RHS through bare LHS");
}

/// A flop clocked by a BIT of a packed vector, and by bit j of unpacked
/// element i, through a child-module port connection — the shapes the
/// edge-select alias pass rewrites.
#[test]
fn select_expression_gated_clocks_latch() {
    let src = r#"
module reg_cell(input clk, input rst_b, input [7:0] d, output reg [7:0] q);
  always @(posedge clk or negedge rst_b)
    if (!rst_b) q <= '0; else q <= d;
endmodule
module tb;
  reg base_clk = 0; always #5 base_clk = ~base_clk;
  reg rst_b = 0;
  reg en_vec = 0, en_arr = 0;
  wire [1:0] vclk;
  wire [1:0] aclk [1:0];
  assign vclk[1] = base_clk & en_vec;
  assign aclk[1][0] = base_clk & en_arr;
  reg [7:0] din = 8'h00;
  wire [7:0] q_vec, q_arr;
  reg_cell c_v (.clk(vclk[1]),    .rst_b(rst_b), .d(din), .q(q_vec));
  reg_cell c_a (.clk(aclk[1][0]), .rst_b(rst_b), .d(din), .q(q_arr));
  reg [7:0] got_vec, got_arr;
  initial begin
    #12 rst_b = 1;
    din = 8'h5A; en_vec = 1; en_arr = 1;
    #20;
    got_vec = q_vec; got_arr = q_arr;
  end
endmodule
"#;
    let sim = simulate(src, 100).expect("simulate failed");
    assert_eq!(u(&sim, "got_vec"), 0x5A, "posedge of a packed-vector bit");
    assert_eq!(
        u(&sim, "got_arr"),
        0x5A,
        "posedge of bit 0 of unpacked element 1"
    );
}

/// `always @(one)` where `one` is a constant net resolved by the t0 settle
/// must fire once — the decode block writes its constant pattern via
/// parameter-indexed bit assigns.
#[test]
fn const_net_edge_fires_at_time_zero() {
    let src = r#"
module tb;
  parameter B0 = 0;
  parameter B2 = 2;
  parameter B6 = 6;
  wire one = 1'b1;
  reg [16:0] dep;
  always @(one) begin
    dep[16:0] = {17{1'b0}};
    dep[B0] = one;
    dep[B2] = one;
    dep[B6] = one;
  end
  reg [16:0] got;
  initial begin
    #10 got = dep;
  end
endmodule
"#;
    let sim = simulate(src, 100).expect("simulate failed");
    assert_eq!(
        u(&sim, "got"),
        0b00000000001000101,
        "const-net level block must evaluate at t0"
    );
}
