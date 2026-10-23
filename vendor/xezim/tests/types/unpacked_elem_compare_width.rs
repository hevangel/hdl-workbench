//! §11.6.1: an unpacked-array ELEMENT select carries the element's width.
//!
//! The bytecode compiler's `expr_max_width` returned 1 for an index select
//! whose base is an unpacked array (only PACKED element widths were
//! consulted), so a comparison like `(addr_i[hs] & mask[d]) == base[d]` sized
//! both operands at max(1,1) and truncated the AND to a single bit. Ibex's bus
//! decoder never matched an address once its enclosing always_comb compiled —
//! every individual read printed correctly (the plain-copy path resizes to the
//! destination), which made the wrong compare maddening to localize. Sibling
//! of the interpreter-side `expr_max_width returned 1 for every index select`
//! fix from earlier.

use xezim::simulate;

fn notes(src: &str) -> Vec<String> {
    let sim = simulate(src, 1_000_000).expect("simulate failed");
    sim.output
        .iter()
        .map(|o| o.message.trim().to_string())
        .filter(|l| l.starts_with("NOTE:"))
        .collect()
}

const SRC: &str = r#"
module m (
  input  logic [7:0] addr_i [2],
  input  logic       which_i,
  output logic       hit_o,
  output logic [1:0] sel_o
);
  logic [0:0] hs;
  logic [7:0] mask [3];
  logic [7:0] base [3];
  assign mask[0]=8'hF0; assign base[0]=8'h10;
  assign mask[1]=8'hF0; assign base[1]=8'h20;
  assign mask[2]=8'hF0; assign base[2]=8'h30;
  always_comb hs = which_i;
  always_comb begin
    hit_o = 1'b0;
    sel_o = '0;
    for (integer d = 0; d < 3; d = d + 1) begin
      if ((addr_i[hs] & mask[d]) == base[d]) begin
        hit_o = 1'b1;
        sel_o = 2'(d);
      end
    end
  end
endmodule
module top;
  logic [7:0] addr [2]; logic which; logic hit; logic [1:0] sel;
  m u (.addr_i(addr), .which_i(which), .hit_o(hit), .sel_o(sel));
  initial begin
    which = 0; addr[0] = 8'h05; addr[1] = 8'h25;
    #1 $display("NOTE: A hit=%b sel=%0d", hit, sel);
    addr[0] = 8'h17;
    #1 $display("NOTE: B hit=%b sel=%0d", hit, sel);
    which = 1;
    #1 $display("NOTE: C hit=%b sel=%0d", hit, sel);
    addr[1] = 8'h39;
    #1 $display("NOTE: D hit=%b sel=%0d", hit, sel);
    $finish;
  end
endmodule
"#;

/// The ibex-bus decode shape: element selects of three different unpacked
/// arrays combined in one compare, plus a §6.24.1 size cast of the loop var.
#[test]
fn unpacked_element_selects_size_a_compare_correctly() {
    assert_eq!(
        notes(SRC),
        vec![
            "NOTE: A hit=0 sel=0",
            "NOTE: B hit=1 sel=0",
            "NOTE: C hit=1 sel=1",
            "NOTE: D hit=1 sel=2",
        ]
    );
}
