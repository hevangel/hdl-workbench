//! §8.25: the generic declaration of a type-parameterized class is its
//! default specialization, so inside it a type parameter stands for its
//! default type. `$bits(DT)` in a class constant read 0 there (sv-tests
//! `class_test_52` shape): `localparam W = $bits(DT)` was 0 and
//! `{$bits(DT) - 1{1'b1}}` replicated 2^32-1 copies, clamped with a width
//! warning. Cross-checked against the reference simulator.

#[test]
fn default_specialization_constants_use_the_default_type() {
    let sim = xezim::simulate(
        r#"
class base; endclass
class how_wide #(type DT=int) extends base;
  localparam Max_int = {$bits(DT) - 1{1'b1}};
  localparam W = $bits(DT);
endclass
module tb;
  initial $display("M|%0d %0d", how_wide#()::Max_int, how_wide#()::W);
endmodule
"#,
        10,
    )
    .expect("simulate");
    let o: Vec<String> = sim.output.iter().map(|o| o.message.clone()).collect();
    assert!(o.iter().any(|l| l == "M|2147483647 32"), "{o:?}");
}
