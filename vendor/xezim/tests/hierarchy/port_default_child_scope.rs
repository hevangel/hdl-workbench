//! §23.2.2.4: an input port's default value is a constant expression of the
//! MODULE that declares the port — `input logic i = F` reads that module's
//! own parameter `F`, per instance (ivtest `br_gh1230`). The default was
//! checked against, and substituted into, the INSTANTIATING scope, where `F`
//! does not exist: "must be a constant expression". Cross-checked against
//! the reference simulator.

#[test]
fn port_default_reads_the_instance_parameters() {
    let sim = xezim::simulate(
        r#"
module Foo #(parameter logic F = 1'b1, parameter int W = 4)
           (input logic i = F, input logic [7:0] v = W * 2 + 1);
  initial #1 $display("P|%m i=%b v=%0d", i, v);
endmodule
module Test;
  defparam g.F = 1'b0;
  Foo f(), g();
  Foo #(.F(1'bz), .W(10)) h();
endmodule
"#,
        10,
    )
    .expect("simulate");
    let mut o: Vec<String> = sim
        .output
        .iter()
        .map(|o| o.message.clone())
        .filter(|l| l.starts_with("P|"))
        .collect();
    o.sort();
    assert_eq!(
        o,
        vec!["P|Test.f i=1 v=9", "P|Test.g i=0 v=9", "P|Test.h i=z v=21"],
    );
}

#[test]
fn runtime_variable_default_is_still_rejected() {
    // ivtest sv_default_port_value3: a $unit variable is not a constant.
    let src = "reg [7:0] v;\n\
               module dut(input wire [7:0] i = v, output wire [7:0] o); assign o = i; endmodule\n\
               module tb; wire [7:0] r; dut d(, r); endmodule\n";
    assert!(xezim::simulate(src, 10).is_err());
}
