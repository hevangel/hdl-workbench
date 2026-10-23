//! End-of-simulation assertion results (§16.12.2–3).
fn check(source: &str, expected: &[&str]) {
    let result = xezim::simulate(source, 100_000).unwrap();
    let lines: Vec<_> = result
        .output
        .iter()
        .map(|entry| entry.message.as_str())
        .collect();
    assert!(
        !lines.iter().any(|line| line.starts_with("FAIL")),
        "{lines:?}"
    );
    for line in expected {
        assert!(lines.contains(line), "missing {line:?}: {lines:?}");
    }
}

const PENDING: &str = include_str!("../issue_cases/issue393_pending_neighbors.sv");

#[test]
fn issue393_original() {
    check(
        include_str!("../issue_cases/issue393_original.sv"),
        &["PASS"],
    );
}

#[test]
fn pending_negation_is_settled_before_finish_final_blocks() {
    let source =
        include_str!("../issue_cases/issue393_original.sv").replace("#10;", "#10 $finish;");
    check(&source, &["PASS"]);
}

#[test]
fn pending_negation_composes_with_property_operators() {
    check(
        PENDING,
        &["T|pending|'{1, 1, 0, 0, 1, 1, 1, 0, 0, 1, 0, 0, 1, 0, 1, 1, 0, 0, 0}"],
    );
}

#[test]
fn resumed_clock_distinguishes_failed_and_pending_operands() {
    let source = PENDING.replace("#10;", "a = 0; #5 clk = 1; #5 clk = 0; #5;");
    check(
        &source,
        &["T|pending|'{0, 0, 0, 2, 2, 0, 0, 0, 0, 0, 0, 0, 1, 0, 2, 0, 0, 0, 0}"],
    );
}

#[test]
fn disable_pulse_cancels_negation_without_another_clock_tick() {
    let source = PENDING.replace("#10;", "#2 reset = 0; #8;");
    check(
        &source,
        &["T|pending|'{1, 1, 0, 0, 1, 1, 1, 0, 0, 1, 0, 0, 1, 0, 1, 1, 0, 0, 0}"],
    );
}

#[test]
fn finish_evaluates_disable_function_in_its_instance_scope() {
    check(
        r#"
module observer(input logic clk, reset);
  int failures = 0;
  function automatic bit disabled_now(); return reset; endfunction
  ap: assert property (@(posedge clk) disable iff (disabled_now())
      not (1'b1 ##1 1'b0)) else failures++;
  final $display("T|guard|%0d", failures);
endmodule
module top;
  logic clk = 0, reset = 0;
  observer dut(clk, reset);
  initial begin #5 clk = 1; #5 reset = 1; $finish; end
endmodule
"#,
        &["T|guard|0"],
    );
}
