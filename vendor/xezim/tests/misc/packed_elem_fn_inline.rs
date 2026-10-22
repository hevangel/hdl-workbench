//! Packed-of-packed element access on register-backed locals inside inlined
//! function bodies — three width bugs that compounded into whole-vector
//! corruption when an automatic function built its result element by element
//! (`y[i] = x[i][9:2]` on typedef'd vectors):
//!
//! 1. The inline-call return width was resolved WITHOUT the typedef table, so
//!    a typedef'd return type defaulted to 32 bits and `return y` truncated a
//!    128-bit result to its low word.
//! 2. `infer_lhs_width` treated `y[i]` on a packed-of-packed LOCAL as a 1-bit
//!    select, so the RHS was compiled at width 1 and the element splice wrote
//!    the value's LSB only.
//! 3. A typedef declared in an instantiated SUBMODULE recorded its total
//!    width but not its packed ELEMENT width, so the splice/extract paths
//!    never engaged and the whole inline bailed to the degraded 1-bit path.
//!
//! The continuous-assign form is the sharpest probe: the same function called
//! through the AST interpreter (e.g. with a `$display` inside, which blocks
//! inlining) was CORRECT, so the truncation only showed on the compiled path.

use xezim::simulate;

fn bit(sim: &xezim::compiler::Simulator, n: &str) -> char {
    let v = sim
        .get_signal(n)
        .or_else(|| sim.get_signal(&format!("tb.{}", n)))
        .unwrap_or_else(|| panic!("signal not found: {}", n));
    match v.get_bit(0) {
        xezim_core::value::LogicBit::Zero => '0',
        xezim_core::value::LogicBit::One => '1',
        xezim_core::value::LogicBit::X => 'x',
        xezim_core::value::LogicBit::Z => 'z',
    }
}

/// Package typedefs, 16-lane 10→8 narrowing through a continuous assign.
/// Exercises the inlined return width (128, not 32), the element-write width
/// (8, not 1), and element writes past bit 32.
#[test]
fn lane_narrowing_fn_through_cont_assign() {
    let src = r#"
package lane_types_pkg;
  typedef logic [7:0] lane8_t;
  typedef logic [9:0] lane10_t;
  typedef lane8_t  [15:0] lane8_vec_t;
  typedef lane10_t [15:0] lane10_vec_t;
endpackage
module lane_repack_unit(input lane_types_pkg::lane10_vec_t wide_in,
                        output lane_types_pkg::lane8_vec_t narrow_out);
  import lane_types_pkg::*;
  function automatic lane8_vec_t narrow_lanes(input lane10_vec_t x);
    lane8_vec_t y;
    for (int i = 0; i < 16; ++i) begin
      y[i] = x[i][9:2];
    end
    return y;
  endfunction
  assign narrow_out = narrow_lanes(wide_in);
endmodule
module tb;
  import lane_types_pkg::*;
  lane10_vec_t wide_in;
  lane8_vec_t narrow_out;
  logic ok_low, ok_high;
  lane_repack_unit u0(.wide_in, .narrow_out);
  initial begin
    wide_in = '0;
    wide_in[0]  = 10'h1da;   // >> 2 = 8'h76
    wide_in[1]  = 10'h1a0;   // >> 2 = 8'h68
    wide_in[2]  = 10'h200;   // >> 2 = 8'h80
    wide_in[15] = 10'h3ff;   // >> 2 = 8'hff (element past bit 32)
    #2;
    ok_low  = (narrow_out[2:0] == 24'h806876);
    ok_high = (narrow_out[15] == 8'hff);
  end
endmodule
"#;
    let sim = simulate(src, 10).expect("simulate failed");
    assert_eq!(
        bit(&sim, "ok_low"),
        '1',
        "low lanes must narrow at full 8-bit width"
    );
    assert_eq!(
        bit(&sim, "ok_high"),
        '1',
        "lane 15 lands past bit 32 of the result"
    );
}

/// Same shape with the typedefs declared INSIDE the instantiated submodule:
/// the element width must resolve through the instance-scoped typedef key.
#[test]
fn submodule_local_typedef_elem_write() {
    let src = r#"
module byte_fill_unit(input logic [159:0] raw_in, output logic [127:0] filled_out);
  typedef logic [7:0] b8_t;
  typedef logic [9:0] b10_t;
  typedef b8_t  [15:0] b8v_t;
  typedef b10_t [15:0] b10v_t;
  function automatic b8v_t fill_lanes(input b10v_t x);
    b8v_t y;
    for (int i = 0; i < 16; ++i) begin
      y[i] = 8'hAB;
    end
    return y;
  endfunction
  assign filled_out = fill_lanes(b10v_t'(raw_in));
endmodule
module tb;
  logic [159:0] raw_in = '0;
  logic [127:0] filled_out;
  logic ok;
  byte_fill_unit u0(.raw_in, .filled_out);
  initial begin
    #2;
    ok = (filled_out == {16{8'hAB}});
  end
endmodule
"#;
    let sim = simulate(src, 10).expect("simulate failed");
    assert_eq!(
        bit(&sim, "ok"),
        '1',
        "all 16 elements must be written 8 bits wide"
    );
}

/// Blocking-call form in an initial block: element-by-element identity and
/// +1 transforms, a single element write into a zeroed local, and an element
/// read of a literal-bound formal. These pinned the AST-layer twin of the
/// same degradation (packed-of-packed locals/formals collapsing to 1-bit
/// selects).
#[test]
fn packed_elem_fn_blocking_calls() {
    let src = r#"
package word_types_pkg;
  typedef logic [7:0] oct_t;
  typedef oct_t [3:0] oct_vec_t;
endpackage
module tb;
  import word_types_pkg::*;
  function automatic oct_vec_t copy_elems(input oct_vec_t x);
    oct_vec_t y;
    for (int i = 0; i < 4; ++i) begin
      y[i] = x[i];
    end
    return y;
  endfunction
  function automatic oct_vec_t bump_elems(input oct_vec_t x);
    oct_vec_t y;
    for (int i = 0; i < 4; ++i) begin
      y[i] = x[i] + 8'd1;
    end
    return y;
  endfunction
  function automatic oct_vec_t mark_one();
    oct_vec_t y;
    y = '0;
    y[1] = 8'hAB;
    return y;
  endfunction
  function automatic oct_t pick_third(input oct_vec_t x);
    return x[2];
  endfunction
  oct_vec_t a, r_copy, r_bump, r_mark;
  oct_t r_pick;
  logic ok_copy, ok_bump, ok_mark, ok_pick;
  initial begin
    a = 32'hDEADBEEF;
    r_copy = copy_elems(a);
    r_bump = bump_elems(a);
    r_mark = mark_one();
    r_pick = pick_third(32'hDEADBEEF);
    ok_copy = (r_copy == 32'hDEADBEEF);
    ok_bump = (r_bump == 32'hDFAEBFF0);
    ok_mark = (r_mark == 32'h0000AB00);
    ok_pick = (r_pick == 8'hAD);
  end
endmodule
"#;
    let sim = simulate(src, 10).expect("simulate failed");
    assert_eq!(bit(&sim, "ok_copy"), '1', "element identity copy");
    assert_eq!(
        bit(&sim, "ok_bump"),
        '1',
        "element read + arithmetic + write"
    );
    assert_eq!(
        bit(&sim, "ok_mark"),
        '1',
        "single element write into zeroed local"
    );
    assert_eq!(
        bit(&sim, "ok_pick"),
        '1',
        "element read of a literal-bound formal"
    );
}

/// Widening direction (`{x[i], 2'b00}` into 10-bit lanes) plus a mode ternary
/// choosing between raw passthrough and the conversion — the full shape of
/// the design that exposed the truncation, checker and DUT computed by two
/// different paths.
#[test]
fn lane_widening_with_mode_select() {
    let src = r#"
package lane_types2_pkg;
  typedef logic [7:0] lane8_t;
  typedef logic [9:0] lane10_t;
  typedef lane8_t  [15:0] lane8_vec_t;
  typedef lane10_t [15:0] lane10_vec_t;
endpackage
module lane_widen_unit(input lane_types2_pkg::lane8_vec_t narrow_in,
                       input logic bypass,
                       output lane_types2_pkg::lane10_vec_t wide_out);
  import lane_types2_pkg::*;
  function automatic lane10_vec_t widen_lanes(input lane8_vec_t x);
    lane10_vec_t y;
    for (int i = 0; i < 16; ++i) begin
      y[i] = {x[i], 2'b00};
    end
    return y;
  endfunction
  assign wide_out = bypass ? {32'h0, narrow_in} : widen_lanes(narrow_in);
endmodule
module tb;
  import lane_types2_pkg::*;
  lane8_vec_t narrow_in;
  logic bypass;
  lane10_vec_t wide_out;
  logic ok_conv, ok_byp;
  lane_widen_unit u0(.narrow_in, .bypass, .wide_out);
  initial begin
    narrow_in = '0;
    narrow_in[0]  = 8'h5a;
    narrow_in[15] = 8'h81;
    bypass = 1'b0;
    #2;
    ok_conv = (wide_out[0] == 10'h168) && (wide_out[15] == 10'h204);
    bypass = 1'b1;
    #2;
    ok_byp = (wide_out == {32'h0, narrow_in});
  end
endmodule
"#;
    let sim = simulate(src, 10).expect("simulate failed");
    assert_eq!(
        bit(&sim, "ok_conv"),
        '1',
        "widened lanes at both ends of the vector"
    );
    assert_eq!(
        bit(&sim, "ok_byp"),
        '1',
        "bypass mode passes the raw vector through"
    );
}
