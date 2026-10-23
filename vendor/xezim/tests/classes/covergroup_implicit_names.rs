//! §19.5 unlabeled coverpoints: one on a variable is named after the
//! variable, so `cg.data.get_inst_coverage()` queries it; one on any other
//! expression gets a generated name. The unlabeled coverpoint used to be
//! keyed by a debug dump of its expression, which changed once the
//! expression was evaluated, so it could read 0% after being hit, and the
//! results file showed the dump. Every expected number was cross-checked
//! against the reference simulator.

use xezim::simulate;

fn lines(src: &str) -> Vec<String> {
    let sim = simulate(src, 100_000).expect("simulate failed");
    sim.output
        .iter()
        .map(|o| o.message.trim().to_string())
        .collect()
}

fn assert_line(lines: &[String], want: &str) {
    assert!(
        lines.iter().any(|l| l == want),
        "expected `{want}`, got {lines:?}"
    );
}

/// `s + 1` is 32 bits wide: its 64 automatic bins hold 2^26 values each, so
/// 2 and 8 hit one bin.
#[test]
fn unlabeled_coverpoints() {
    let l = lines(
        r#"
module tb;
  bit [2:0] s;
  bit [3:0] data;
  covergroup cg1; coverpoint data; endgroup
  covergroup cg2; coverpoint s + 1; endgroup
  covergroup cg3; cp : coverpoint s + 1; endgroup
  cg1 c1 = new(); cg2 c2 = new(); cg3 c3 = new();
  initial begin
    s = 1; data = 1; c1.sample(); c2.sample(); c3.sample();
    s = 7; data = 2; c1.sample(); c2.sample(); c3.sample();
    $display("cg1 %0.2f  cg2 %0.2f  cg3 %0.2f", c1.get_inst_coverage(), c2.get_inst_coverage(), c3.get_inst_coverage());
    $display("data %0.2f", c1.data.get_inst_coverage());
  end
endmodule
"#,
    );
    assert_line(&l, "cg1 12.50  cg2 1.56  cg3 1.56");
    assert_line(&l, "data 12.50");
}
