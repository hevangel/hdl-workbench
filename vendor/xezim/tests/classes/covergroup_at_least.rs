//! §19.7 `option.at_least` on automatic and cross bins: a bin counts as
//! covered once it has at least `at_least` hits. Only explicit bins used to
//! honour it; automatic coverpoint bins and cross bins counted as covered
//! on their first hit. Every expected number was cross-checked against the
//! reference simulator.

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

/// `x` = 0 once and 1 twice: of the four automatic bins only `1` reaches
/// two hits.
#[test]
fn at_least_on_automatic_bins() {
    let l = lines(
        r#"
module tb;
  bit [1:0] x;
  covergroup cg;
    option.at_least = 2;
    cp_x : coverpoint x;
  endgroup
  cg c = new();
  initial begin
    x = 0; c.sample();
    x = 1; c.sample();
    x = 1; c.sample();
    $display("auto %0.2f", c.cp_x.get_inst_coverage());
  end
endmodule
"#,
    );
    assert_line(&l, "auto 25.00");
}

/// The cross has two bins, (a0, b0) hit twice and (a1, b0) once.
#[test]
fn at_least_on_cross_bins() {
    let l = lines(
        r#"
module tb;
  bit [1:0] a, b;
  covergroup cg;
    option.at_least = 2;
    cp_a : coverpoint a { bins a0 = {0}; bins a1 = {1}; }
    cp_b : coverpoint b { bins b0 = {0}; }
    axb : cross cp_a, cp_b;
  endgroup
  cg c = new();
  initial begin
    a = 0; b = 0; c.sample(); c.sample();
    a = 1; b = 0; c.sample();
    $display("cp_a %0.2f cross %0.2f", c.cp_a.get_inst_coverage(), c.axb.get_inst_coverage());
  end
endmodule
"#,
    );
    assert_line(&l, "cp_a 50.00 cross 50.00");
}
