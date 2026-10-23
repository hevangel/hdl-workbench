//! §6.18 / §23.10 — module-scope typedefs whose widths depend on PARAMETERS,
//! in a module instanced several times with different parameterizations.
//! Reference-validated.
//!
//! An inlined child's typedef registered only its BARE name in the global
//! width/type tables, computed with that instance's parameters — so the key
//! was last-writer-wins across instances. Variables survived (their widths
//! resolve during their own instance's pass, while the table briefly holds the
//! right value), which is what made everything look fine in a single-instance
//! MWE. Anything resolved LATER by name — a subroutine's return or formal
//! type, and the member types inside a stored struct typedef — read the LAST
//! instance's width.
//!
//! The field signature: a block instanced under several hierarchies with
//! different parameters simulates correctly alone, but in the full design one
//! instance's functions assemble values at another instance's widths — fields
//! land at wrong offsets in exactly one instance, and which instance depends
//! on elaboration order.
//!
//! Typedefs now also register under the instance-scoped key
//! `"<prefix><name>"`; subroutine signatures and the nested member references
//! inside a scoped typedef are renamed to those keys.

use xezim::simulate;

fn u(sim: &xezim::compiler::Simulator, n: &str) -> u64 {
    sim.get_signal(n)
        .or_else(|| sim.get_signal(&format!("tb.{}", n)))
        .unwrap_or_else(|| panic!("signal not found: {}", n))
        .to_u64()
        .unwrap_or_else(|| panic!("{} not u64-able (x/z?)", n))
}

/// A function returning a typedef'd width must size per instance — including
/// an instance with the SAME parameters under a different hierarchy.
#[test]
fn function_return_widths_are_per_instance() {
    let src = r#"
module blk #(parameter int W = 8) (input logic clk);
  typedef logic [W-1:0] word_t;
  int fbits, w_typedef;
  word_t v;
  function automatic word_t all_ones(); return '1; endfunction
  initial begin
    v = '1;
    w_typedef = $bits(v);
    fbits = $bits(all_ones());
  end
endmodule
module mid (input logic clk);
  blk #(.W(20)) deep(clk);
endmodule
module tb;
  logic clk = 0;
  blk #(.W(4))  b4(clk);
  blk #(.W(20)) b20(clk);
  mid           m(clk);
  int r4f, r4v, r20f, rdf;
  initial begin
    #1;
    r4f = b4.fbits; r4v = b4.w_typedef; r20f = b20.fbits; rdf = m.deep.fbits;
  end
endmodule
"#;
    let sim = simulate(src, 50).expect("simulate failed");
    assert_eq!(u(&sim, "r4v"), 4, "a variable of the typedef");
    assert_eq!(
        u(&sim, "r4f"),
        4,
        "the W=4 instance's function must not return 20 bits"
    );
    assert_eq!(u(&sim, "r20f"), 20);
    assert_eq!(u(&sim, "rdf"), 20, "same W under another hierarchy");
}

/// A param-dependent packed STRUCT typedef assembled by a function — the
/// member types are typedefs themselves, so the stored type must reference
/// the instance's own copies.
#[test]
fn struct_typedefs_assemble_at_their_own_widths() {
    let src = r#"
module splitter #(parameter int AW = 8, parameter int DW = 16)
  (input logic [AW-1:0] addr, input logic [DW-1:0] data,
   output logic [AW+DW:0] bus);
  typedef logic [AW-1:0] a_t;
  typedef logic [DW-1:0] d_t;
  typedef struct packed { logic wen; d_t d; a_t a; } pkt_t;
  function automatic pkt_t mk(input a_t a, input d_t d);
    mk.wen = 1'b1; mk.d = d; mk.a = a;
    return mk;
  endfunction
  pkt_t p;
  always_comb begin
    p = mk(addr, data);
    bus = p;
  end
endmodule
module wrap (input logic [3:0] a, input logic [7:0] d, output logic [12:0] b);
  splitter #(.AW(4), .DW(8)) s(a, d, b);
endmodule
module tb;
  logic [7:0]  a1; logic [15:0] d1; logic [24:0] b1;
  logic [3:0]  a2; logic [7:0]  d2; logic [12:0] b2;
  logic [3:0]  a3; logic [7:0]  d3; logic [12:0] b3;
  splitter #(.AW(8), .DW(16)) big(a1, d1, b1);
  splitter #(.AW(4), .DW(8))  sml(a2, d2, b2);
  wrap                        w(a3, d3, b3);
  int rb, rs, rd;
  initial begin
    a1 = 8'hA5; d1 = 16'h1234;
    a2 = 4'h7;  d2 = 8'h9C;
    a3 = 4'h3;  d3 = 8'h5E;
    #1;
    rb = b1; rs = b2; rd = b3;
  end
endmodule
"#;
    let sim = simulate(src, 50).expect("simulate failed");
    assert_eq!(
        u(&sim, "rb"),
        0x11234a5,
        "the wide instance packs at ITS widths"
    );
    assert_eq!(u(&sim, "rs"), 0x19c7, "the narrow one at its own");
    assert_eq!(u(&sim, "rd"), 0x15e3, "and the nested one at its own");
}
