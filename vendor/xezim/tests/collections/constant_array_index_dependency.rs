//! A CONSTANT array subscript depends on exactly one element.
//!
//! `collect_expr_reads` registered a read of `arr[idx]` against EVERY element
//! of `arr`, unconditionally — correct, but quadratic. After generate
//! unrolling an index like `q[(c + d) % N]` folds to a literal, yet each such
//! reader was still recorded against all N elements. On a 128-column fabric
//! that gave every 1-bit element 1664 dependent comb entries where the design
//! connects about 13, so one bit changing dragged ~128x more entries into the
//! settle worklist than necessary. The 2-D path (`m[i][j]`) had the same shape:
//! every read expanded to the whole array.
//!
//! Folding both subscripts shrank the dependency graph on a synthetic
//! gate-level benchmark from 2,601,985 edges to 32,897 (79x) and cut its
//! runtime 4.3x, with byte-identical results.
//!
//! The risk in narrowing a read set is dropping a dependency and leaving a
//! signal stale, so these tests pin the behaviour from both sides: constant
//! indices must still propagate, and a genuinely DYNAMIC index must still
//! depend on every element.

use xezim::simulate;

fn get(sim: &xezim::compiler::Simulator, name: &str) -> u64 {
    sim.get_signal(name)
        .or_else(|| sim.get_signal(&format!("tb.{}", name)))
        .unwrap_or_else(|| panic!("signal not found: {}", name))
        .to_u64()
        .unwrap_or_else(|| panic!("signal {} not u64-able", name))
}

/// Constant subscripts — including ones that only become constant after
/// parameter/genvar folding — must still re-propagate when the element moves.
const CONST_IDX: &str = r#"
module tb;
  localparam int N = 4;
  logic [7:0] arr  [N];
  logic [7:0] grid [N][2];
  wire  [7:0] w_lit, w_expr, w_mod, w_2d;
  assign w_lit  = arr[2];
  assign w_expr = arr[1 + 1];          // folds to 2
  assign w_mod  = arr[(3 + 3) % N];    // folds to 2
  assign w_2d   = grid[(1 + 2) % N][1];

  logic [7:0] a1, a2, a3, a4, b1, b2, b3, b4;
  initial begin
    arr[2] = 8'h11;
    grid[3][1] = 8'h22;
    #1;
    a1 = w_lit; a2 = w_expr; a3 = w_mod; a4 = w_2d;
    // Move them again: a narrowed read set must still carry the update.
    arr[2] = 8'hAA;
    grid[3][1] = 8'hBB;
    #1;
    b1 = w_lit; b2 = w_expr; b3 = w_mod; b4 = w_2d;
  end
endmodule
"#;

#[test]
fn constant_subscripts_still_propagate() {
    let sim = simulate(CONST_IDX, 100).expect("simulate failed");
    assert_eq!(get(&sim, "a1") & 0xFF, 0x11);
    assert_eq!(get(&sim, "a2") & 0xFF, 0x11);
    assert_eq!(get(&sim, "a3") & 0xFF, 0x11);
    assert_eq!(get(&sim, "a4") & 0xFF, 0x22);
    assert_eq!(get(&sim, "b1") & 0xFF, 0xAA);
    assert_eq!(get(&sim, "b2") & 0xFF, 0xAA);
    assert_eq!(get(&sim, "b3") & 0xFF, 0xAA);
    assert_eq!(get(&sim, "b4") & 0xFF, 0xBB);
}

/// A dynamic subscript must keep depending on every element: whichever one
/// changes has to re-fire the reader, and changing the SELECTOR must too.
///
/// NOTE: only the 1-D form is asserted here. A DYNAMIC subscript on a
/// multi-dimensional array (`grid[sel][1]` in a continuous assign or an
/// `always @(*)`) still reads x — a pre-existing gap, unrelated to the read-set
/// narrowing this file covers: the bytecode Index arm resolves a multi-dim
/// element only when every subscript folds to a constant, and the dynamic case
/// falls through to the plain bit-select path. Confirmed against a reference
/// simulator, which returns the element. See `multi_dim_elem_read_and_param_dims`
/// for the constant-subscript half that is fixed.
const DYN_IDX: &str = r#"
module tb;
  logic [7:0] arr [4];
  logic [1:0] sel;
  wire  [7:0] w_dyn;
  assign w_dyn = arr[sel];

  logic [7:0] s0, s1, s2;
  initial begin
    arr[0] = 8'h10; arr[1] = 8'h11; arr[2] = 8'h12; arr[3] = 8'h13;
    sel = 2'd0;
    #1;
    s0 = w_dyn;
    sel = 2'd3;          // selector moves -> must re-evaluate
    #1;
    s1 = w_dyn;
    arr[3] = 8'hEE;      // the SELECTED element moves -> must re-evaluate
    #1;
    s2 = w_dyn;
  end
endmodule
"#;

#[test]
fn dynamic_subscript_still_depends_on_every_element() {
    let sim = simulate(DYN_IDX, 100).expect("simulate failed");
    assert_eq!(get(&sim, "s0") & 0xFF, 0x10);
    assert_eq!(get(&sim, "s1") & 0xFF, 0x13);
    assert_eq!(get(&sim, "s2") & 0xFF, 0xEE);
}
