//! §23.2.2.4: only an input port takes a default value; an inout port is a
//! net and cannot. The reference simulator rejects the inout default and runs
//! the input default with the same output.

use xezim::simulate;

#[test]
fn inout_port_with_default() {
    let src = r#"
module M (
  inout [31:0] x, y = 1
);
endmodule
module test;
  wire [31:0] x, y;
  M i_m (.x(x), .y(y));
endmodule
"#;
    assert!(simulate(src, 10).is_err());
}

#[test]
fn input_port_with_default() {
    let src = r#"
module M (input [7:0] x = 8'd7, inout [7:0] y);
  initial #1 $display("D|%0d", x);
endmodule
module test;
  wire [7:0] y;
  M i_m (.y(y));
endmodule
"#;
    let sim = simulate(src, 10).expect("input default is legal");
    assert!(
        sim.output.iter().any(|o| o.message == "D|7"),
        "{:?}",
        sim.output
    );
}
