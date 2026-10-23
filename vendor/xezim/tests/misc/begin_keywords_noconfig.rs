//! §22.14: under `begin_keywords "1364-2001-noconfig"` the configuration
//! keywords (`instance`, `cell`, `design`, `use`, …) are ordinary
//! identifiers. Expected line is the reference simulator's.

use xezim::simulate;

#[test]
fn config_keywords_are_identifiers_in_a_noconfig_region() {
    let src = r#"`begin_keywords "1364-2001-noconfig"
module sub(input a, output b);
  assign b = ~a;
endmodule
module top;
  reg cell, design;
  wire use;
  sub instance(cell, use);
  initial begin
    cell = 0; design = 1;
    #1 $display("K|%b %b %b", cell, design, use);
  end
endmodule
`end_keywords
"#;
    let sim = simulate(src, 10).expect("simulate");
    assert!(
        sim.output.iter().any(|o| o.message == "K|0 1 1"),
        "{:?}",
        sim.output
    );
}
