//! §22.5.1: every formal of a macro is replaced in one pass over the body.
//! Substituting them one at a time rescanned the text an earlier actual had
//! inserted, so a string actual shifted the string-literal ranges and a later
//! formal with a default stayed in the expansion. Expected lines are the
//! reference simulator's.

use xezim::simulate;

#[test]
fn defaults_substitute_after_a_string_actual() {
    let src = r#"`define STOP_HERE
`define m(a0, a1=HERE, a2=HERE) \
  `ifdef STOP_``a1 \
    $display(a0); \
  `elsif STOP_``a2 \
    $display(a0, a1); \
  `else \
    $display(a0, a1, a2); \
  `endif
`define q(x, y=2) $display("Q|%0d %0d", x, y);
module top;
  initial begin
    `m("M|none");
    `m("M|%s", "hello");
    `m("M|%0d %s", 123, "hello");
    `q(1)
    `q("a" == "a", 5)
  end
endmodule
"#;
    let sim = simulate(src, 10).expect("simulate");
    let got: Vec<&str> = sim.output.iter().map(|o| o.message.as_str()).collect();
    assert_eq!(got, ["M|none", "M|hello", "M|123 hello", "Q|1 2", "Q|1 5"]);
}
