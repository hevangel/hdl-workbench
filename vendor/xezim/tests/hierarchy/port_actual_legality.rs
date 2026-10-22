//! §23.3.3: an output or inout port connects to something that can be driven
//! — a net or variable, a select or concatenation of them — never a literal
//! or an operator expression, and an inout port connects to a net, not a
//! variable. The reference simulator rejects the same instances at
//! elaboration and runs the legal module with the same output.

use xezim::simulate;

const M: &str = "module m(output o, inout io);\n  assign o = 1'b1;\nendmodule\n";

#[test]
fn undrivable_port_actuals() {
    for inst in [
        "wire a, b;\n  m u1(.o(1'b0), .io(b));",
        "wire a, b;\n  logic v;\n  m u1(.o(a), .io(v));",
        "wire a;\n  m u1(.o(~a), .io());",
        "wire a, b;\n  m u1(a, ({2{b}}));",
    ] {
        let src = format!("{M}module test;\n  {inst}\nendmodule\n");
        assert!(simulate(&src, 10).is_err(), "accepted:\n{src}");
    }
    let src = "module m1(output reg [7:0] x);\nendmodule\n\
               module tb;\n  wire [3:0] y;\n  m1 foo({4'hx, y});\nendmodule\n";
    assert!(simulate(src, 10).is_err(), "accepted:\n{src}");
}

#[test]
fn drivable_port_actuals() {
    let src = r#"
module m(output [1:0] o, inout [1:0] io);
  assign o = 2'b10;
endmodule
module test;
  wire a, b;
  wire [1:0] w;
  logic [1:0] v;
  m u1(.o({a, b}), .io(w));
  m u2(.o(v), .io(({a, b})));
  initial #1 $display("P|%b %b %b", a, b, v);
endmodule
"#;
    let sim = simulate(src, 10).expect("drivable actuals are legal");
    assert!(
        sim.output.iter().any(|o| o.message == "P|1 0 10"),
        "{:?}",
        sim.output
    );
}
