//! §20.7 — `$bits` of an EXPRESSION in a constant/elaboration-time position.
//!
//! Const-eval resolves `$bits` of a parameter or a type NAME, but it has no
//! signal table, so `$bits` of a variable, a port, a struct-member select, a
//! part-select or an element select all fell through to **0** — silently, with
//! no diagnostic. Anything sized from one collapsed:
//!
//! ```systemverilog
//! sync_vector_wrapper #(.WIDTH($bits(src.wptr0))) u (...);   // WIDTH = 0
//! localparam W = $bits(port.member);                          // W = 0
//! ```
//!
//! A `[WIDTH-1:0]` port then elaborated as `[-1:0]`, so a synchronizer bus was
//! sized to nothing and its output stayed x for the whole run. The trigger was
//! a CDC testbench whose 4 pointer synchronizers were all parameterized off
//! `$bits` of a struct member of an input port — it reported x on every output
//! bit while a reference simulator ran it clean.
//!
//! `$bits(<type_name>)` always worked, which is why this hid: the idiomatic
//! `$bits(my_type_t)` spelling is far more common in tests than
//! `$bits(some_signal.field)` is.
//!
//! Every expectation below is byte-identical to a reference simulator.

use xezim::simulate;

fn u(sim: &xezim::compiler::Simulator, n: &str) -> u64 {
    sim.get_signal(n)
        .or_else(|| sim.get_signal(&format!("top.{}", n)))
        .unwrap_or_else(|| panic!("signal not found: {}", n))
        .to_u64()
        .unwrap_or_else(|| panic!("{} not u64-able (x/z?)", n))
}

/// The full range of operand shapes, as module-body localparams.
#[test]
fn bits_of_a_signal_expression_in_a_localparam() {
    let src = r#"
package pk;
  typedef struct packed { logic [1:0] a; } fld_t;
  typedef struct packed { fld_t w1; fld_t w0; } two_t;
  typedef logic [5:0] vec_t;
endpackage
module top;
  import pk::*;
  logic [7:0] pv;
  logic       sc;
  vec_t       tv;
  two_t       st;
  logic [3:0] arr [4];
  localparam A = $bits(pv);          // plain packed vector
  localparam B = $bits(sc);          // scalar
  localparam C = $bits(tv);          // typedef'd vector
  localparam D = $bits(st);          // packed struct
  localparam E = $bits(pv[3:0]);     // part-select
  localparam F = $bits(pv[2]);       // bit-select
  localparam G = $bits(arr[1]);      // unpacked-array element
  localparam H = $bits(st.w0);       // struct member
  localparam I = $bits(st.w0.a);     // nested member
  localparam J = $bits(fld_t);       // type name — always worked
  localparam K = $bits(pv) - 1;      // inside an expression
  int a, b, c, d, e, f, g, h, i, j, k;
  initial begin
    a = A; b = B; c = C; d = D; e = E; f = F; g = G; h = H; i = I; j = J; k = K;
  end
endmodule
"#;
    let sim = simulate(src, 20).expect("simulate failed");
    assert_eq!(u(&sim, "a"), 8, "plain packed vector");
    assert_eq!(u(&sim, "b"), 1, "scalar");
    assert_eq!(u(&sim, "c"), 6, "typedef'd vector");
    assert_eq!(u(&sim, "d"), 4, "packed struct");
    assert_eq!(u(&sim, "e"), 4, "part-select");
    assert_eq!(u(&sim, "f"), 1, "bit-select");
    assert_eq!(u(&sim, "g"), 4, "unpacked-array element");
    assert_eq!(u(&sim, "h"), 2, "struct member");
    assert_eq!(u(&sim, "i"), 2, "nested struct member");
    assert_eq!(u(&sim, "j"), 2, "type name");
    assert_eq!(u(&sim, "k"), 7, "$bits inside a larger expression");
}

/// The same operands on a sub-module's own PORT, in that sub-module's body
/// localparam. This path never consults the elaborated signal table (the
/// sub-module's names are not in it yet), so it resolves from the declared
/// types instead — a separate mechanism that needs its own coverage.
#[test]
fn bits_of_a_port_in_a_submodule_localparam() {
    let src = r#"
package pk;
  typedef struct packed { logic [1:0] a; } fld_t;
  typedef struct packed { fld_t w1; fld_t w0; } two_t;
endpackage
module mid import pk::*; (
  input clk, input two_t si, output two_t so,
  output int o_pm, output int o_pw, output int o_lm, output int o_pn);
  two_t loc;
  localparam P_MEMBER  = $bits(si.w0);
  localparam P_WHOLE   = $bits(si);
  localparam L_MEMBER  = $bits(loc.w0);
  localparam P_NESTED  = $bits(si.w0.a);
  assign o_pm = P_MEMBER;
  assign o_pw = P_WHOLE;
  assign o_lm = L_MEMBER;
  assign o_pn = P_NESTED;
endmodule
module top;
  import pk::*;
  logic clk = 0;
  two_t si, so;
  int pm, pw, lm, pn;
  mid d (clk, si, so, pm, pw, lm, pn);
endmodule
"#;
    let sim = simulate(src, 20).expect("simulate failed");
    assert_eq!(u(&sim, "pm"), 2, "member of a struct-typed port");
    assert_eq!(u(&sim, "pw"), 4, "the whole struct port");
    assert_eq!(u(&sim, "lm"), 2, "member of a local struct variable");
    assert_eq!(u(&sim, "pn"), 2, "nested member of a port");
}

/// The reported shape: a parameter OVERRIDE at an instance site, taken from
/// `$bits` of a struct member of the enclosing module's port. This is what
/// sized the synchronizer bus to zero.
#[test]
fn bits_of_a_port_member_as_an_instance_parameter_override() {
    let src = r#"
package pk;
  typedef struct packed { logic [1:0] a; } fld_t;
  typedef struct packed { fld_t w1; fld_t w0; } two_t;
endpackage
module leaf #(parameter WIDTH = 99) (input wire clk, output int w_out);
  assign w_out = WIDTH;
endmodule
module mid import pk::*; (
  input clk, input two_t si, output two_t so,
  output int o_m, output int o_w, output int o_c);
  leaf #(.WIDTH($bits(si.w0))) u_m (.clk(clk), .w_out(o_m));
  leaf #(.WIDTH($bits(si)))    u_w (.clk(clk), .w_out(o_w));
  leaf #(.WIDTH(2))            u_c (.clk(clk), .w_out(o_c));
endmodule
module top;
  import pk::*;
  logic clk = 0;
  two_t si, so;
  int wm, ww, wc;
  mid d (clk, si, so, wm, ww, wc);
endmodule
"#;
    let sim = simulate(src, 20).expect("simulate failed");
    assert_eq!(u(&sim, "wm"), 2, "override from $bits(port.member)");
    assert_eq!(u(&sim, "ww"), 4, "override from $bits(port)");
    assert_eq!(u(&sim, "wc"), 2, "literal override, the control");
}

/// End-to-end: a bus whose WIDTH comes from `$bits` of a struct member, driven
/// bit-by-bit by generated sub-instances into a struct-member output. This is
/// the CDC-synchronizer shape reduced to its skeleton, and the assertion that
/// actually failed — every output bit was x.
#[test]
fn a_bus_sized_by_bits_of_a_struct_member_carries_data() {
    let src = r#"
`timescale 1ns/1ns
module bitcell (input wire clk, input wire d, output logic q);
  logic m;
  always @(posedge clk) begin m <= d; q <= m; end
endmodule
module vecwrap #(parameter WIDTH = 10) (
  input wire clk, input wire [WIDTH-1:0] din, output wire [WIDTH-1:0] dout);
  logic [WIDTH-1:0] sel;
  always_comb sel = din;
  genvar i;
  generate for (i = 0; i < WIDTH; i++) begin : g
    bitcell u (.clk(clk), .d(sel[i]), .q(dout[i]));
  end endgenerate
endmodule
package pk;
  typedef struct packed { logic [9:0] a; } fld_t;
  typedef struct packed { fld_t w1; fld_t w0; } two_t;
endpackage
module dut import pk::*; (input clk, input two_t si, output two_t so);
  vecwrap #(.WIDTH($bits(si.w0))) u0 (.clk(clk), .din(si.w0), .dout(so.w0));
  vecwrap #(.WIDTH($bits(si.w1))) u1 (.clk(clk), .din(si.w1), .dout(so.w1));
endmodule
module top;
  import pk::*;
  logic clk = 0;
  always #5 clk = ~clk;
  two_t si, so;
  int unk, got_w0, got_w1;
  dut d (clk, si, so);
  initial begin
    si = '0;
    repeat (4) @(posedge clk);
    si.w0 = 10'h155; si.w1 = 10'h2AA;
    repeat (4) @(posedge clk); #1;
    unk    = $isunknown(so);
    got_w0 = so.w0;
    got_w1 = so.w1;
  end
endmodule
"#;
    let sim = simulate(src, 400).expect("simulate failed");
    assert_eq!(u(&sim, "unk"), 0, "no bit of the output may be x");
    assert_eq!(u(&sim, "got_w0"), 0x155, "first bus carries its data");
    assert_eq!(u(&sim, "got_w1"), 0x2AA, "second bus too");
}

/// The guard: `$bits` of a PARAMETER keeps using the parameter's own width, and
/// `$bits` of a type name still resolves through the typedef table. A signal
/// and a parameter can share a name across scopes, so the prebind must not
/// hijack either.
#[test]
fn bits_of_parameters_and_type_names_is_unchanged() {
    let src = r#"
module top;
  typedef logic [11:0] wide_t;
  parameter  logic [6:0] P = 7'h5;
  localparam Q = $bits(P);
  localparam R = $bits(wide_t);
  localparam S = $bits(3'b101);
  int q, r, s;
  initial begin q = Q; r = R; s = S; end
endmodule
"#;
    let sim = simulate(src, 20).expect("simulate failed");
    assert_eq!(
        u(&sim, "q"),
        7,
        "$bits of a parameter is its declared width"
    );
    assert_eq!(u(&sim, "r"), 12, "$bits of a type name");
    assert_eq!(u(&sim, "s"), 3, "$bits of a sized literal");
}
