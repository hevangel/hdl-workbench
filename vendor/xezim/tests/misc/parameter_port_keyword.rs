//! §6.20.1 / A.1.3: a parameter port declared without `parameter` or
//! `localparam` is a bare `name = value` or starts with a data type; a range
//! or signing alone needs the keyword. The reference simulator rejects the
//! same headers and runs the legal one with the same output.

use xezim::simulate;

#[test]
fn implicit_type_without_parameter_keyword() {
    for hdr in [
        "#([7:0] A = 1)",
        "#(signed A = 1)",
        "#(parameter A = 1, signed B = 2)",
        "#(parameter [7:0] A = 1, [7:0] B = 2)",
    ] {
        let src = format!("module test {hdr};\n  initial $display(\"%0d\", A);\nendmodule\n");
        assert!(simulate(&src, 10).is_err(), "accepted:\n{src}");
    }
}

#[test]
fn parameter_ports_without_keyword() {
    let src = r#"
module m #(A = 1, B = 2, int C = 3, logic [3:0] D = 4, parameter signed [7:0] E = -5);
  initial $display("P|%0d %0d %0d %0d %0d", A, B, C, D, E);
endmodule
module test;
  m u();
endmodule
"#;
    let sim = simulate(src, 10).expect("typed or bare parameter ports are legal");
    assert!(
        sim.output.iter().any(|o| o.message == "P|1 2 3 4 -5"),
        "{:?}",
        sim.output
    );
}
