//! §9.3.4 / §27.3: only a named block may carry an end label. The rejected
//! modules are rejected by the reference simulator too; the legal one prints
//! the same lines there.

use xezim::simulate;

#[test]
fn end_label_on_unnamed_block() {
    for body in [
        "initial begin end : label",
        "initial fork join : label",
        "generate if (1) begin end : label endgenerate",
        "genvar i; for (i = 0; i < 2; i++) begin end : label",
        "initial if (1) begin end else begin end : label",
    ] {
        let src = format!("module test;\n  {body}\nendmodule\n");
        assert!(simulate(&src, 10).is_err(), "accepted:\n{src}");
    }
}

#[test]
fn end_labels_on_named_blocks() {
    let src = r#"
module test;
  genvar i;
  for (i = 0; i < 2; i++) begin : g
  end : g
  if (1) begin : h
  end : h
  initial begin : a
    fork : f
      $display("L|fork");
    join : f
    lbl : begin
      $display("L|labelled");
    end : lbl
  end : a
endmodule
"#;
    let sim = simulate(src, 10).expect("named blocks may carry end labels");
    let lines: Vec<&str> = sim
        .output
        .iter()
        .filter_map(|o| o.message.strip_prefix("L|"))
        .collect();
    assert_eq!(lines, ["fork", "labelled"], "{:?}", sim.output);
}
