//! §23.2.2.1: a ranged non-ANSI subroutine port completed by a vector
//! declaration of the same range (`input [3:0] x; reg [3:0] x;`) is one port,
//! not a port shadowed by a local variable. Outputs match the reference
//! simulator.

use xezim::simulate;

#[test]
fn ranged_port_and_vector_declaration() {
    let src = r#"
module test;
  task t;
    input [3:0] x;
    reg [3:0] x;
    output [7:0] y;
    reg [7:0] y;
    y = {x, x};
  endtask
  function [7:0] f;
    input signed [7:0] a;
    reg signed [7:0] a;
    f = a >>> 1;
  endfunction
  reg [7:0] q;
  initial begin
    t(4'd5, q);
    $display("R|%h %0d", q, $signed(f(-8'sd6)));
  end
endmodule
"#;
    let sim = simulate(src, 10).expect("ranged non-ANSI ports are legal");
    assert!(
        sim.output.iter().any(|o| o.message == "R|55 -3"),
        "{:?}",
        sim.output
    );
}
