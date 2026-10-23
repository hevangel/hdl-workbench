//! §19.5.7: sampling a value of an `illegal_bins` is a run-time error. It
//! used to be a stderr note that `--error-exit` did not count, and the
//! message showed the value as a Rust debug `Some(7)`. Every expected
//! number was cross-checked against the reference simulator, which also
//! reports one error.

use xezim::simulate;

fn run(src: &str) -> (Vec<String>, u64) {
    let sim = simulate(src, 100_000).expect("simulate failed");
    let lines = sim
        .output
        .iter()
        .map(|o| o.message.trim().to_string())
        .collect();
    (lines, sim.error_count)
}

fn assert_line(lines: &[String], want: &str) {
    assert!(
        lines.iter().any(|l| l == want),
        "expected `{want}`, got {lines:?}"
    );
}

#[test]
fn illegal_coverpoint_bin_is_an_error() {
    let (l, errors) = run(r#"
module tb;
  bit [2:0] s;
  bit [3:0] data;
  covergroup cg;
    cp_s : coverpoint s { bins ok = {[0:5]}; illegal_bins bad = {7}; }
    coverpoint data;
    coverpoint s + 1;
  endgroup
  cg c = new();
  initial begin
    s = 1; c.sample();
    s = 7; c.sample();
    $display("cp_s %0.2f", c.cp_s.get_inst_coverage());
    $display("data %0.2f", c.data.get_inst_coverage());
    $display("cg   %0.2f", c.get_inst_coverage());
  end
endmodule
"#);
    assert_eq!(errors, 1, "{l:?}");
    assert!(
        l.iter()
            .any(|m| m.contains("Illegal bin") && m.contains("cp_s.bad") && m.contains("value 7")),
        "{l:?}"
    );
    assert_line(&l, "cp_s 100.00");
    assert_line(&l, "data 6.25");
    assert_line(&l, "cg   35.94");
}
