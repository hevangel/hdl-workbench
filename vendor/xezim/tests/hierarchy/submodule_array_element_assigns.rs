//! Continuous assignments to one bit of an unpacked-array element,
//! `assign r[i][j] = ...`, where `r` is a 1-D unpacked array of packed
//! vectors.
//!
//! They must drive the array inside an instantiated module exactly as at the
//! top level: the sub-module form used to be dropped, leaving the array `z`
//! and anything clocked through it dead. The generate-loop form is the shape
//! of a PLIC's per-hart enable clock gates. Neighbouring shapes (part-selects,
//! 2-D arrays, nested instances, label-mapped elements, variables driven
//! through output ports, procedural writes) are pinned alongside.
//! Expected values are the reference simulator's.

use xezim::simulate;

fn out(src: &str) -> Vec<String> {
    let sim = simulate(src, 1000).expect("simulate failed");
    sim.output.iter().map(|o| o.message.clone()).collect()
}

#[test]
fn top_level_element_bit_assigns_drive_the_array() {
    let o = out(r#"
module top;
  logic a = 1;
  wire [1:0] r [1:0];
  assign r[0][0] = a;
  assign r[0][1] = ~a;
  assign r[1][1] = a;
  assign r[1][0] = 1'b0;
  wire [3:0] p [1:0];
  assign p[1] = {a, a, 2'b01};
  initial begin
    #1 $display("T|r0=%b r1=%b p1=%b", r[0], r[1], p[1]);
    a = 0;
    #1 $display("T|r0=%b r1=%b p1=%b", r[0], r[1], p[1]);
    $finish;
  end
endmodule
"#);
    assert_eq!(
        o,
        ["T|r0=01 r1=10 p1=1101", "T|r0=10 r1=00 p1=0001"],
        "{o:?}"
    );
}

#[test]
fn submodule_element_bit_assigns_drive_the_array() {
    let o = out(r#"
module direct (input logic a, output logic [1:0] r0, output logic [1:0] r1);
  wire [1:0] r [1:0];
  assign r[0][0] = a;
  assign r[0][1] = ~a;
  assign r[1][1] = a;
  assign r[1][0] = 1'b0;
  assign r0 = r[0];
  assign r1 = r[1];
endmodule
module top;
  logic a = 1;
  wire [1:0] r0, r1;
  direct dr (.a(a), .r0(r0), .r1(r1));
  initial begin
    #1 $display("T|r0=%b r1=%b", r0, r1);
    a = 0;
    #1 $display("T|r0=%b r1=%b", r0, r1);
    $finish;
  end
endmodule
"#);
    assert_eq!(o, ["T|r0=01 r1=10", "T|r0=10 r1=00"], "{o:?}");
}

#[test]
fn generate_loop_element_bit_clock_gates_fire_their_flops() {
    let o = out(r#"
module gclk (input logic clk_in, input logic en, output logic clk_out);
  assign clk_out = clk_in & en;
endmodule
module flop (input logic clk, input logic [3:0] d, output logic [3:0] q);
  initial q = 0;
  always @(posedge clk) q <= d;
endmodule
module busif (input logic clk, input logic [1:0] we0, input logic [1:0] we1,
              input logic [3:0] d, output logic [7:0] q0, output logic [7:0] q1,
              output logic [1:0] en0, output logic [1:0] en1);
  wire [1:0] wclk [1:0];
  wire [1:0] wen [1:0];
  wire [7:0] q [1:0];
  genvar i, j;
  for (i = 0; i < 2; i++) begin : G
    for (j = 0; j < 2; j++) begin : J
      assign wen[i][j] = (i == 0) ? we0[j] : we1[j];
      gclk g (.clk_in(clk), .en(wen[i][j]), .clk_out(wclk[i][j]));
      flop f (.clk(wclk[i][j]), .d(d), .q(q[i][4*j+:4]));
    end
  end
  assign q0 = q[0];
  assign q1 = q[1];
  assign en0 = wen[0];
  assign en1 = wen[1];
endmodule
module top;
  logic clk = 0;
  logic [1:0] we0 = 2'b01, we1 = 2'b10;
  logic [3:0] d = 4'h5;
  wire [7:0] q0, q1;
  wire [1:0] en0, en1;
  busif b (.clk(clk), .we0(we0), .we1(we1), .d(d), .q0(q0), .q1(q1), .en0(en0), .en1(en1));
  initial begin
    #1 clk = 1;
    #1 $display("T|en0=%b en1=%b q0=%h q1=%h", en0, en1, q0, q1);
    d = 4'ha; clk = 0; we0 = 2'b10; we1 = 2'b01;
    #1 clk = 1;
    #1 $display("T|en0=%b en1=%b q0=%h q1=%h", en0, en1, q0, q1);
    $finish;
  end
endmodule
"#);
    assert_eq!(
        o,
        ["T|en0=01 en1=10 q0=05 q1=50", "T|en0=10 en1=01 q0=a5 q1=5a"],
        "{o:?}"
    );
}

#[test]
fn submodule_element_part_select_assigns_drive_the_array() {
    let o = out(r#"
module psel (input logic [1:0] a, output logic [3:0] r0, output logic [3:0] r1);
  wire [3:0] r [1:0];
  genvar k;
  for (k = 0; k < 2; k++) begin : G
    assign r[k][2*k+:2] = a;
    assign r[k][2*(1-k)+:2] = ~a;
  end
  assign r0 = r[0];
  assign r1 = r[1];
endmodule
module top;
  logic [1:0] b = 2'b01;
  wire [3:0] r0, r1;
  psel ps (.a(b), .r0(r0), .r1(r1));
  initial begin
    #1 $display("T|r0=%b r1=%b", r0, r1);
    b = 2'b10;
    #1 $display("T|r0=%b r1=%b", r0, r1);
    $finish;
  end
endmodule
"#);
    assert_eq!(o, ["T|r0=1001 r1=0110", "T|r0=0110 r1=1001"], "{o:?}");
}

/// `r[i][j][k]` on a 2-D array: the element `r[i][j]` is its own signal and
/// `[k]` one bit of it. Dropped at the top level too, not only in a
/// sub-module.
#[test]
fn two_dimensional_element_bit_assigns_drive_the_array() {
    let o = out(r#"
module twod (input logic a, output logic [2:0] q00, output logic [2:0] q01,
             output logic [2:0] q10, output logic [2:0] q11);
  wire [2:0] r [1:0][1:0];
  assign r[0][0][0] = a;
  assign r[0][0][2:1] = {a, ~a};
  assign r[0][1] = {3{a}};
  assign r[1][0][1] = ~a;
  assign r[1][1][2] = a;
  assign q00 = r[0][0];
  assign q01 = r[0][1];
  assign q10 = r[1][0];
  assign q11 = r[1][1];
endmodule
module top;
  logic a = 1;
  wire [2:0] q00, q01, q10, q11;
  wire [2:0] t [1:0][1:0];
  assign t[1][0][2] = a;
  assign t[0][1][0] = ~a;
  twod td (.a(a), .q00(q00), .q01(q01), .q10(q10), .q11(q11));
  initial begin
    #1 $display("T|%b %b %b %b top %b %b", q00, q01, q10, q11, t[1][0], t[0][1]);
    a = 0;
    #1 $display("T|%b %b %b %b top %b %b", q00, q01, q10, q11, t[1][0], t[0][1]);
    $finish;
  end
endmodule
"#);
    assert_eq!(
        o,
        [
            "T|101 111 z0z 1zz top 1zz zz0",
            "T|010 000 z1z 0zz top 0zz zz1"
        ],
        "{o:?}"
    );
}

/// An element bit driven through an OUTPUT PORT connection, and the plain
/// assigns one level further down (a sub-module inside a sub-module).
#[test]
fn nested_submodule_and_output_port_element_bits() {
    let o = out(r#"
module gclk (input logic clk_in, input logic en, output logic clk_out);
  assign clk_out = clk_in & en;
endmodule
module inner (input logic a, output logic [1:0] o0, output logic [1:0] o1);
  wire [1:0] r [1:0];
  assign r[0][0] = a;
  assign r[0][1] = ~a;
  assign r[1][1] = a;
  assign r[1][0] = 1'b1;
  assign o0 = r[0];
  assign o1 = r[1];
endmodule
module outer (input logic a, output logic [1:0] o0, output logic [1:0] o1,
              output logic [1:0] p0);
  wire [1:0] pr [1:0];
  inner u (.a(a), .o0(o0), .o1(o1));
  gclk g0 (.clk_in(a), .en(1'b1), .clk_out(pr[0][1]));
  gclk g1 (.clk_in(a), .en(1'b0), .clk_out(pr[0][0]));
  assign p0 = pr[0];
endmodule
module top;
  logic a = 1;
  wire [1:0] o0, o1, p0;
  outer ou (.a(a), .o0(o0), .o1(o1), .p0(p0));
  initial begin
    #1 $display("T|%b %b %b", o0, o1, p0);
    a = 0;
    #1 $display("T|%b %b %b", o0, o1, p0);
    $finish;
  end
endmodule
"#);
    assert_eq!(o, ["T|01 11 10", "T|10 01 00"], "{o:?}");
}

/// §7.4.1 label mapping inside the element: a non-zero-based `[4:1]`
/// element of a `[2:3]` array, and an ascending `[0:3]` element.
#[test]
fn non_zero_based_and_ascending_element_bits_in_a_submodule() {
    let o = out(r#"
module nzb (input logic a, output logic [3:0] d0, output logic [3:0] d1,
            output logic [0:3] u0, output logic [0:3] u1);
  wire [4:1] d [2:3];
  wire [0:3] u [0:1];
  assign d[2][1] = a;
  assign d[2][4] = ~a;
  assign d[3][3:2] = {a, 1'b1};
  assign u[0][0] = a;
  assign u[0][3] = ~a;
  assign u[1][1:2] = {a, 1'b0};
  assign d0 = d[2];
  assign d1 = d[3];
  assign u0 = u[0];
  assign u1 = u[1];
endmodule
module top;
  logic a = 1;
  wire [3:0] d0, d1;
  wire [0:3] u0, u1;
  nzb nz (.a(a), .d0(d0), .d1(d1), .u0(u0), .u1(u1));
  initial begin
    #1 $display("T|%b %b %b %b", d0, d1, u0, u1);
    a = 0;
    #1 $display("T|%b %b %b %b", d0, d1, u0, u1);
    $finish;
  end
endmodule
"#);
    assert_eq!(
        o,
        ["T|0zz1 z11z 1zz0 z10z", "T|1zz0 z01z 0zz1 z00z"],
        "{o:?}"
    );
}

/// A VARIABLE array whose element bits are each driven by one output port
/// (§6.5 allows one continuous driver per bit), plus a non-ANSI module with
/// an ascending `[0:1]` array. Undriven variable bits stay x.
#[test]
fn variable_element_bits_driven_by_output_ports_and_non_ansi() {
    let o = out(r#"
module gclk (clk_in, en, clk_out);
  input clk_in; input en; output clk_out;
  assign clk_out = clk_in & en;
endmodule
module vdrv (input logic a, output logic [1:0] v0, output logic [1:0] v1, output logic s);
  logic [1:0] v [1:0];
  logic sv [1:0];
  gclk g00 (.clk_in(a), .en(1'b1), .clk_out(v[0][0]));
  gclk g01 (.clk_in(1'b1), .en(a), .clk_out(v[0][1]));
  gclk g11 (.clk_in(a), .en(a), .clk_out(v[1][1]));
  gclk gs (.clk_in(a), .en(1'b1), .clk_out(sv[1]));
  assign v0 = v[0];
  assign v1 = v[1];
  assign s = sv[1];
endmodule
module nansi (a, r0, r1);
  input a; output [1:0] r0; output [1:0] r1;
  wire [1:0] r [0:1];
  assign r[0][1] = a;
  assign r[1][0] = ~a;
  assign r0 = r[0];
  assign r1 = r[1];
endmodule
module top;
  logic a = 1;
  wire [1:0] v0, v1, r0, r1;
  wire s;
  vdrv vd (.a(a), .v0(v0), .v1(v1), .s(s));
  nansi na (a, r0, r1);
  initial begin
    #1 $display("T|%b %b %b %b %b", v0, v1, s, r0, r1);
    a = 0;
    #1 $display("T|%b %b %b %b %b", v0, v1, s, r0, r1);
    $finish;
  end
endmodule
"#);
    assert_eq!(o, ["T|11 1x 1 1z z0", "T|00 0x 0 0z z1"], "{o:?}");
}

/// Procedural bit writes into an element of a 2-D array (constant and
/// variable indices) and of a 1-D array.
#[test]
fn procedural_element_bit_writes_on_one_and_two_dimensional_arrays() {
    let o = out(r#"
module top;
  logic a = 1;
  logic [2:0] r [1:0][1:0];
  logic [2:0] s [1:0];
  integer i = 1, j = 0, k = 2;
  initial begin
    r[0][0] = 3'b000; r[1][0] = 3'b000; s[1] = 3'b000;
    r[0][0][0] = a;
    r[i][j][k] = a;
    s[1][2] = a;
    s[i][1] = a;
    #1 $display("T|pr %b %b %b", r[0][0], r[1][0], s[1]);
    $finish;
  end
endmodule
"#);
    assert_eq!(o, ["T|pr 001 100 110"], "{o:?}");
}

/// The right-hand side of an element-bit assign is evaluated at the width of
/// its one-bit target (§11.6.1): `(a + b) >> 1` loses the carry.
#[test]
fn element_bit_assign_rhs_takes_the_one_bit_context() {
    let o = out(r#"
module top;
  logic a = 1, b = 0;
  wire [1:0] r [1:0];
  wire [1:0] s [1:0];
  assign r[0][0] = a;
  assign r[0][1] = a & b;
  assign r[1][1] = ~(a | b);
  assign r[1][0] = a ? b : 1'b1;
  assign s[1][0] = (a + b) >> 1;
  assign s[1][1] = r[0][0] ^ b;
  initial begin
    #1 $display("T|%b %b %b", r[0], r[1], s[1]);
    b = 1;
    #1 $display("T|%b %b %b", r[0], r[1], s[1]);
    $finish;
  end
endmodule
"#);
    assert_eq!(o, ["T|01 00 10", "T|11 01 00"], "{o:?}");
}

/// A multi-D PACKED element (`wire [3:0][7:0] r [1:0]`): the trailing index
/// selects a byte lane of the element, not a bit.
#[test]
fn packed_lane_of_an_element_in_a_submodule() {
    let o = out(r#"
module sub (input logic [7:0] a, output logic [31:0] o0, output logic [7:0] rd);
  wire [3:0][7:0] r [1:0];
  assign r[0][1] = a;
  assign r[0][0] = 8'h11;
  assign r[0][2] = 8'h33;
  assign r[0][3] = 8'h44;
  assign o0 = r[0];
  assign rd = r[0][1];
endmodule
module top;
  logic [7:0] a = 8'h22;
  wire [3:0][7:0] t [1:0];
  wire [31:0] o0; wire [7:0] rd;
  assign t[1][2] = a;
  assign t[1][0] = 8'h11;
  sub s (.a(a), .o0(o0), .rd(rd));
  initial begin
    #1 $display("T|%h %h %h %h", o0, rd, t[1], t[1][2]);
    a = 8'h55;
    #1 $display("T|%h %h %h %h", o0, rd, t[1], t[1][2]);
    $finish;
  end
endmodule
"#);
    assert_eq!(
        o,
        ["T|44332211 22 zz22zz11 22", "T|44335511 55 zz55zz11 55"],
        "{o:?}"
    );
}
