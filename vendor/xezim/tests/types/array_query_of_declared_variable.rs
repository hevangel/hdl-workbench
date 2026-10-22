//! IEEE 1800-2023 §20.6.2 / §20.7: `$bits` and the array query functions
//! (`$size`, `$dimensions`, `$unpacked_dimensions`, `$left`, `$right`,
//! `$low`, `$high`, `$increment`) are constant functions of a variable's
//! DECLARED type when that type is fixed-size, so they may size a
//! `localparam`, a parameter default or override, or a packed width.
//!
//! Elaboration only knew a variable's element width and its first unpacked
//! dimension, so over an unpacked array `localparam P = $bits(fx)` folded to
//! 0 (the reference simulator gives 128), `$unpacked_dimensions(fx2)` to 0
//! (2), `$increment` to 0, and `logic [$bits(fx)-1:0]` declared one bit.
//! The queries now answer from the declaration itself — every unpacked
//! dimension in declared order, then the packed ones (§7.4.5), including a
//! typedef's own unpacked dimensions. Every expected value below comes from
//! the reference simulator.

use xezim::simulate;

fn t_lines(src: &str) -> Vec<String> {
    let mut v: Vec<String> = simulate(src, 100)
        .expect("simulate")
        .output
        .iter()
        .map(|o| o.message.trim_end().to_string())
        .filter(|l| l.starts_with("T|"))
        .collect();
    v.sort();
    v
}

fn sorted(lines: &[&str]) -> Vec<String> {
    let mut v: Vec<String> = lines.iter().map(|s| s.to_string()).collect();
    v.sort();
    v
}

/// Fixed unpacked arrays — one- and multi-dimensional, packed + unpacked,
/// struct elements, a typedef'd array type, ascending and descending and
/// negative bounds — queried in localparams, packed widths, parameter
/// overrides, and at run time for comparison.
#[test]
fn queries_over_declared_unpacked_arrays() {
    let got = t_lines(
        r#"
module sub #(parameter int W = 1) (output logic [W-1:0] o);
  assign o = '1;
endmodule
module top;
  typedef struct packed { logic [3:0] a; logic [11:0] b; } ps_t;
  typedef int arr_t [3:0][1:5];
  int fx[4];
  int fx2[2][3];
  logic [7:0] pu [0:2][5:1];
  logic [3:0][1:0] pp [6];
  ps_t sa [2:7];
  arr_t ta;
  bit bb [-1:2];
  byte dd [3:-2];
  localparam P1 = $bits(fx);
  localparam P2 = $unpacked_dimensions(fx2);
  localparam P3 = $bits(fx2);
  localparam P4 = $dimensions(fx2);
  localparam P5 = $size(fx2);
  localparam P6 = $size(fx2, 2);
  localparam P7 = $bits(pu);
  localparam P8 = $dimensions(pu);
  localparam P9 = $unpacked_dimensions(pu);
  localparam P10 = $left(pu, 2);
  localparam P11 = $right(pu, 2);
  localparam P12 = $low(pu, 2);
  localparam P13 = $high(pu, 2);
  localparam P14 = $increment(pu, 2);
  localparam P15 = $left(pu, 3);
  localparam P16 = $right(pu);
  localparam P17 = $bits(pp);
  localparam P18 = $dimensions(pp);
  localparam P19 = $unpacked_dimensions(pp);
  localparam P20 = $size(pp, 2);
  localparam P21 = $left(pp, 3);
  localparam P22 = $bits(sa);
  localparam P23 = $left(sa);
  localparam P24 = $increment(sa);
  localparam P25 = $size(sa);
  localparam P26 = $bits(ta);
  localparam P27 = $left(ta, 2);
  localparam P28 = $size(ta, 2);
  localparam P29 = $unpacked_dimensions(ta);
  localparam P30 = $bits(arr_t);
  localparam P31 = $dimensions(arr_t);
  localparam P32 = $low(bb);
  localparam P33 = $high(bb);
  localparam P34 = $increment(dd);
  localparam P35 = $low(dd);
  localparam P36 = $high(dd);
  localparam P37 = $bits(dd);
  localparam P38 = $left(fx);
  localparam P39 = $right(fx);
  localparam P40 = $increment(fx);
  localparam P41 = $high(fx2, 2);
  localparam P42 = $dimensions(fx);
  localparam P43 = $unpacked_dimensions(sa);
  localparam P44 = $size(pu, 3);
  logic [$bits(fx)-1:0] wv;
  logic [$size(fx2,2)*4-1:0] wv2;
  wire [$bits(pu)-1:0] o1;
  sub #(.W($size(sa))) u1 (.o());
  sub #(.W($bits(fx2))) u2 (.o());
  initial begin
    $display("T|A %0d %0d %0d %0d %0d %0d", P1, P2, P3, P4, P5, P6);
    $display("T|B %0d %0d %0d %0d %0d %0d %0d %0d %0d %0d", P7, P8, P9, P10, P11, P12, P13, P14, P15, P16);
    $display("T|C %0d %0d %0d %0d %0d", P17, P18, P19, P20, P21);
    $display("T|D %0d %0d %0d %0d", P22, P23, P24, P25);
    $display("T|E %0d %0d %0d %0d %0d %0d", P26, P27, P28, P29, P30, P31);
    $display("T|F %0d %0d %0d %0d %0d %0d", P32, P33, P34, P35, P36, P37);
    $display("T|G %0d %0d %0d %0d %0d %0d %0d", P38, P39, P40, P41, P42, P43, P44);
    $display("T|H %0d %0d %0d", $bits(wv), $bits(wv2), $bits(o1));
    $display("T|I %0d %0d", u1.W, u2.W);
    $display("T|J %0d %0d %0d", $bits(fx), $unpacked_dimensions(fx2), $left(pu,2));
  end
endmodule
"#,
    );
    assert_eq!(
        got,
        sorted(&[
            "T|A 128 2 192 3 2 3",
            "T|B 120 3 2 5 1 1 5 1 7 2",
            "T|C 48 3 1 4 1",
            "T|D 96 2 -1 6",
            "T|E 640 1 5 2 640 3",
            "T|F -1 2 1 -2 3 48",
            "T|G 0 3 -1 2 2 1 8",
            "T|H 128 12 120",
            "T|I 6 192",
            "T|J 128 2 5",
        ]),
    );
}

/// Enum and unpacked-struct elements, a scalar, packed-only variables, and a
/// parameter default sized from a declared array.
#[test]
fn queries_over_other_element_types() {
    let got = t_lines(
        r#"
module top;
  typedef struct { int a; byte b; } us_t;
  typedef enum logic [2:0] { E0, E1 } e_t;
  us_t usa [3];
  e_t ea [2][4];
  logic s;
  logic [5:2] v;
  bit [3:0][7:0] pk;
  int fx2[2][3];
  localparam A = $bits(ea);
  localparam B = $dimensions(ea);
  localparam E = $dimensions(s);
  localparam F = $left(v);
  localparam G = $dimensions(pk);
  localparam H = $size(pk, 2);
  localparam I = $unpacked_dimensions(v);
  localparam J = $bits(usa);
  parameter int PD = $bits(fx2) + 1;
  initial $display("T|Q %0d %0d %0d %0d %0d %0d %0d %0d %0d", A, B, E, F, G, H, I, J, PD);
endmodule
"#,
    );
    assert_eq!(got, sorted(&["T|Q 24 2 0 5 2 8 0 120 193"]));
}

/// The same queries inside an instantiated module, where each instance's
/// array is sized by its own parameter, and in a child's parameter override
/// and default.
#[test]
fn queries_in_instance_scopes() {
    let got = t_lines(
        r#"
module sub #(parameter int N = 4) (input logic [7:0] pin [N]);
  typedef struct packed { logic [2:0] a; logic [4:0] b; } st_t;
  int arr [N][2];
  st_t sarr [N:1];
  localparam int B1 = $bits(arr);
  localparam int B2 = $size(arr);
  localparam int B3 = $unpacked_dimensions(arr);
  localparam int B4 = $bits(sarr);
  localparam int B5 = $increment(sarr);
  localparam int B6 = $bits(pin);
  localparam int B7 = $left(sarr);
  logic [$bits(arr)-1:0] flat;
  initial $display("T|SUB%0d %0d %0d %0d %0d %0d %0d %0d %0d", N, B1, B2, B3, B4, B5, B6, B7, $bits(flat));
endmodule
module mid #(parameter int W = $bits(int)) ();
  initial $display("T|MID %0d", W);
endmodule
module top;
  logic [7:0] p3 [3];
  logic [7:0] p6 [6];
  shortint sv [5];
  sub #(.N(3)) u3 (.pin(p3));
  sub #(.N(6)) u6 (.pin(p6));
  mid #(.W($bits(sv))) m1 ();
  mid #(.W($size(sv) * 2)) m2 ();
  mid m3 ();
endmodule
"#,
    );
    assert_eq!(
        got,
        sorted(&[
            "T|SUB3 192 3 2 24 1 24 3 192",
            "T|SUB6 384 6 2 48 1 48 6 384",
            "T|MID 80",
            "T|MID 10",
            "T|MID 32",
        ]),
    );
}

/// A variable typed by the module's TYPE parameter is sized from each
/// instance's binding — the default and an override.
#[test]
fn queries_over_type_parameter_typed_variables() {
    let got = t_lines(
        r#"
module sub #(parameter type T = logic [3:0]) ();
  T x;
  T arr [3];
  localparam B1 = $bits(x);
  localparam B2 = $bits(arr);
  localparam B3 = $size(arr);
  logic [$bits(x)-1:0] w;
  initial $display("T|S %0d %0d %0d %0d %0d", B1, B2, B3, $bits(w), $bits(arr));
endmodule
module top;
  sub u0 ();
  sub #(.T(byte)) u1 ();
endmodule
"#,
    );
    assert_eq!(got, sorted(&["T|S 4 12 3 4 12", "T|S 8 24 3 8 24"]));
}
