//! §6.20.1/§23.10: an override (named, or a defparam) must name an
//! overridable parameter — with a parameter port list, a body `parameter` is
//! local; a type parameter takes a type and no defparam; a header parameter
//! without a default must be overridden by every instance. Each rejected case
//! is also rejected by the reference simulator, and the legal design prints
//! the same lines there.

use xezim::simulate;

fn rejected(src: &str) -> String {
    match simulate(src, 100) {
        Ok(_) => panic!("accepted an illegal design:\n{src}"),
        Err(e) => e,
    }
}

#[test]
fn illegal_parameter_overrides() {
    for src in [
        "module a #(parameter A = 1); parameter B = 1; endmodule\n\
         module test; a #(.A(10), .B(20)) i_a(); endmodule",
        "module a; localparam A = 1; endmodule\n\
         module test; a i_a(); defparam i_a.A = 10; endmodule",
        "module a #(parameter A = 1); endmodule\n\
         module test; a i_a(); defparam i_a.Z = 10; endmodule",
        "module a #(parameter A = 1); parameter B = 2; endmodule\n\
         module test; a i_a(); defparam i_a.B = 20; endmodule",
        "module a #(parameter A); endmodule\nmodule test; a i_a(); endmodule",
        "module M #(type T = int); T x; endmodule\nmodule test; M #(.T(10)) m(); endmodule",
    ] {
        rejected(src);
    }
}

#[test]
fn legal_parameter_overrides() {
    let src = r#"
module a #(parameter A = 1, parameter B = 2); parameter C = 3;
  initial #1 $display("A|%0d %0d %0d", A, B, C);
endmodule
module c; parameter P = 1; parameter Q = 2;
  initial #2 $display("C|%0d %0d", P, Q);
endmodule
module d #(parameter N); initial #3 $display("D|%0d", N); endmodule
module test;
  a #(.B(7)) i_a();
  defparam i_a.A = 5;
  c #(.P(4)) i_c();
  defparam i_c.Q = 9;
  d #(3) i_d();
endmodule
"#;
    let sim = simulate(src, 10).expect("legal overrides must run");
    let got: Vec<&str> = sim
        .output
        .iter()
        .map(|o| o.message.as_str())
        .filter(|m| m.contains('|'))
        .collect();
    assert_eq!(got, ["A|5 7 3", "C|4 9", "D|3"]);
}
