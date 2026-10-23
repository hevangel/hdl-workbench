`timescale 1ns/1ps

// ----- self-check macros (tests/classes SVTEST_* style, inlined) -----
`ifndef SVTEST_DEFS_SVH
`define SVTEST_DEFS_SVH

`define SVTEST_INIT \
int failures = 0;

`define SVTEST_CHECK(expr, msg) \
if (!(expr)) begin \
  failures++; \
  $display("FAIL @%0t : %s", $time, msg); \
end

`define SVTEST_PASSFAIL \
if (failures == 0) begin \
  $display("TEST_PASS"); \
end else begin \
  $display("TEST_FAIL count=%0d", failures); \
  $fatal(1); \
end

`endif

// ============================================================================
// param_range_signedness.sv: value semantics of parameters and localparams
// declared with an EXPLICIT type or range.
//
// IEEE 1800 §6.20.2 / §10.7 / §23.10.2: a parameter or localparam declared
// with an explicit type or range is ASSIGNED its initializer, and each
// override (`#(...)` or defparam) is likewise an assignment to the formal's
// type. The value is evaluated in the declared width context, then wrapped
// to that width with the DECLARED signedness:
//  * a bare range `[3:0]` is UNSIGNED (`-1` -> 15, `256` in `[7:0]` -> 0),
//    and a value read through it never sign-extends: through an assign, a
//    `bit [1:0]`, a generate-if or ternary condition, `C + C`, or a range
//    `logic [C-1:0]` (PART 1);
//  * `signed [3:0]` wraps (`15` -> -1), for a default and an override;
//  * a typedef'd SIGNED type keeps its sign, including a typedef local to
//    the module (`typedef logic signed [1:0] T; localparam T C = ...`);
//  * defaults, named `.P(-1)` and positional `#(-1)` overrides, and
//    localparams in the parameter port list all wrap (PART 2), and the
//    wrapped value is what hierarchical reads and child ports see (PART 3);
//  * an override is evaluated in the formal's width: `.P(4'hF + 4'h1)` on
//    `parameter [7:0] P` is 16, and a formal whose range uses another
//    formal (`parameter [W-1:0] P`) takes the instance's own W (PART 4);
//  * a value parameter typed by a type parameter takes that type's full
//    width, for the default type and an overriding one (PART 4);
//  * defparam values, unpacked-array elements, `signed` with no range, 2-state
//    types (X/Z -> 0) and real initializers (rounded) convert the same way
//    (PART 5).
// An untyped parameter keeps its value and signedness unchanged (§6.20.2).
//
// Checks marked [control] pin neighbouring behaviour (signed wrap, typedef'd
// unsigned, int, shift and replication amounts, real, untyped parameters, the
// width context of `localparam [7:0] C = P + P`) so the conversion does not
// overcorrect.
//
// Self-checking: only failing checks print ("FAIL @<time> : <msg>"); the
// final block prints TEST_PASS, or TEST_FAIL count=N.
// ============================================================================

// ----- PART 1 modules: localparam with an explicit range/type -----

module mid_a1  #(parameter P = 1) (output [6:0] y);
  localparam [1:0] C = 1 + P;         // 2, wrapped to 2'b10, UNSIGNED
  assign y = C;                       // 2: an unsigned range does not sign-extend
endmodule

module mid_a2  #(parameter P = 1) (output [6:0] y);
  localparam signed [1:0] C = 1 + P;  // 2 wraps to -2 in 2-bit signed
  assign y = C;                       // [control] 126 is CORRECT here
endmodule

module mid_a3  #(parameter P = 1) (output [6:0] y);
  localparam bit [1:0] C = 1 + P;     // 2-state 2-bit unsigned
  assign y = C;                       // 2
endmodule

module mid_a4  #(parameter P = 1) (output [6:0] y);
  typedef logic [1:0] T;
  localparam T C = 1 + P;             // [control] typedef'd unsigned works
  assign y = C;
endmodule

module mid_a4b #(parameter P = 1) (output [6:0] y);
  typedef logic signed [1:0] T;
  localparam T C = 1 + P;             // 2 wraps to -2 (signed typedef)
  assign y = C;                       // 126: the typedef's sign is kept
endmodule

module mid_a5  #(parameter P = 1) (output [6:0] y);
  localparam int C = 1 + P;           // [control] int: 32-bit, no wrap
  assign y = C;
endmodule

module mid_a6  #(parameter P = 1) (output [6:0] y);
  localparam [1:0] C = 1 + P;
  if (C == 2) assign y = 7'd2;        // generate-if condition sees C == 2
  else        assign y = 7'd126;
endmodule

module mid_a7  #(parameter P = 1) (output [6:0] y);
  localparam [1:0] C = 1 + P;
  assign y = (C == 2) ? 7'd2 : 7'd126;  // ternary condition sees C == 2
endmodule

module mid_a8  #(parameter P = 1) (output [6:0] y);
  localparam [1:0] C = 1 + P;
  assign y = C + C;                   // 4, unsigned arithmetic
endmodule

module mid_a9  #(parameter P = 1) (output [6:0] y);
  localparam [1:0] C = 1 + P;
  assign y = 7'd1 << C;               // [control] shift amount works
endmodule

module mid_a10 #(parameter P = 1) (output [6:0] y);
  localparam [1:0] C = 1 + P;
  assign y = {C{1'b1}};               // [control] replication count works
endmodule

module mid_a11 #(parameter P = 1) (output [6:0] y);
  localparam [1:0] C = 1 + P;
  logic [C-1:0] r;                    // [1:0] -> 2 bits
  assign y = $bits(r);                // 2
endmodule

module mid_a12 #(parameter P = 1) (output [6:0] y);
  localparam real R = 1 + P;          // [control] real localparam
  assign y = (R == 2.0);
endmodule

// ----- PART 2 modules: parameter with an explicit range (default/override) -----

module mid_b1 #(parameter [3:0] P = -1) (output [6:0] y);
  assign y = P;                       // 15: -1 wrapped to [3:0], unsigned
endmodule

module mid_b2 #(parameter [3:0] P = 0) (output [6:0] y);
  assign y = P;                       // 15: the .P(-1) override wraps
endmodule

module mid_b3 #(parameter [3:0] P = 0) (output [6:0] y);
  assign y = P;                       // 15: the positional #(-1) override wraps
endmodule

module mid_b4 #(parameter signed [3:0] P = 15) (output [6:0] y);
  assign y = P;                       // 127: 15 wraps to -1, sign-extends
endmodule

module mid_b5 #(parameter [7:0] P = 256) (output [31:0] y);
  assign y = P;                       // 0: 256 wraps
endmodule

module mid_b6 #(parameter P = 1, localparam [3:0] LP = -1) (output [6:0] y);
  assign y = LP;                      // 15: a localparam in the #() list wraps
endmodule

module mid_b7 #(parameter P = -1) (output [6:0] y);
  assign y = P;                       // [control] untyped keeps -1 (127)
endmodule

module mid_b8 #(parameter signed [3:0] P = 0) (output [6:0] y);
  assign y = P;                       // 127: the .P(15) override wraps to -1
endmodule

module mid_b9 #(parameter [3:0] P = 15) (output [6:0] y);
  localparam [7:0] C = P + P;         // [control] 30
  assign y = C;
endmodule

// ----- PART 3 modules: consumption contexts (hier read, child port) -----

module child (input [1:0] n, output [6:0] y);
  assign y = n;
endmodule

module mid_c1 #(parameter P = 1) (output [6:0] y);
  localparam [1:0] C = 1 + P;
  assign y = C;                       // read hierarchically as u_c1.C too
endmodule

module mid_orig #(parameter P = 0) (output [6:0] y);
  localparam [1:0] C = 1 + P;         // C feeding a child port
  child u (.n(C), .y(y));             // the child port sees 2
endmodule

// ----- PART 4 modules: override width context, type-parameter typing -----

module mid_d1 #(parameter type T = logic [63:0], parameter T P = 64'h1234_5678_9abc_def0) ();
endmodule  // P keeps all 64 bits of the default type

module mid_d2 #(parameter type T = int, parameter T P = 0) ();
endmodule  // overridden with T = logic [47:0]: P has 48 bits; default T: int

module mid_d3 #(parameter type T = logic [63:0], parameter T P = '1) ();
endmodule  // '1 fills the 64-bit type

module mid_d5 #(parameter [7:0] P = 0) ();
endmodule  // .P(4'hF + 4'h1) is evaluated in 8 bits: 16

module mid_d6 #(parameter W = 8, parameter [W-1:0] P = 0) ();
endmodule  // the range uses this instance's W, not the parent's

module mid_d7 #(parameter W = 8, parameter type T = logic [W-1:0], parameter T P = '1) ();
endmodule  // T's default sized by the instance's W

// ----- PART 5 modules: defparam, arrays, no-range signed, 2-state, real -----

module mid_e1 #(parameter [3:0] P = 0) ();
endmodule  // defparam -1: 15

module mid_e2 #(parameter signed [3:0] P = 0) ();
endmodule  // defparam 4'hF: -1

module mid_e3 #(parameter logic [3:0] A [0:1] = '{-1, 17}) ();
endmodule  // each element wraps: 15, 1

module mid_e4 #(parameter logic [3:0] A [0:1] = '{0, 0}) ();
endmodule  // .A('{-1, 18}): 15, 2

module mid_e5 #(parameter signed P = 4'b1111) ();
endmodule  // signed, width of the value: 4-bit -1

module mid_e6 #(parameter signed P = 0) ();
endmodule  // .P(4'b1111): -1

module mid_e7 #(parameter bit [3:0] P = 4'bx01z) ();
  localparam bit [3:0] LB = 4'bz10x;  // 2-state: X/Z become 0
  localparam int LR = 2.6;            // real rounds: 3
endmodule

module mid_e8 #(parameter int P = 2.6) ();
endmodule  // 3; .P(2.6) also 3

module mid_e9 #(parameter [7:0] P = 0) ();
endmodule  // .P(300.4): 300 wraps to 44

// ----- top: instantiate everything, check, verdict -----

module top;
  `SVTEST_INIT

  // [control] B9TOP: module-scope parameters (no instance override path)
  parameter [3:0] TP = 15;
  localparam [7:0] TC = TP + TP;

  wire [6:0]  ya1, ya2, ya3, ya4, ya4b, ya5, ya6, ya7, ya8, ya9, ya10, ya11, ya12;
  wire [6:0]  yb1, yb2, yb3, yb4, yb6, yb7, yb8, yb9;
  wire [31:0] yb5;
  wire [6:0]  yc1, yorig;

  mid_a1   u_a1  (.y(ya1));
  mid_a2   u_a2  (.y(ya2));
  mid_a3   u_a3  (.y(ya3));
  mid_a4   u_a4  (.y(ya4));
  mid_a4b  u_a4b (.y(ya4b));
  mid_a5   u_a5  (.y(ya5));
  mid_a6   u_a6  (.y(ya6));
  mid_a7   u_a7  (.y(ya7));
  mid_a8   u_a8  (.y(ya8));
  mid_a9   u_a9  (.y(ya9));
  mid_a10  u_a10 (.y(ya10));
  mid_a11  u_a11 (.y(ya11));
  mid_a12  u_a12 (.y(ya12));

  mid_b1   u_b1  (.y(yb1));
  mid_b2 #(.P(-1)) u_b2 (.y(yb2));
  mid_b3 #(-1)     u_b3 (.y(yb3));
  mid_b4   u_b4  (.y(yb4));
  mid_b5   u_b5  (.y(yb5));
  mid_b6   u_b6  (.y(yb6));
  mid_b7   u_b7  (.y(yb7));
  mid_b8 #(.P(15)) u_b8 (.y(yb8));
  mid_b9   u_b9  (.y(yb9));

  mid_c1   u_c1  (.y(yc1));
  mid_orig #(.P(1)) u_orig (.y(yorig));

  // PART 4: a parent W that differs from the child's own W.
  parameter W = 4;
  mid_d1 u_d1 ();
  mid_d2 #(.T(logic [47:0]), .P(48'hFFFF_0000_1234)) u_d2 ();
  mid_d2 #(.P(-5)) u_d2i ();
  mid_d3 u_d3 ();
  mid_d5 #(.P(4'hF + 4'h1)) u_d5 ();
  mid_d6 #(.W(16), .P(16'hABCD)) u_d6 ();
  mid_d6 #(16, 16'h1234) u_d6o ();
  mid_d7 #(.W(48)) u_d7 ();

  // PART 5
  mid_e1 u_e1 ();
  defparam u_e1.P = -1;
  mid_e2 u_e2 ();
  defparam u_e2.P = 4'hF;
  mid_e3 u_e3 ();
  mid_e4 #(.A('{-1, 18})) u_e4 ();
  mid_e5 u_e5 ();
  mid_e6 #(.P(4'b1111)) u_e6 ();
  mid_e7 u_e7 ();
  mid_e8 u_e8 ();
  mid_e8 #(.P(2.6)) u_e8o ();
  mid_e9 #(.P(300.4)) u_e9 ();

  initial begin
    #1;

    // ---- PART 1: localparam explicit range/type value semantics ----
    `SVTEST_CHECK(ya1 === 7'd2,
      $sformatf("A1 localparam [1:0] C=1+P assign y=%0d (expect 2, unsigned range must not sign-extend)", ya1))
    `SVTEST_CHECK(ya2 === 7'd126,
      $sformatf("A2 [control] localparam signed [1:0] y=%0d (expect 126)", ya2))
    `SVTEST_CHECK(ya3 === 7'd2,
      $sformatf("A3 localparam bit [1:0] y=%0d (expect 2)", ya3))
    `SVTEST_CHECK(ya4 === 7'd2,
      $sformatf("A4 [control] typedef logic [1:0] y=%0d (expect 2)", ya4))
    `SVTEST_CHECK(ya4b === 7'd126,
      $sformatf("A4B typedef logic signed [1:0] y=%0d (expect 126, sign lost)", ya4b))
    `SVTEST_CHECK(ya5 === 7'd2,
      $sformatf("A5 [control] localparam int y=%0d (expect 2)", ya5))
    `SVTEST_CHECK(ya6 === 7'd2,
      $sformatf("A6 generate-if cond C==2 y=%0d (expect 2)", ya6))
    `SVTEST_CHECK(ya7 === 7'd2,
      $sformatf("A7 ternary cond C==2 y=%0d (expect 2)", ya7))
    `SVTEST_CHECK(ya8 === 7'd4,
      $sformatf("A8 C+C y=%0d (expect 4, not -4 sign-extended)", ya8))
    `SVTEST_CHECK(ya9 === 7'd4,
      $sformatf("A9 [control] 1<<C y=%0d (expect 4)", ya9))
    `SVTEST_CHECK(ya10 === 7'd3,
      $sformatf("A10 [control] {C{1'b1}} y=%0d (expect 3)", ya10))
    `SVTEST_CHECK(ya11 === 7'd2,
      $sformatf("A11 $bits([C-1:0]) y=%0d (expect 2)", ya11))
    `SVTEST_CHECK(ya12 === 7'd1,
      $sformatf("A12 [control] real R==2.0 y=%0d (expect 1)", ya12))

    // ---- PART 2: parameter explicit range, defaults and overrides ----
    `SVTEST_CHECK(yb1 === 7'd15,
      $sformatf("B1 parameter [3:0] P=-1 y=%0d (expect 15, wrapped unsigned)", yb1))
    `SVTEST_CHECK(u_b1.P === 15,
      $sformatf("B1P hier u_b1.P=%0d (expect 15)", u_b1.P))
    `SVTEST_CHECK(yb2 === 7'd15,
      $sformatf("B2 .P(-1) override y=%0d (expect 15)", yb2))
    `SVTEST_CHECK(u_b2.P === 15,
      $sformatf("B2P hier u_b2.P=%0d (expect 15)", u_b2.P))
    `SVTEST_CHECK(yb3 === 7'd15,
      $sformatf("B3 positional #(-1) y=%0d (expect 15)", yb3))
    `SVTEST_CHECK(u_b3.P === 15,
      $sformatf("B3P hier u_b3.P=%0d (expect 15)", u_b3.P))
    `SVTEST_CHECK(yb4 === 7'd127,
      $sformatf("B4 parameter signed [3:0] P=15 y=%0d (expect 127 via -1)", yb4))
    `SVTEST_CHECK(u_b4.P === -1,
      $sformatf("B4P hier u_b4.P=%0d (expect -1)", u_b4.P))
    `SVTEST_CHECK(yb5 === 32'd0,
      $sformatf("B5 parameter [7:0] P=256 y=%0d (expect 0, wrapped)", yb5))
    `SVTEST_CHECK(u_b5.P === 0,
      $sformatf("B5P hier u_b5.P=%0d (expect 0)", u_b5.P))
    `SVTEST_CHECK(yb6 === 7'd15,
      $sformatf("B6 localparam [3:0] LP=-1 in #() list y=%0d (expect 15)", yb6))
    `SVTEST_CHECK(u_b6.LP === 15,
      $sformatf("B6P hier u_b6.LP=%0d (expect 15)", u_b6.LP))
    `SVTEST_CHECK(u_b7.P === -1,
      $sformatf("B7 [control] untyped P=-1 hier=%0d (expect -1)", u_b7.P))
    `SVTEST_CHECK(yb8 === 7'd127,
      $sformatf("B8 signed [3:0] .P(15) override y=%0d (expect 127 via -1)", yb8))
    `SVTEST_CHECK(u_b8.P === -1,
      $sformatf("B8P hier u_b8.P=%0d (expect -1)", u_b8.P))
    `SVTEST_CHECK(yb9 === 7'd30,
      $sformatf("B9 [control] [3:0] P=15, [7:0] C=P+P y=%0d (expect 30)", yb9))
    `SVTEST_CHECK(TC === 8'd30,
      $sformatf("B9TOP [control] module-scope TC=%0d (expect 30)", TC))

    // ---- PART 3: consumption contexts ----
    `SVTEST_CHECK(u_c1.C === 2,
      $sformatf("C1 hier u_c1.C=%0d (expect 2)", u_c1.C))
    `SVTEST_CHECK(yc1 === 7'd2,
      $sformatf("C1Y localparam to signal y=%0d (expect 2)", yc1))
    `SVTEST_CHECK(yorig === 7'd2,
      $sformatf("ORIG localparam C to child port y=%0d (expect 2)", yorig))

    // ---- PART 4: override width context, type-parameter typing ----
    `SVTEST_CHECK(u_d1.P === 64'h1234_5678_9abc_def0,
      $sformatf("D1 parameter T P, default T = logic [63:0]: P=%h", u_d1.P))
    `SVTEST_CHECK(u_d2.P === 48'hFFFF_0000_1234 && $bits(u_d2.P) == 48,
      $sformatf("D2 parameter T P, .T(logic [47:0]) override: P=%h", u_d2.P))
    `SVTEST_CHECK(u_d2i.P === -5,
      $sformatf("D2I parameter T P, default T = int, .P(-5): P=%0d", u_d2i.P))
    `SVTEST_CHECK(u_d3.P === 64'hFFFF_FFFF_FFFF_FFFF,
      $sformatf("D3 parameter T P = '1, T = logic [63:0]: P=%h", u_d3.P))
    `SVTEST_CHECK(u_d5.P === 16,
      $sformatf("D5 [7:0] .P(4'hF + 4'h1)=%0d (expect 16)", u_d5.P))
    `SVTEST_CHECK(u_d6.P === 16'hABCD,
      $sformatf("D6 [W-1:0] .W(16) .P(16'hABCD)=%h", u_d6.P))
    `SVTEST_CHECK(u_d6o.P === 16'h1234,
      $sformatf("D6O [W-1:0] #(16, 16'h1234)=%h", u_d6o.P))
    `SVTEST_CHECK(u_d7.P === 48'hFFFF_FFFF_FFFF,
      $sformatf("D7 T = logic [W-1:0], .W(48), P = '1: P=%h", u_d7.P))

    // ---- PART 5: defparam, arrays, no-range signed, 2-state, real ----
    `SVTEST_CHECK(u_e1.P === 15,
      $sformatf("E1 defparam [3:0] P = -1: P=%0d (expect 15)", u_e1.P))
    `SVTEST_CHECK(u_e2.P === -1,
      $sformatf("E2 defparam signed [3:0] P = 4'hF: P=%0d (expect -1)", u_e2.P))
    `SVTEST_CHECK(u_e3.A[0] === 15 && u_e3.A[1] === 1,
      $sformatf("E3 [3:0] A = '{-1, 17}: %0d %0d (expect 15 1)", u_e3.A[0], u_e3.A[1]))
    `SVTEST_CHECK(u_e4.A[0] === 15 && u_e4.A[1] === 2,
      $sformatf("E4 .A('{-1, 18}): %0d %0d (expect 15 2)", u_e4.A[0], u_e4.A[1]))
    `SVTEST_CHECK(u_e5.P === -1 && $bits(u_e5.P) == 4,
      $sformatf("E5 signed P = 4'b1111: P=%0d (expect -1)", u_e5.P))
    `SVTEST_CHECK(u_e6.P === -1,
      $sformatf("E6 signed P, .P(4'b1111): P=%0d (expect -1)", u_e6.P))
    `SVTEST_CHECK(u_e7.P === 4'b0010,
      $sformatf("E7 bit [3:0] P = 4'bx01z: P=%b (expect 0010)", u_e7.P))
    `SVTEST_CHECK(u_e7.LB === 4'b0100,
      $sformatf("E7LB bit [3:0] LB = 4'bz10x: LB=%b (expect 0100)", u_e7.LB))
    `SVTEST_CHECK(u_e7.LR === 3,
      $sformatf("E7LR int LR = 2.6: LR=%0d (expect 3)", u_e7.LR))
    `SVTEST_CHECK(u_e8.P === 3,
      $sformatf("E8 int P = 2.6: P=%0d (expect 3)", u_e8.P))
    `SVTEST_CHECK(u_e8o.P === 3,
      $sformatf("E8O int .P(2.6): P=%0d (expect 3)", u_e8o.P))
    `SVTEST_CHECK(u_e9.P === 44,
      $sformatf("E9 [7:0] .P(300.4): P=%0d (expect 44)", u_e9.P))
  end

  final begin
    `SVTEST_PASSFAIL
  end
endmodule