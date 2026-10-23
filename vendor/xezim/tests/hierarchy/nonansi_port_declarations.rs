//! §23.2.2.1: every name in a non-ANSI port list is declared in the module
//! body with a port direction. The reference simulator rejects a port with no
//! declaration or with only a net declaration, and runs the legal module with
//! the same output.

use xezim::simulate;

#[test]
fn port_without_declaration() {
    let src = r#"
module tb;
  wire [3:0] a, y;
  test uut (.a(a), .y(y));
endmodule
module test(a, b, y);
  input  [3:0] a;
  output [3:0] y;
  assign y = a;
endmodule
"#;
    assert!(simulate(src, 10).is_err());
    let src = "module m(a, b); input a; wire b; endmodule\n\
               module tb; wire x, y; m u(x, y); endmodule\n";
    assert!(simulate(src, 10).is_err());
}

#[test]
fn ports_declared_in_body() {
    let src = r#"
module m(a, b, , y);
  input [3:0] a;
  input b;
  wire b;
  output [3:0] y;
  assign y = b ? a : 4'd0;
endmodule
module tb;
  reg [3:0] a = 4'd6;
  wire [3:0] y;
  m u(a, 1'b1, , y);
  initial #1 $display("D|%0d", y);
endmodule
"#;
    let sim = simulate(src, 10).expect("declared ports are legal");
    assert!(
        sim.output.iter().any(|o| o.message == "D|6"),
        "{:?}",
        sim.output
    );
}
