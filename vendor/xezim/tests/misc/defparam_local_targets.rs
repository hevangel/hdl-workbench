//! §23.10 / §27.2: a defparam that names a parameter of its own module must
//! name an overridable one, and a parameter declared in a generate block is a
//! local parameter no defparam can override. The reference simulator rejects
//! both and runs the legal module with the same output.

use xezim::simulate;

#[test]
fn defparam_of_missing_or_local_parameter() {
    let src = r#"
module main;
  test tt();
  defparam foo = 3;
endmodule
module test;
  parameter foo = 10;
  reg [foo-1:0] bar;
endmodule
"#;
    assert!(simulate(src, 10).is_err());
    let src = r#"
module test;
  genvar i;
  for (i = 0; i < 2; i = i + 1) begin : loop
    parameter A = i;
    reg [A:0] r = A+1;
  end
  defparam loop[0].A = 10;
endmodule
"#;
    assert!(simulate(src, 10).is_err());
    let src = r#"
module test;
  generate
    genvar i;
    for (i = 0; i < 2; i = i + 1) begin : loop
      parameter A = i;
      reg [A:0] r = A+1;
    end
  endgenerate
  defparam loop[1].A = 20;
endmodule
"#;
    assert!(simulate(src, 10).is_err());
}

#[test]
fn defparam_of_instance_parameter() {
    let src = r#"
module main;
  parameter own = 1;
  test tt();
  defparam tt.foo = 3;
  defparam own = 2;
  initial #1 $display("D|%0d", tt.foo);
endmodule
module test;
  parameter foo = 10;
endmodule
"#;
    let sim = simulate(src, 10).expect("instance defparams are legal");
    assert!(
        sim.output.iter().any(|o| o.message == "D|3"),
        "{:?}",
        sim.output
    );
}
