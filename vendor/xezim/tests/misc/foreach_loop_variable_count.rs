//! §12.7.3: a foreach loop names at most one loop variable per dimension of
//! the array, counting its packed dimensions (an `int` element counts as one).
//! The reference simulator rejects the extra variables and runs the legal
//! loops with the same output.

use xezim::simulate;

#[test]
fn more_loop_variables_than_dimensions() {
    for body in [
        "logic a[10];\n  initial foreach (a[i, j]) $display(\"%0d\", i);",
        "logic [7:0] a[2];\n  initial foreach (a[i, j, k]) $display(\"%0d\", i);",
        "int b[2];\n  initial foreach (b[i, j, k]) $display(\"%0d\", i);",
    ] {
        let src = format!("module test;\n  {body}\nendmodule\n");
        assert!(simulate(&src, 10).is_err(), "accepted:\n{src}");
    }
}

#[test]
fn loop_variables_over_packed_dimensions() {
    let src = r#"
module test;
  logic [7:0] a[2];
  int b[2];
  initial begin
    foreach (a[i, j]) if (i == 1 && j == 7) $display("E|a %0d %0d", i, j);
    foreach (b[i, j]) if (i == 0 && j == 31) $display("E|b %0d %0d", i, j);
  end
endmodule
"#;
    let sim = simulate(src, 10).expect("one variable per dimension is legal");
    let lines: Vec<&str> = sim
        .output
        .iter()
        .filter_map(|o| o.message.strip_prefix("E|"))
        .collect();
    assert_eq!(lines, ["a 1 7", "b 0 31"], "{:?}", sim.output);
}
