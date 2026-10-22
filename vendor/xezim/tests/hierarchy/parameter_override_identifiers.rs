//! §23.10: a parameter value in an instantiation is a constant expression
//! over declared names. The reference simulator rejects an undeclared name
//! and runs the legal module with the same output.

use xezim::simulate;

#[test]
fn undeclared_parameter_value() {
    let src = r#"
module testbench;
  foo #(ASDF) bar();
endmodule
module foo #(parameter A = 1);
endmodule
"#;
    assert!(simulate(src, 10).is_err());
}

#[test]
fn declared_parameter_values() {
    let src = r#"
package p;
  parameter PW = 5;
endpackage
module foo #(parameter A = 1);
  initial $display("O|%0d", A);
endmodule
module bar #(parameter type T = int);
  initial $display("O|%0d", $bits(T));
endmodule
module testbench;
  import p::*;
  localparam L = 2;
  typedef logic [3:0] nib_t;
  foo #(L * 3) a();
  foo #(.A(PW)) b();
  bar #(nib_t) c();
endmodule
"#;
    let sim = simulate(src, 10).expect("declared parameter values are legal");
    let mut lines: Vec<&str> = sim
        .output
        .iter()
        .filter_map(|o| o.message.strip_prefix("O|"))
        .collect();
    lines.sort();
    assert_eq!(lines, ["4", "5", "6"], "{:?}", sim.output);
}
