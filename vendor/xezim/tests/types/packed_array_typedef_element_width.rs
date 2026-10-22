//! §7.4.1: a packed array whose ELEMENT type comes from a typedef.
//!
//! `u32_t [4:0] SEC;` declares five 32-bit elements, so `SEC[0]` selects a
//! whole 32-bit element. The element width is *not* visible in the
//! declaration's dimensions — those carry only the array index range `[4:0]`;
//! the width lives in the typedef. The elaborator's element-width helper
//! returned `None` for a typedef'd element type, so the width defaulted to one
//! bit and `SEC[0] <= 32'hDEAD_BEEF` stored a single BIT (the value read back
//! as 1).
//!
//! The equivalent inline form `logic [4:0][31:0]` was always correct, which is
//! what disguised this: it looks like a packed-struct problem, because a packed
//! array member of a packed struct is the commonest way to write it.
//!
//! A second, independent gap sat behind the first: per-member element widths
//! are registered by the body-declaration paths but were missing on the PORT
//! path, so a struct carrying such a member through a module port lost the
//! metadata even once the typedef case worked. Both are covered here, since
//! fixing only one still leaves a hierarchical design broken. Reference-
//! validated.

use xezim::simulate;

fn u(sim: &xezim::compiler::Simulator, n: &str) -> u64 {
    sim.get_signal(n)
        .or_else(|| sim.get_signal(&format!("tb.{}", n)))
        .unwrap_or_else(|| panic!("signal not found: {}", n))
        .to_u64()
        .unwrap_or_else(|| panic!("{} not u64-able (x/z?)", n))
}

/// A bare packed array with a typedef'd element type, next to the inline form
/// that always worked.
#[test]
fn typedef_element_type_selects_whole_element() {
    let src = r#"
typedef logic [31:0] u32_t;
module tb;
  logic [4:0][31:0] inl;
  u32_t [4:0]       tdef;
  logic [31:0] a, b;
  initial begin
    inl = '0; tdef = '0;
    inl[0]  = 32'hDEAD_BEEF;
    tdef[0] = 32'hDEAD_BEEF;
    #1;
    a = inl[0];
    b = tdef[0];
  end
endmodule
"#;
    let sim = simulate(src, 100).expect("simulate failed");
    assert_eq!(u(&sim, "a"), 0xDEAD_BEEF, "inline packed dims");
    assert_eq!(u(&sim, "b"), 0xDEAD_BEEF, "typedef'd element type");
}

/// The same array as a member of a packed struct, written nonblockingly from a
/// clocked block — the reported shape.
#[test]
fn packed_struct_member_array_element_write() {
    let src = r#"
typedef logic [31:0] u32_t;
typedef struct packed {
  logic [7:0] REG_A;
  logic [7:0] REG_B;
  u32_t [4:0] REG_SEC;
} st_t;
module tb;
  logic clk = 0;
  logic rst = 1;
  st_t cnfg;
  logic [31:0] s0, s4;
  always #5 clk = ~clk;
  always_ff @(posedge clk) begin
    if (rst) begin
      cnfg.REG_A <= 8'h00; cnfg.REG_B <= 8'h00; cnfg.REG_SEC <= 'h0;
    end else begin
      if ($time == 35) cnfg.REG_SEC[0] <= 32'hDEAD_BEEF;
      if ($time == 55) cnfg.REG_SEC[4] <= 32'hCAFE_BABE;
    end
  end
  initial begin
    #20 rst = 0;
    #60;
    s0 = cnfg.REG_SEC[0];
    s4 = cnfg.REG_SEC[4];
  end
endmodule
"#;
    let sim = simulate(src, 300).expect("simulate failed");
    assert_eq!(u(&sim, "s0"), 0xDEAD_BEEF, "element 0");
    assert_eq!(u(&sim, "s4"), 0xCAFE_BABE, "element 4");
}

/// Through a module PORT, with the type coming from a package — the second gap.
/// The element writes happen inside the submodule and must be visible both on
/// the port and through the hierarchical path.
#[test]
fn struct_with_array_member_through_a_port() {
    let src = r#"
package spec_types;
  typedef logic [31:0] u32_t;
  typedef struct packed { logic [7:0] A; u32_t [4:0] SEC; } st_t;
endpackage
module leaf (input logic clk, input logic rst, output spec_types::st_t cnfg);
  always_ff @(posedge clk) begin
    if (rst) begin
      cnfg.A <= 8'h00; cnfg.SEC <= 'h0;
    end else begin
      if ($time == 35) cnfg.SEC[0] <= 32'hDEAD_BEEF;
      if ($time == 55) cnfg.SEC[4] <= 32'hCAFE_BABE;
    end
  end
endmodule
module tb;
  logic clk = 0, rst = 1;
  spec_types::st_t cnfg;
  logic [31:0] p0, p4, inner0;
  leaf u (.clk(clk), .rst(rst), .cnfg(cnfg));
  always #5 clk = ~clk;
  initial begin
    #20 rst = 0;
    #60;
    p0 = cnfg.SEC[0];
    p4 = cnfg.SEC[4];
    inner0 = u.cnfg.SEC[0];
  end
endmodule
"#;
    let sim = simulate(src, 300).expect("simulate failed");
    assert_eq!(u(&sim, "p0"), 0xDEAD_BEEF, "element 0 on the port");
    assert_eq!(u(&sim, "p4"), 0xCAFE_BABE, "element 4 on the port");
    assert_eq!(u(&sim, "inner0"), 0xDEAD_BEEF, "hierarchical read");
}

/// Two levels of hierarchy, with the whole member forwarded to a flat vector —
/// confirms the element layout, not just individual reads.
#[test]
fn array_member_layout_survives_two_levels() {
    let src = r#"
package spec_types;
  typedef logic [31:0] u32_t;
  typedef struct packed { logic [7:0] A; u32_t [4:0] SEC; } st_t;
endpackage
module spec (input logic clk, input logic rst, input logic wr,
             input logic [4:0] addr, input logic [31:0] wdata,
             output spec_types::st_t cnfg);
  always_ff @(posedge clk) begin
    if (rst) begin
      cnfg.A <= 8'h00; cnfg.SEC <= 'h0;
    end else if (wr) begin
      case (addr)
        5'd8:  cnfg.SEC[0] <= wdata;
        5'd24: cnfg.SEC[4] <= wdata;
        default: ;
      endcase
    end
  end
endmodule
module mid (input logic clk, input logic rst, input logic wr,
            input logic [4:0] addr, input logic [31:0] wdata,
            output logic [159:0] flat);
  spec_types::st_t cnfg;
  spec u_spec (.clk(clk), .rst(rst), .wr(wr), .addr(addr), .wdata(wdata), .cnfg(cnfg));
  always_ff @(posedge clk) flat <= cnfg.SEC;
endmodule
module tb;
  logic clk = 0, rst = 1, wr = 0;
  logic [4:0] addr = 0;
  logic [31:0] wdata = 0;
  wire [159:0] flat;
  logic [31:0] lo, hi;
  mid u (.clk(clk), .rst(rst), .wr(wr), .addr(addr), .wdata(wdata), .flat(flat));
  always #5 clk = ~clk;
  initial begin
    #20 rst = 0;
    @(negedge clk); wr = 1; addr = 5'd8;  wdata = 32'hDEAD_BEEF;
    @(negedge clk);        addr = 5'd24; wdata = 32'hCAFE_BABE;
    @(negedge clk); wr = 0;
    repeat (3) @(posedge clk);
    #1;
    lo = flat[31:0];
    hi = flat[159:128];
  end
endmodule
"#;
    let sim = simulate(src, 400).expect("simulate failed");
    assert_eq!(
        u(&sim, "lo"),
        0xDEAD_BEEF,
        "element 0 lands in the low bits"
    );
    assert_eq!(
        u(&sim, "hi"),
        0xCAFE_BABE,
        "element 4 lands in the high bits"
    );
}
