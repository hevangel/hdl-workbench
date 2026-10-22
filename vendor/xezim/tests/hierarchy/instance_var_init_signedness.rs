//! §6.8: a variable declared in an instantiated module or interface takes
//! its DECLARED signedness; an initializer only supplies the value. The
//! instance path kept the signed-ness of the unsized literal, so `logic q =
//! 1;` read back through `u.q` as -1. Cross-checked against the reference
//! simulator.

use xezim::simulate;

#[test]
fn instance_variable_initializer_keeps_declared_signedness() {
    let src = r#"
module m1;
  logic q = 1;
  logic [3:0] w = 4'hf;
  int s = -2;
endmodule
interface i1;
  logic q = 1;
endinterface
module tb;
  m1 u();
  i1 v();
  initial #1 $display("%0d %0d %0d %0d", u.q, u.w, u.s, v.q);
endmodule
"#;
    let sim = simulate(src, 1_000).expect("simulate failed");
    let out: Vec<String> = sim.output.iter().map(|o| o.message.clone()).collect();
    assert_eq!(out, ["1 15 -2 1"]);
}
