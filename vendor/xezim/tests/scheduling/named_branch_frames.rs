//! IEEE 1800-2023 §9.3, §9.6.2, §18.16: a suspended branch keeps its
//! named block, including its `%m` scope and `disable` target.

use xezim::simulate;

fn lines(source: &str) -> Vec<String> {
    simulate(source, 100)
        .expect("simulate failed")
        .output
        .iter()
        .map(|line| line.message.clone())
        .filter(|line| line.starts_with("T|"))
        .collect()
}

#[test]
fn named_randcase_arm_preserves_scope_and_disable() {
    let source = r#"module top;
  initial begin
    randcase
      1: begin : chosen
        #1;
        $display("T|scope %m");
        disable chosen;
        $display("T|unreachable");
      end
    endcase
    $display("T|done");
  end
endmodule
"#;
    assert_eq!(lines(source), ["T|scope top.chosen", "T|done"]);
}

#[test]
fn named_suspended_branch_neighbors() {
    let source = include_str!("named_branch_frames.sv");
    // Expected lines were checked against the reference simulator.
    assert_eq!(lines(source), EXPECTED);
}

const EXPECTED: &[&str] = &[
    "T|selected top.selected",
    "T|after-selected top",
    "T|declared top.declared 23",
    "T|inner top.outer_scope.inner_scope",
    "T|outer top.outer_scope",
    "T|repeated top.repeated 0",
    "T|repeated top.repeated 1",
    "T|case top.case_scope",
    "T|unnamed top",
    "T|done top",
];
