//! §6.20.2: a parameter value cannot depend on itself, directly or through
//! other parameters, and every name in it must be declared, even in an
//! operand the value does not select. The reference simulator rejects the
//! same modules and runs the legal one with the same output. (An acyclic
//! forward reference stays accepted: see forward_referenced_parameter.)

use xezim::simulate;

#[test]
fn parameter_value_names_later_or_undeclared() {
    for body in [
        "parameter PARAMB = PARAMB + 6;",
        "parameter PARAMB = PARAMA;\n  parameter PARAMA = PARAMB;",
        "parameter real PARAMB = PARAMB + 1.0;",
        "parameter y = 1;\n  parameter a = 0;\n  parameter x = y ? a : b;",
    ] {
        let src = format!("module top;\n  {body}\n  initial $display(\"x\");\nendmodule\n");
        assert!(simulate(&src, 10).is_err(), "accepted:\n{src}");
    }
}

#[test]
fn parameter_values_in_declaration_order() {
    let src = r#"
package p;
  parameter PW = 7;
endpackage
module m #(parameter A = 2, B = A * 3) ();
  import p::*;
  localparam C = B + PW;
  initial $display("R|%0d %0d %0d", A, B, C);
endmodule
module top;
  typedef enum {IDLE, RUN} st_t;
  parameter st_t S = RUN;
  parameter W = 4, V = W + 1;
  localparam X = V * 2 + $bits(st_t);
  m #(.A(1)) u();
  initial $display("R|%0d %0d", S, X);
endmodule
"#;
    let sim = simulate(src, 10).expect("ordered parameter values are legal");
    let mut lines: Vec<&str> = sim
        .output
        .iter()
        .filter_map(|o| o.message.strip_prefix("R|"))
        .collect();
    lines.sort();
    assert_eq!(lines, ["1 3 10", "1 42"], "{:?}", sim.output);
}
