//! §27.6: a generate block's name is declared in the enclosing module, so it
//! cannot repeat a variable, another generate block or a named procedural
//! block there; alternative branches of one if-generate may share a name. The
//! reference simulator rejects the same modules and runs the legal one with
//! the same output.

use xezim::simulate;

#[test]
fn generate_block_name_declared_twice() {
    for body in [
        "reg named;\n  for (genvar gv = 0; gv < 1; gv = gv + 1) begin : named end",
        "for (genvar gv = 0; gv < 1; gv = gv + 1) begin : match end\n  \
         for (genvar gv = 0; gv < 1; gv = gv + 1) begin : match end",
        "localparam up = 1;\n  if (up) begin : block1 wire w; end\n  \
         initial begin : block1 reg r; end",
    ] {
        let src = format!("module test;\n  {body}\nendmodule\n");
        assert!(simulate(&src, 10).is_err(), "accepted:\n{src}");
    }
}

#[test]
fn distinct_generate_block_names() {
    let src = r#"
module test;
  localparam up = 1;
  if (up) begin : blk
    wire w = 1'b1;
  end else begin : blk
    wire w = 1'b0;
  end
  for (genvar gv = 0; gv < 2; gv = gv + 1) begin : loop
    wire v = gv[0];
  end
  initial begin : run
    #1 $display("N|%b %b", blk.w, loop[1].v);
  end
endmodule
"#;
    let sim = simulate(src, 10).expect("distinct block names are legal");
    assert!(
        sim.output.iter().any(|o| o.message == "N|1 1"),
        "{:?}",
        sim.output
    );
}
