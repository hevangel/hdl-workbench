//! §6.24.1: a cast whose type is a PACKAGE-SCOPED name — `pkg::T'(v)` for a
//! typedef, `pkg::W'(v)` for a constant size — was parsed as a plain
//! parenthesis and dropped, so `pkg::nib_t'(8'hAB)` kept all eight bits and
//! `pkg::pint_t'(a + b)` summed two 8-bit operands at 8 bits. It now resolves
//! through the qualified `pkg::T` registration like a bare `T'(v)`, in
//! procedural code, continuous assigns and constant expressions.
//!
//! Every expectation was cross-checked against the reference simulator.

use xezim::simulate;

#[test]
fn package_scoped_casts_convert() {
    let src = r#"
package fbp;
  typedef int pint_t;
  typedef logic [11:0] p12_t;
  typedef logic [3:0] pnib_t;
  localparam int PW = 12;
endpackage
module top;
  localparam bit [7:0] PA = 200;
  localparam bit [7:0] PB = 100;
  localparam int C1 = fbp::pint_t'(PA + PB);
  localparam int C2 = fbp::pnib_t'(PA + PB);
  localparam int C3 = fbp::PW'(PA + PB);
  bit [7:0] a = 200, b = 100;
  wire [31:0] w1 = fbp::pint_t'(a + b);
  wire [31:0] w2 = fbp::pnib_t'(a + b);
  initial begin
    $display("A %0d %0d %0d %0d", fbp::pint_t'(a + b), fbp::p12_t'(a + b), fbp::pnib_t'(a + b), fbp::PW'(a + b));
    $display("B %0d %0d %0d", fbp::pnib_t'(8'hAB), $bits(fbp::pint_t'(a)), fbp::pint_t'(a + b) > 255);
    $display("C %0d %0d %0d", C1, C2, C3);
    #1 $display("D %0d %0d", w1, w2);
  end
endmodule
"#;
    let sim = simulate(src, 10).expect("simulate failed");
    let o: Vec<String> = sim.output.iter().map(|o| o.message.clone()).collect();
    for want in ["A 300 300 12 300", "B 11 32 1", "C 300 12 300", "D 300 12"] {
        assert!(
            o.iter().any(|l| l.trim() == want),
            "missing {want:?} in {o:?}"
        );
    }
}
