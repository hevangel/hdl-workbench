//! Recursive nonvacuity and assertion action controls (§16.14.8, §20.12).
fn lines(source: &str) -> Vec<String> {
    let result = xezim::simulate(source, 100_000).expect("simulation failed");
    let lines: Vec<_> = result.output.iter().map(|o| o.message.clone()).collect();
    assert!(!lines.iter().any(|s| s.starts_with("FAIL")), "{lines:?}");
    lines
        .into_iter()
        .filter(|s| s.starts_with("T|") || s == "PASS")
        .collect()
}

#[test]
fn original_nested_implication_and_property_if() {
    let output = lines(include_str!("../issue_cases/issue445_original.sv"));
    assert_eq!(output, ["PASS"]);
}

/// Check each input combination separately so errors cannot cancel in totals.
#[test]
fn nested_vacuity_truth_table_and_action_controls() {
    for values in 0..16 {
        let a = values & 1 != 0;
        let b = values & 2 != 0;
        let c = values & 4 != 0;
        let d = values & 8 != 0;
        let pair_fail = (a && !b) || (c && !d);
        let outcomes = [
            (c && a && b, c && a && !b),
            (c && a && b, c && a && !b),
            (
                if c { a && b } else { d && b },
                if c { a && !b } else { d && !b },
            ),
            (c && d && a && b, c && d && a && !b),
            (!pair_fail && ((a && b) || (c && d)), pair_fail),
            // §16.14.8: `(a |-> b) or (c |-> d)` is nonvacuous whenever either
            // operand's attempt is nonvacuous and the `or` holds. The left
            // implication is nonvacuous iff `a`, the right iff `c`; the `or`
            // fails only when both operands do.
            (
                (a || c) && !((a && !b) && (c && !d)),
                (a && !b) && (c && !d),
            ),
        ];
        for control in [
            "$assertvacuousoff",
            "$assertpasson",
            "$assertpassoff; $assertnonvacuouson",
        ] {
            let source = format!(
                r#"
module top;
  logic clk = 0, a = {a}, b = {b}, c = {c}, d = {d};
  int passes[6], failures[6];
  initial begin {control}; end
  ap0: assert property (@(posedge clk) c |-> (a |-> b)) passes[0]++; else failures[0]++;
  ap1: assert property (@(posedge clk) if (c) (a |-> b)) passes[1]++; else failures[1]++;
  ap2: assert property (@(posedge clk) if (c) (a |-> b) else (d |-> b)) passes[2]++; else failures[2]++;
  ap3: assert property (@(posedge clk) c |-> (d |-> (a |-> b))) passes[3]++; else failures[3]++;
  ap4: assert property (@(posedge clk) (a |-> b) and (c |-> d)) passes[4]++; else failures[4]++;
  ap5: assert property (@(posedge clk) (a |-> b) or (c |-> d)) passes[5]++; else failures[5]++;
  initial begin
    #5 clk = 1;
    #1;
    for (int k = 0; k < 6; k++) $display("T|%0d|%0d %0d", k, passes[k], failures[k]);
    $finish;
  end
endmodule
"#,
                a = u8::from(a),
                b = u8::from(b),
                c = u8::from(c),
                d = u8::from(d)
            );
            let expected: Vec<_> = outcomes
                .iter()
                .enumerate()
                .map(|(k, &(nonvacuous, failed))| {
                    let pass = if control == "$assertpasson" {
                        !failed
                    } else {
                        nonvacuous
                    };
                    format!("T|{k}|{} {}", u8::from(pass), u8::from(failed))
                })
                .collect();
            assert_eq!(
                lines(&source),
                expected,
                "values={values}, control={control}"
            );
        }
    }
}

/// A nonvacuous consequent may finish before other antecedent matches or
/// delayed consequents. Retain it; a later failure must still take priority.
#[test]
fn ranged_antecedent_retains_recursive_nonvacuity_across_ticks() {
    for outer in ["|->", "|=>"] {
        for inner in ["|->", "|=>"] {
            let first = if outer == "|->" { 1 } else { 2 };
            for active in [0, first, first + 1, first + 2] {
                for success in [0, 1] {
                    let source = format!(
                        r#"
module top;
  logic clk = 0, launch = 0, a = 0, b = {success};
  int passes = 0, failures = 0, hits = 0;
  initial $assertvacuousoff;
  ap: assert property (@(posedge clk) (launch ##[0:2] 1'b1) {outer} (a {inner} b))
      passes++; else failures++;
  cp: cover property (@(posedge clk) (launch ##[0:2] 1'b1) {outer} (a {inner} b)) hits++;
  initial begin
    for (int tick = 0; tick < 8; tick++) begin
      launch = tick == 1;
      a = tick == {active};
      #5 clk = 1;
      #5 clk = 0;
    end
    $display("T|%0d %0d %0d", passes, failures, hits);
    $finish;
  end
endmodule
"#
                    );
                    let nonvacuous = active != 0 && success == 1;
                    let failed = active != 0 && success == 0;
                    let expected = format!(
                        "T|{} {} {}",
                        u8::from(nonvacuous),
                        u8::from(failed),
                        u8::from(nonvacuous)
                    );
                    assert_eq!(
                        lines(&source),
                        [expected],
                        "{outer} / {inner}, active={active}, b={success}"
                    );
                }
            }
        }
    }
}
