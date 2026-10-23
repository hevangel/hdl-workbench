//! Round 3 — two independent defects, both reference-validated.
//!
//! 1. **A struct member that is itself a packed array lost its element width
//!    when the struct was declared inside an INSTANTIATED module.** The
//!    top-level and PORT paths register per-member widths keyed
//!    `<signal>.<member>`; the sub-module BODY path did not. So
//!    `u32_t [4:0] REG_SEC;` inside a leaf module made `mcp.SEC[0] <= v` write
//!    ONE BIT. The identical declaration at top level always worked, which is
//!    exactly what disguised this as a port or NBA problem — a user testbench
//!    hit it three levels down a hierarchy.
//!
//! 2. **An untyped class parameter was sized to one bit.** §6.20.2: an untyped
//!    parameter takes the type of its value (an unsized decimal is signed
//!    32-bit). `parameter A = 1;` in a class stored `1'b1` and read back as -1
//!    inside a method, while `localparam B = 2;` truncated to 0. Reads from
//!    OUTSIDE the class looked right, which hid it. (ivtest sv_class_localparam)

use xezim::simulate;

fn u(sim: &xezim::compiler::Simulator, n: &str) -> u64 {
    sim.get_signal(n)
        .or_else(|| sim.get_signal(&format!("tb.{}", n)))
        .unwrap_or_else(|| panic!("signal not found: {}", n))
        .to_u64()
        .unwrap_or_else(|| panic!("{} not u64-able (x/z?)", n))
        & 0xFFFF_FFFF
}

/// The packed-array member must slice whole elements at every level: inside
/// the instance, on its output port, and on the parent's own variable.
#[test]
fn packed_array_struct_member_inside_an_instance() {
    let src = r#"
package p;
  typedef logic [31:0] u32_t;
  typedef struct packed {
    logic [7:0] A;
    logic [7:0] B;
    u32_t [4:0] SEC;
    logic [7:0] C;
    logic [7:0] D;
  } t;
endpackage
module leaf (input logic clk, output p::t cnfg);
  p::t mcp;
  always_ff @(posedge clk) begin
    mcp.SEC[0] <= 32'hDEADBEEF;
    mcp.SEC[4] <= 32'hCAFEBABE;
  end
  always_comb cnfg = mcp;
endmodule
module tb;
  logic clk = 0;
  always #5 clk = ~clk;
  p::t cw;
  leaf u(.clk(clk), .cnfg(cw));
  int int0, int4, port0, port4, par0, par4, bsec;
  initial begin
    @(posedge clk); @(posedge clk); #1;
    int0  = u.mcp.SEC[0];  int4  = u.mcp.SEC[4];
    port0 = u.cnfg.SEC[0]; port4 = u.cnfg.SEC[4];
    par0  = cw.SEC[0];     par4  = cw.SEC[4];
    bsec  = $bits(cw.SEC);
  end
endmodule
"#;
    let sim = simulate(src, 200).expect("simulate failed");
    assert_eq!(u(&sim, "int0"), 0xDEAD_BEEF, "instance-internal element 0");
    assert_eq!(u(&sim, "int4"), 0xCAFE_BABE, "instance-internal element 4");
    assert_eq!(u(&sim, "port0"), 0xDEAD_BEEF, "through the output port");
    assert_eq!(u(&sim, "port4"), 0xCAFE_BABE);
    assert_eq!(u(&sim, "par0"), 0xDEAD_BEEF, "parent-side variable");
    assert_eq!(u(&sim, "par4"), 0xCAFE_BABE);
    assert_eq!(u(&sim, "bsec"), 160, "5 x 32 bits");
}

/// §6.20.2: an untyped class parameter takes its value's type — visible from
/// inside a method, which is where the 1-bit sizing showed up.
#[test]
fn untyped_class_parameters_take_their_value_type() {
    let src = r#"
module tb;
  class C;
    parameter  A = 1;
    localparam B = 2;
    localparam W = 300;          // needs more than 8 bits
    function int ga(); return A; endfunction
    function int gb(); return B; endfunction
    function int gw(); return W; endfunction
    function bit chk(); return A == 1 && B == 2; endfunction
  endclass
  int a_in, b_in, w_in, chk_in, a_out, b_out;
  initial begin
    C c; c = new;
    a_in = c.ga(); b_in = c.gb(); w_in = c.gw(); chk_in = c.chk();
    a_out = c.A;   b_out = c.B;
  end
endmodule
"#;
    let sim = simulate(src, 50).expect("simulate failed");
    assert_eq!(
        u(&sim, "a_in"),
        1,
        "read inside a method (was -1: 1 bit, signed)"
    );
    assert_eq!(u(&sim, "b_in"), 2, "was 0: truncated to 1 bit");
    assert_eq!(u(&sim, "w_in"), 300, "wider value keeps its bits");
    assert_eq!(u(&sim, "chk_in"), 1, "comparisons inside the method");
    assert_eq!(u(&sim, "a_out"), 1, "read from outside still fine");
    assert_eq!(u(&sim, "b_out"), 2);
}
