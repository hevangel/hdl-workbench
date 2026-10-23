//! §12.7.1: a for-loop variable declaration names a data type, so `var` with
//! only a range (`for (var [7:0] i = 0; ...)`) is illegal. The reference
//! simulator rejects it too, and runs the typed forms with the same output.

use xezim::simulate;

#[test]
fn var_without_data_type() {
    for init in ["var [7:0] i = 0", "var signed [7:0] i = 0"] {
        let src = format!(
            "module test;\n  initial for ({init}; i < 2; i++) $display(\"%0d\", i);\nendmodule\n"
        );
        assert!(simulate(&src, 10).is_err(), "accepted:\n{src}");
    }
}

#[test]
fn var_with_data_type() {
    let src = r#"
module test;
  initial begin
    for (var int i = 0; i < 2; i++) $display("F|%0d", i);
    for (var logic [7:0] j = 5; j < 6; j++) $display("F|%0d", j);
  end
endmodule
"#;
    let sim = simulate(src, 10).expect("typed loop variables are legal");
    let lines: Vec<&str> = sim
        .output
        .iter()
        .filter_map(|o| o.message.strip_prefix("F|"))
        .collect();
    assert_eq!(lines, ["0", "1", "5"], "{:?}", sim.output);
}
