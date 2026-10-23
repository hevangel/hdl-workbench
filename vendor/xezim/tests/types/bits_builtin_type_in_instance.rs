//! §20.6.2: `$bits(<built-in type>)` inside an INSTANTIATED module (ivtest
//! `sv_type_param6`). The check for names declared nowhere in the design,
//! run over inlined instance bodies, took the type keyword `integer` in the
//! type-argument position for an undeclared identifier and aborted
//! elaboration. Cross-checked against the reference simulator.

#[test]
fn bits_of_builtin_types_in_a_child_module() {
    let sim = xezim::simulate(
        r#"
module m #(parameter type T = logic) ();
  T x;
  initial $display("B|%0d %0d %0d %0d", $bits(x), $bits(integer), $bits(T), $bits(byte));
endmodule
module top;
  m #(.T(integer)) u();
endmodule
"#,
        10,
    )
    .expect("simulate");
    let o: Vec<String> = sim.output.iter().map(|o| o.message.clone()).collect();
    assert!(o.iter().any(|l| l == "B|32 32 32 8"), "{o:?}");
}
