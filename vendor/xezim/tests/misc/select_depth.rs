//! §7.4.6 / §11.5.1: a variable or net takes one select per dimension (an
//! `int` has one), so a bit-select of a scalar or a select past the last
//! dimension is an error. The reference simulator rejects the same selects
//! and runs the legal module with the same output.

use xezim::simulate;

#[test]
fn too_many_selects() {
    for body in [
        "logic s;\n  initial begin s = 1; $display(\"%b\", s[0]); end",
        "logic [7:0] v;\n  initial begin v = 1; $display(\"%b\", v[1][0]); end",
        "wire w;\n  assign w[0] = 1'b1;",
        "logic [7:0] m [2];\n  initial $display(\"%b\", m[0][1][0]);",
        "reg [15:0] memory[3:0];\n  reg [3:0] value;\n  initial value = memory[0][0][3:0];",
        "reg svar;\n  wire wsbs = svar[0];",
    ] {
        let src = format!("module test;\n  {body}\nendmodule\n");
        assert!(simulate(&src, 10).is_err(), "accepted:\n{src}");
    }
}

#[test]
fn one_select_per_dimension() {
    let src = r#"
module test;
  logic s;
  int i;
  integer g;
  logic [7:0] v;
  logic [7:0] m [2];
  logic [3:0][7:0] p;
  logic [7:0] q [2][3];
  initial begin
    s = 1; i = 5; g = 6; v = 8'h5a; m[0] = 8'h11; m[1] = 8'h22;
    p = 32'h12345678; q[1][2] = 8'hab;
    #1 $display("S|%b %b %b %b %h %b", i[0], g[1], v[1], m[1][1], m[0][7:4], s);
    $display("S|%h %h %b %h", p[1], p[2][7:4], p[3][0], q[1][2][7:4]);
  end
endmodule
"#;
    let sim = simulate(src, 10).expect("one select per dimension is legal");
    let lines: Vec<&str> = sim
        .output
        .iter()
        .filter_map(|o| o.message.strip_prefix("S|"))
        .collect();
    assert_eq!(lines, ["1 1 1 1 1 1", "56 3 0 a"], "{:?}", sim.output);
}
