//! ivtest round 4 — four defects, all reference-validated.
//!
//! 1. **§6.19 enums declared inside a struct member's type** install their
//!    members in the scope enclosing the struct (`struct packed { enum
//!    integer { A } e; } s;` makes `A` visible) — the registration never
//!    recursed into struct members (`enum_in_struct`).
//! 2. **§6.19 X/Z-valued enum members** (`XX = 'bx`, `XZ = 32'h1x2z3xxz`):
//!    the u64 member pipeline masked the x bits to 0. The registered constant
//!    is now rebuilt as a 4-state Value when the initializer carries x/z
//!    (`enum_test1`).
//! 3. **Package variables with packed multi-D types** (`reg [1:0][7:0] y`)
//!    lacked the packed-shape maps, so `P::y[0]` read 0
//!    (`package_vec_part_select`).
//! 4. **Ports declared with an ARRAY typedef** (`typedef logic [A-1:0] T[B];
//!    input T x;`) inherit the typedef's unpacked dims — the ANSI-port path
//!    registered a scalar of the element width, so `$size(x,1)` reported the
//!    packed width (`module_port_typedef_array1`). Root cause was twofold:
//!    the port arm never consulted `typedef_unpacked_dims`, and the
//!    capture-time dim fold skipped `[B]` because a parameter-named dim
//!    parses as an ASSOCIATIVE dimension keyed by "type B".

use xezim::simulate;

fn u(sim: &xezim::compiler::Simulator, n: &str) -> u64 {
    sim.get_signal(n)
        .or_else(|| sim.get_signal(&format!("test.{}", n)))
        .unwrap_or_else(|| panic!("signal not found: {}", n))
        .to_u64()
        .unwrap_or_else(|| panic!("{} not u64-able (x/z?)", n))
}

/// Enum inside a struct member type: members visible in the enclosing scope.
#[test]
fn enum_inside_struct_member_installs_members() {
    let src = r#"
module test;
  struct packed {
    enum integer { SA = 3, SB } e;
  } s;
  int va, vb, eq;
  initial begin
    s.e = SA;
    va = SA; vb = SB;
    eq = (s.e == SA);
  end
endmodule
"#;
    let sim = simulate(src, 20).expect("simulate failed");
    assert_eq!(u(&sim, "va"), 3);
    assert_eq!(u(&sim, "vb"), 4, "auto-increment continues");
    assert_eq!(u(&sim, "eq"), 1);
}

/// X/Z-valued enum members keep their 4-state pattern.
#[test]
fn enum_members_with_xz_values() {
    let src = r#"
module test;
  enum integer { IDLE, XX = 'bx, XY = 'b01, YY = 'b10, XZ = 32'h1x2z3xxz } ns;
  int xx_is_x, xz_exact, xy_v;
  initial begin
    xx_is_x  = (XX === 'bx);
    xz_exact = (XZ === 32'h1x2z3xxz);
    xy_v     = XY;
  end
endmodule
"#;
    let sim = simulate(src, 20).expect("simulate failed");
    assert_eq!(u(&sim, "xx_is_x"), 1, "XX === 'bx");
    assert_eq!(u(&sim, "xz_exact"), 1, "the full x/z pattern survives");
    assert_eq!(u(&sim, "xy_v"), 1, "plain members unchanged");
}

/// Package vars: part-selects and packed-2D element selects through P::.
#[test]
fn package_var_selects() {
    let src = r#"
package P;
  reg [7:0] x = 8'h5a;
  reg [1:0][7:0] y = 16'h5af0;
endpackage
module test;
  int lo, hi, y0, y1;
  initial begin
    lo = P::x[3:0]; hi = P::x[7:4];
    y0 = P::y[0]; y1 = P::y[1];
  end
endmodule
"#;
    let sim = simulate(src, 20).expect("simulate failed");
    assert_eq!(u(&sim, "lo"), 0xA);
    assert_eq!(u(&sim, "hi"), 0x5);
    assert_eq!(u(&sim, "y0"), 0xF0, "packed-2D element through P::");
    assert_eq!(u(&sim, "y1"), 0x5A);
}

/// A port declared with an array typedef gets the typedef's unpacked shape.
#[test]
fn typedef_array_port_inherits_unpacked_dims() {
    let src = r#"
localparam A = 2;
localparam B = 4;
typedef logic [A-1:0] T[B];
module test (input T x);
  int s1, s2, b, d;
  initial begin
    s1 = $size(x, 1);
    s2 = $size(x, 2);
    b  = $bits(x);
    d  = $dimensions(x);
  end
endmodule
"#;
    let sim = simulate(src, 20).expect("simulate failed");
    assert_eq!(u(&sim, "s1"), 4, "dim 1 is the unpacked [B]");
    assert_eq!(u(&sim, "s2"), 2, "dim 2 is the packed [A-1:0]");
    assert_eq!(u(&sim, "b"), 8, "4 elements x 2 bits");
    assert_eq!(u(&sim, "d"), 2);
}

/// §7.4.2: packed dims AFTER an enum body (`enum {...} [1:0] x;`) make a
/// packed array of the enum — mirroring the struct body-suffix form
/// (ivtest `array_packed`).
#[test]
fn enum_body_suffix_packed_dims() {
    let src = r#"
module test;
  typedef enum logic [7:0] { A } E;
  E [1:0] ep2;
  enum logic [7:0] { B } [1:0] ep3;
  int b2, b3;
  initial begin
    b2 = $bits(ep2);
    b3 = $bits(ep3);
  end
endmodule
"#;
    let sim = simulate(src, 20).expect("simulate failed");
    assert_eq!(u(&sim, "b2"), 16, "typedef'd enum with packed dims");
    assert_eq!(u(&sim, "b3"), 16, "anonymous enum with body-suffix dims");
}

/// §7.2.1: a packed-struct member read carries the member's DECLARED
/// signedness — the slice itself is a raw bit pattern (ivtest
/// `struct_packed_sysfunct2`: `%0d` of an `int` member printed unsigned).
#[test]
fn struct_member_reads_keep_declared_signedness() {
    let src = r#"
module test;
  struct packed { int s; int unsigned u; } x;
  int neg, via_int, both_u;
  initial begin
    x.s = -20;
    x.u = -10;
    neg     = (x.s < 0);
    via_int = x.s;
    both_u  = (x.u > 0);
  end
endmodule
"#;
    let sim = simulate(src, 20).expect("simulate failed");
    assert_eq!(u(&sim, "neg"), 1, "int member compares signed");
    assert_eq!(u(&sim, "via_int") as u32 as i32, -20);
    assert_eq!(u(&sim, "both_u"), 1, "unsigned member stays unsigned");
}

/// §7.4.1 ASCENDING packed dims (`[0:N-1]`) mirror slot order — element 0 is
/// the TOP slot, and within an ascending element type bit label 0 is the MSB
/// (ivtest `br884` unaligned packed-array access + `struct10`'s `bar` half).
/// Three layers were unmirrored: element offsets in single-element writes,
/// inner bit labels in `arr[i][m:l]` writes, and single-dim ascending STRUCT
/// MEMBERS (`bit [0:15] b; b[14:15]` is the LOW two bits) — the last guarded
/// to single-dim members only, since element scaling already normalizes
/// multi-dim ones.
#[test]
fn ascending_packed_dims_mirror_slots() {
    let src = r#"
module test;
  logic [0:3][3:0] lt;
  struct packed { bit [0:7][7:0] a; bit [0:15] b; } bar;
  logic [15:0] lt_after_e0, lt_after_range, bar_b1, bar_b2;
  logic [63:0] bar_a1, bar_a2;
  initial begin
    lt = '0; lt[0] = 4'hF;            // ascending: element 0 = TOP nibble
    lt_after_e0 = lt;
    lt = '0; lt[0][1:0] = 2'b11;      // inner [3:0] descending: low 2 bits of the top nibble
    lt_after_range = lt;
    bar = '0; bar.b = '1; bar.b[14:15] = '0;  // ascending member: labels 14,15 = LOW bits
    bar_b1 = bar.b;
    bar = '0; bar.b = '1; bar.b[0:1] = '0;    // labels 0,1 = TOP bits
    bar_b2 = bar.b;
    bar = '0; bar.a[5:6] = 16'h1234;  // ascending elements 5,6 = slots 2,1
    bar_a1 = bar.a;
    bar = '0; bar.a[2] = 8'h42;       // element 2 = slot 5
    bar_a2 = bar.a;
  end
endmodule
"#;
    let sim = simulate(src, 20).expect("simulate failed");
    assert_eq!(u(&sim, "lt_after_e0"), 0xF000, "element 0 is the top slot");
    assert_eq!(
        u(&sim, "lt_after_range"),
        0x3000,
        "inner labels stay descending"
    );
    assert_eq!(u(&sim, "bar_b1"), 0xFFFC, "labels 14:15 are the low bits");
    assert_eq!(u(&sim, "bar_b2"), 0x3FFF, "labels 0:1 are the top bits");
    assert_eq!(
        u(&sim, "bar_a1"),
        0x0000_0000_0012_3400,
        "element range mirrors"
    );
    assert_eq!(
        u(&sim, "bar_a2"),
        0x0000_4200_0000_0000,
        "single element mirrors"
    );
}

/// §6.16: string equality is by TEXT and 2-state — an out-of-bounds read of a
/// string queue/dynamic array stores x bits whose text is "", and the compare
/// against "" must be 1, not X (ivtest `sv_queue_oob_string`,
/// `sv_darray_oob_string`).
#[test]
fn oob_string_collection_reads_compare_empty() {
    let src = r#"
module test;
  string q[$];
  string d[];
  string x, y;
  int ex, ey, ne;
  initial begin
    x = q[1];
    d = new[1];
    y = d[5];
    ex = (x == "");
    ey = (y == "");
    ne = (x != "abc");
  end
endmodule
"#;
    let sim = simulate(src, 20).expect("simulate failed");
    assert_eq!(
        u(&sim, "ex"),
        1,
        "oob queue element compares equal to empty"
    );
    assert_eq!(u(&sim, "ey"), 1, "oob dynamic-array element too");
    assert_eq!(u(&sim, "ne"), 1, "!= is 2-state as well");
}

/// §23.2.2 / A.1.2: module-HEADER imports precede the parameter list and are
/// usable in it (ivtest `mod_inst_pkg`): `module m import P::*; #(parameter
/// X = FOO)` must see the package's FOO. Tested in the TOP-module form — the
/// strict reference rejects an imported function in a port range outright
/// ("external function may not be used in a constant expression"), so the
/// lenient acceptance itself is the pinned behavior, and the instantiated
/// form's port sizing remains a known gap.
#[test]
fn header_imports_feed_parameter_defaults() {
    let src = r#"
package fooPkg;
  localparam FOO = 5;
endpackage
package barPkg;
  function int get_size(input int x); return x + 3; endfunction
endpackage
module test import fooPkg::*, barPkg::*; #(parameter P = FOO) (input [get_size(7)-1:0] ip);
  int pv, bv;
  initial begin
    pv = P;
    bv = $bits(ip);
  end
endmodule
"#;
    let sim = simulate(src, 20).expect("simulate failed");
    assert_eq!(
        u(&sim, "pv"),
        5,
        "the imported FOO reaches the parameter default"
    );
    assert_eq!(u(&sim, "bv"), 10, "the imported function sizes the port");
}

/// §6.19.2/§6.19.6: an enum's state-ness follows its BASE type — a 2-state
/// base defaults the variable to 0 (4-state to x), and next/prev of an
/// INVALID value returns that same default (ivtest `pr3366217i`).
#[test]
fn enum_default_and_invalid_next_follow_base_type() {
    let src = r#"
module top;
  enum bit [3:0] {a2 = 1, b2 = 2} evar2;
  enum reg [3:0] {a4 = 1, b4 = 2} evar4;
  int i2, x4, n2, n4;
  initial begin
    i2 = (evar2 === 0);
    x4 = (evar4 === 4'bx);
    evar2 = evar2.next; n2 = (evar2 === 0);
    evar4 = evar4.next; n4 = (evar4 === 4'bx);
  end
endmodule
"#;
    let sim = simulate(src, 20).expect("simulate failed");
    assert_eq!(u(&sim, "i2"), 1, "2-state base defaults to 0");
    assert_eq!(u(&sim, "x4"), 1, "4-state base defaults to x");
    assert_eq!(
        u(&sim, "n2"),
        1,
        "next of invalid stays at the 2-state default"
    );
    assert_eq!(u(&sim, "n4"), 1, "next of invalid stays x for 4-state");
}

/// §3.14.2.3: `timeunit`/`timeprecision` at COMPILATION-UNIT scope apply to
/// modules with no timescale of their own — previously parsed and DROPPED, so
/// fractional delays truncated to the 1 ns default tick and `#78.1ps`
/// collapsed to 0 (ivtest `test_tliteral`).
#[test]
fn unit_scope_timeunit_declarations_apply() {
    let src = r#"
timeunit 1ns;
timeprecision 10ps;
module test;
  parameter factor = 1e-9/10e-12;
  longint t1, t2;
  initial begin
    #33.1ns;
    t1 = $realtime*factor;
    #78.1ps;
    t2 = $realtime*factor;
  end
endmodule
"#;
    let sim = simulate(src, 200).expect("simulate failed");
    assert_eq!(u(&sim, "t1"), 3310, "33.1ns lands on the 10ps grid");
    assert_eq!(u(&sim, "t2"), 3318, "78.1ps rounds to 8 ticks more");
}

/// §13.4.3 constant functions over REALS (ivtest `cfunc_assign_op_real`):
/// four faces of one theme — the const-function evaluator was integer-only.
/// Formals now convert to their declared type (`input real x` bound to `5`),
/// real results survive the return/substitution (they were re-literalized via
/// to_i64), `eval_init_for_width` no longer bit-reinterprets reals through
/// resize, and ++/-- are real-aware at both interpreter arms.
#[test]
fn constant_functions_over_reals() {
    let src = r#"
module test;
  function real f_div(input real x);
    begin x /= 2; f_div = x; end
  endfunction
  function real f_inc(input real x);
    begin ++x; f_inc = x; end
  endfunction
  function int f_int(input int x);
    f_int = x + 1;
  endfunction
  localparam D5 = f_div(5);
  localparam I5 = f_inc(5);
  localparam N5 = f_int(5);
  int d_ok, i_ok, n_ok;
  initial begin
    d_ok = (D5 == 2.5);
    i_ok = (I5 == 6.0);
    n_ok = (N5 == 6);
  end
endmodule
"#;
    let sim = simulate(src, 20).expect("simulate failed");
    assert_eq!(u(&sim, "d_ok"), 1, "real division keeps the fraction");
    assert_eq!(u(&sim, "i_ok"), 1, "++ on a real formal");
    assert_eq!(u(&sim, "n_ok"), 1, "integer functions unchanged");
}
