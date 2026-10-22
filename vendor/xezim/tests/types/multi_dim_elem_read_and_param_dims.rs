//! Two defects found by running a synthetic gate-level benchmark against a
//! reference simulator. Neither involves any design IP — both reproduce in a
//! dozen lines.
//!
//! 1. READING a multi-dimensional unpacked array element in a CONTINUOUS
//!    assign (or an `always @(*)`) returned x. The bytecode compiler's Index
//!    arm only recognised `Ident[i]`; for `grid[i][j]` the base of the outer
//!    Index is itself an Index, so nothing matched and it fell through to the
//!    plain BIT-SELECT path — `grid[1][2]` compiled to "bit 2 of bit 1 of the
//!    array's base signal". Procedural reads went down a different path and
//!    were always correct, which is what made this hard to spot.
//!
//! 2. A bare identifier as an UNPACKED DIMENSION (`wire w [COLS];`) was parsed
//!    as an ASSOCIATIVE array and fell back to a 64-entry dynamic array.
//!    `[NAME]` is genuinely ambiguous in the grammar (§A.2.5) — associative
//!    when NAME is a type, sized when NAME is a parameter — and the parser has
//!    no type table, so it committed to associative. A 2-D `wire w [COLS][DEP]`
//!    then had the wrong shape: elements past the mis-sized bound did not
//!    exist, so instances bound to `w[i][j]` drove nothing and the net read z.
//!    Resolved during elaboration instead, where the parameter table exists.
//!    `[COLS-0]`, `[0:COLS-1]` and `[4]` always worked.
//!
//! All expectations reference-simulator verified.

use xezim::simulate;

fn get(sim: &xezim::compiler::Simulator, name: &str) -> u64 {
    sim.get_signal(name)
        .or_else(|| sim.get_signal(&format!("tb.{}", name)))
        .unwrap_or_else(|| panic!("signal not found: {}", name))
        .to_u64()
        .unwrap_or_else(|| panic!("signal {} not u64-able", name))
}

/// (1) continuous / comb reads of a 2-D unpacked element.
const ELEM_READ: &str = r#"
module tb;
  logic [7:0] grid [2][3];
  logic [7:0] lin  [3];          // 1-D control: this path always worked
  wire  [7:0] ca_2d, ca_1d;
  logic       idx_1d = 1'b0;
  wire  [7:0] dyn_1d;
  wire        bit_2d;
  logic [7:0] comb_2d;
  assign ca_2d  = grid[1][2];
  assign ca_1d  = lin[2];
  assign dyn_1d = lin[idx_1d];
  assign bit_2d = grid[1][2][0];
  always @(*) comb_2d = grid[1][2];
  logic [7:0] seen_ca, seen_comb, seen_1d;
  logic       seen_bit;
  logic [7:0] seen_ca2;
  logic [7:0] seen_dyn_1d;
  initial begin
    lin[0]     = 8'h11;
    lin[1]     = 8'h22;
    grid[1][2] = 8'hA5;
    lin[2]     = 8'hA5;
    #1;
    seen_ca   = ca_2d;
    seen_comb = comb_2d;
    seen_1d   = ca_1d;
    seen_bit  = bit_2d;
    grid[1][2] = 8'h5A;          // must re-propagate, not latch
    idx_1d = 1'b1;
    #1;
    seen_ca2  = ca_2d;
    seen_dyn_1d = dyn_1d;
  end
endmodule
"#;

#[test]
fn multi_dim_element_read_in_a_continuous_assign() {
    let sim = simulate(ELEM_READ, 100).expect("simulate failed");
    assert_eq!(get(&sim, "seen_ca") & 0xFF, 0xA5);
    assert_eq!(get(&sim, "seen_comb") & 0xFF, 0xA5);
    assert_eq!(get(&sim, "seen_1d") & 0xFF, 0xA5);
    assert_eq!(get(&sim, "seen_bit") & 1, 1);
    assert_eq!(get(&sim, "seen_ca2") & 0xFF, 0x5A);
    assert_eq!(get(&sim, "seen_dyn_1d") & 0xFF, 0x22);
}

/// (2) parameter-named unpacked dimensions, and instances bound to elements of
/// a 2-D net array declared that way.
const PARAM_DIMS: &str = r#"
module inv_cell (input wire a, output wire z);
  assign z = ~a;
endmodule

module tb;
  localparam int COLS = 4, DEP = 2;
  wire  by_ident [COLS];         // the ambiguous form
  wire  by_expr  [COLS-0];       // always worked
  wire  by_range [0:COLS-1];     // always worked
  wire  by_lit   [4];            // always worked
  wire  grid2d   [COLS][DEP];    // 2-D from parameters
  logic src = 1'b0;

  genvar c;
  generate
    for (c = 0; c < COLS; c++) begin : col
      inv_cell u (.a(src), .z(grid2d[c][0]));
    end
  endgenerate

  int w_ident, w_expr, w_range, w_lit;
  logic [3:0] driven;            // one bit per column of grid2d[c][0]
  initial begin
    #1;
    w_ident = $size(by_ident);
    w_expr  = $size(by_expr);
    w_range = $size(by_range);
    w_lit   = $size(by_lit);
    driven  = {grid2d[3][0], grid2d[2][0], grid2d[1][0], grid2d[0][0]};
  end
endmodule
"#;

#[test]
fn parameter_named_unpacked_dimension_is_a_size_not_an_associative_key() {
    let sim = simulate(PARAM_DIMS, 100).expect("simulate failed");
    assert_eq!(get(&sim, "w_ident"), 4); // was 64
    assert_eq!(get(&sim, "w_expr"), 4);
    assert_eq!(get(&sim, "w_range"), 4);
    assert_eq!(get(&sim, "w_lit"), 4);
    // Every column's instance drives its element; before the fix the columns
    // past the mis-sized bound read z.
    assert_eq!(get(&sim, "driven") & 0xF, 0xF);
}

/// §9.2.2.2: a reader of a fixed N-dimensional element is sensitive to that
/// element, just like the established two-dimensional case above.
#[test]
fn three_dimensional_element_change_reaches_comb_reader() {
    let src = r#"
module tb;
  logic [7:0] stimulus = '0;
  logic [7:0] volume [0:0][0:0][0:0];
  logic [7:0] result;

  always_comb volume[0][0][0] = stimulus;
  assign result = volume[0][0][0] ^ 8'hA5;

  initial begin
    #1 stimulus = 8'h11;
    #1 stimulus = 8'h22;
    #1 stimulus = 8'h33;
    #1;
  end
endmodule
"#;
    let sim = simulate(src, 20).expect("simulate failed");
    assert_eq!(get(&sim, "volume[0][0][0]") & 0xFF, 0x33);
    assert_eq!(get(&sim, "result") & 0xFF, 0x96);
}
