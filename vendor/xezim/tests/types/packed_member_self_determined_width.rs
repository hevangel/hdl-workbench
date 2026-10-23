//! §11.6.1 self-determined width of a packed-struct MEMBER read.
//!
//! `expr_max_width` resolved an identifier only through the signal table, and
//! `req.addr` (a member of a packed-struct signal or port) is not a signal of
//! its own — the miss fell through to 0. Every SELF-DETERMINED use then took
//! the `.max(1)` floor, so inside a concatenation
//! `{(req.addr >> 5), lsbs}` the shift's left operand was resized to ONE BIT
//! and the whole term evaluated to 0; the concatenation returned just `lsbs`.
//! The same expression assigned directly (`x = req.addr >> 5`) was correct,
//! because there the assignment's own width drove the context — which is what
//! made this look like a concatenation bug rather than a width-inference one.
//! Values verified against the reference simulator.

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
package p;
  typedef struct packed {
    logic [31:0] addr;
    logic [7:0]  len;
    logic [2:0]  size;
    logic        write;
  } req_t;
  typedef struct packed { logic [31:0] vaddr; logic [15:0] bcnt; } out_t;
endpackage
module dut(input p::req_t req, input logic [4:0] lsbs, output p::out_t o);
  always_comb begin
    o = '0;
    o.vaddr = {(req.addr >> 5), lsbs};
  end
endmodule
module top;
  p::req_t req; logic [4:0] lsbs; p::out_t o;
  logic [36:0] wide;
  logic [31:0] shifted;
  logic [39:0] mixed;
  dut d(req, lsbs, o);
  always_comb wide    = {(req.addr >> 5), lsbs};
  always_comb shifted = req.addr >> 5;
  always_comb mixed   = {req.len, (req.addr >> 8), req.size};
  initial begin
    req = '0; req.addr = 32'hace1531e; req.len = 8'h12; req.size = 3'h3; req.write = 1'b1;
    lsbs = 5'h07;
    #1 $display("NOTE: %h %h %h %h", o.vaddr, wide, shifted, mixed);
    $finish;
  end
endmodule
"#;

#[test]
fn packed_member_keeps_its_width_in_self_determined_context() {
    // member=ace15307 (37-bit concat truncated to the 32-bit field),
    // wide=00ace15307 (full 37 bits), shifted=05670a98,
    // mixed={8'h12, (addr>>8) as 32 bits, 3'h3} truncated to 40 bits
    assert_eq!(
        notes(SRC),
        ["NOTE: ace15307 00ace15307 05670a98 9005670a9b"]
    );
}
