//! §21.2.1.7: `%p` of a string-valued CALL or concatenation printed the
//! string's bytes as one huge decimal number, and `%0p` of any string kept
//! the quotes the reference drops.
//!
//! UVM AVIPs log their agent configs with
//! `$sformatf("cfg = \n %0p", cfg.sprint())`; every such table came out as a
//! 1000-digit integer. The typed `%p` renderers only know variables, so a
//! call or concatenation fell through to the packed-integer default. The
//! compact form now also prints a lone string without quotes. Every
//! expectation below was cross-checked against the reference simulator.

use xezim::simulate;

#[test]
fn string_calls_and_compact_strings() {
    let sim = simulate(
        r#"
class C;
  string nm = "cc";
  function string sprint();
    return {"line1\n", nm};
  endfunction
endclass
function string f();
  return "fx";
endfunction
module top;
  initial begin
    C c = new;
    string s = "ab";
    $display("A[%p][%0p]", s, s);
    $display("B[%p][%0p]", f(), f());
    $display("C[%p][%0p]", c.sprint(), c.sprint());
    $display("%s", $sformatf("D = \n %0p", c.sprint()));
    $display("E[%p]", {s, "z"});
  end
endmodule
"#,
        100,
    )
    .expect("simulate failed");
    let text = sim
        .output
        .iter()
        .map(|o| o.message.clone())
        .collect::<Vec<_>>()
        .join("\n");
    for w in [
        "A[\"ab\"][ab]",
        "B[\"fx\"][fx]",
        "C[\"line1\ncc\"][line1\ncc]",
        "D = \n line1\ncc",
        "E[\"abz\"]",
    ] {
        assert!(text.contains(w), "expected {w:?} in:\n{text}");
    }
}
