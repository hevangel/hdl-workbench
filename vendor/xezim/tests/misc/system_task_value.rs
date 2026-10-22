//! §20: a system task returns no value, so it cannot be used as a function.
//! The reference simulator rejects `val = $display;` at elaboration and runs
//! the legal module with the same output.

use xezim::simulate;

#[test]
fn system_task_in_expression() {
    for stmt in [
        "val = $display;",
        "val = $finish;",
        "if ($write(\"x\")) val = 1;",
    ] {
        let src = format!("module top;\n  integer val;\n  initial begin {stmt} end\nendmodule\n");
        assert!(simulate(&src, 10).is_err(), "accepted:\n{src}");
    }
}

#[test]
fn system_functions_in_expressions() {
    let src = r#"
module top;
  reg [79:0] str;
  integer val, n;
  initial begin
    str = "5";
    n = $sscanf(str, "%d", val);
    $display("F|%0d %0d %0d", n, val, $clog2(9));
  end
endmodule
"#;
    let sim = simulate(src, 10).expect("system functions are legal");
    assert!(
        sim.output.iter().any(|o| o.message == "F|1 5 4"),
        "{:?}",
        sim.output
    );
}
