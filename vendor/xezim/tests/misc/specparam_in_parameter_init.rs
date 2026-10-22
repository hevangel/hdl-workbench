//! §6.20.5: "parameter and localparam shall not be assigned a constant
//! expression that includes any specify parameters." The reciprocal direction
//! — a `specparam` reading a `parameter`/`localparam`, or another `specparam`
//! — is LEGAL and must keep elaborating.

use xezim::simulate;

#[test]
fn specparam_in_parameter_initializer_is_illegal() {
    let src = r#"
module top;
  specparam delay = 50;
  parameter p = delay + 2;
  initial $display("unreachable");
endmodule
"#;
    let err = match simulate(src, 10) {
        Err(e) => e,
        Ok(_) => panic!("a parameter may not read a specparam"),
    };
    assert!(err.contains("§6.20.5"), "unexpected diagnostic: {}", err);
    assert!(
        err.contains("specify parameter 'delay'"),
        "unexpected: {}",
        err
    );
}

#[test]
fn specparam_in_localparam_initializer_is_illegal() {
    let src = r#"
module top;
  specparam spd = 4'b0010;
  localparam lp = spd << 1;
  initial $display("unreachable");
endmodule
"#;
    let err = match simulate(src, 10) {
        Err(e) => e,
        Ok(_) => panic!("a localparam may not read a specparam"),
    };
    assert!(err.contains("§6.20.5"), "unexpected diagnostic: {}", err);
}

#[test]
fn legal_specparam_directions_are_fine() {
    // specparam reading a parameter (legal), specparam reading a specparam
    // (legal), and a parameter reading only parameters/localparams (legal).
    let src = r#"
module top;
  parameter int D = 2;
  specparam SPEC = D + 1;
  specparam SPEC2 = SPEC + 3;
  localparam int LOCAL = D + 1;
  parameter int P2 = D + LOCAL;
  initial begin
    if (SPEC == 3 && SPEC2 == 6 && LOCAL == 3 && P2 == 5)
      $display("OK|legal specparam directions");
    else
      $display("BAD|%0d %0d %0d %0d", SPEC, SPEC2, LOCAL, P2);
  end
endmodule
"#;
    let sim = simulate(src, 10).expect("legal specparam directions must elaborate");
    assert!(
        sim.output
            .iter()
            .any(|o| o.message == "OK|legal specparam directions"),
        "{:?}",
        sim.output
    );
}
// §6.20.5: declaration order and unrelated lexical scopes do not change the rule.
#[test]
fn forward_specparam_is_rejected() {
    let err = simulate("module top; parameter P=D+1; specparam D=3; endmodule", 10)
        .err()
        .expect("declaration must be rejected");
    assert!(err.contains("§6.20.5"), "{err}");
}
#[test]
fn header_parameter_may_not_read_specparam() {
    let err = simulate(
        "module top #(parameter P=D+1)(); specparam D=3; endmodule",
        10,
    )
    .err()
    .expect("declaration must be rejected");
    assert!(err.contains("§6.20.5"), "{err}");
}
#[test]
fn specparam_names_do_not_leak_between_modules() {
    let result = simulate(
        r#"module first; specparam D=3; endmodule
 module top; parameter D=4; localparam P=D+1; initial $display("T|scope %0d",P); endmodule"#,
        10,
    )
    .unwrap();
    assert!(result.output.iter().any(|o| o.message == "T|scope 5"));
}
#[test]
fn generate_local_constant_shadows_outer_specparam() {
    let result = simulate(
        r#"module top; specparam D=3;
 if(1) begin:g localparam D=4; localparam P=D+1; initial $display("T|scope %0d",P); end
 endmodule"#,
        10,
    )
    .unwrap();
    assert!(result.output.iter().any(|o| o.message == "T|scope 5"));
}
#[test]
fn unrelated_class_constant_is_not_a_specparam() {
    let result = simulate(
        r#"module first; specparam D=3; endmodule
 class c; localparam D=4; localparam P=D+1; endclass
 module top; initial $display("T|scope %0d",c::P); endmodule"#,
        10,
    )
    .unwrap();
    assert!(result.output.iter().any(|o| o.message == "T|scope 5"));
}
