//! §23.2.2.1: a non-ANSI port list may name a port by a port expression —
//! `.name(expr)`, a part-select or a concatenation — connecting the port to
//! part of the module's signals. Outputs match the reference simulator.

use xezim::simulate;

fn lines(src: &str) -> Vec<String> {
    let sim = simulate(src, 100).expect("port expressions must elaborate");
    sim.output
        .iter()
        .filter_map(|o| o.message.strip_prefix("P|").map(str::to_string))
        .collect()
}

#[test]
fn named_port_expression_input() {
    let src = r#"
module dummy (.B(A[2:1]));
  input [2:1] A;
  always @(A) if ($time > 0) $display("P|%b", A);
endmodule
module test;
  reg [2:0] A;
  dummy d(A[1:0]);
  initial begin
    #1 A = 3'b001;
    #1 A = 3'b110;
    #1 $finish(0);
  end
endmodule
"#;
    assert_eq!(lines(src), ["01", "10"]);
}

#[test]
fn part_select_ports() {
    let src = r#"
module top;
  reg [23:0] in1;
  reg [54:0] in2;
  initial begin
    in1 = 24'hfc0fc0;
    in2 = 55'h07c1f07c1f07c1;
    #1 $display("P|%h", dut.arg);
  end
  test dut(in1, in2);
endmodule
module test(arg[119:96], arg[78:24]);
  input [119:24] arg;
endmodule
"#;
    assert_eq!(lines(src), ["fc0fc0zzzzZ7c1f07c1f07c1"]);
}

#[test]
fn concatenation_and_null_ports() {
    let src = r#"
module c(.a({b, c}), );
  input [3:0] b;
  input c;
  initial #1 $display("P|%b %b", b, c);
endmodule
module port_3(dummy_1, , in[7:0], dummy_2, out[7:0], );
  input [7:0] in;
  output [7:0] out;
  output dummy_1;
  output dummy_2;
  assign out = in;
endmodule
module test;
  wire [4:0] w = 5'b10110;
  reg [7:0] data = 8'h5a;
  wire [7:0] out;
  c ci(.a(w));
  port_3 dut(, , data[7:0], , out[7:0], );
  initial #2 $display("P|%h", out);
endmodule
"#;
    assert_eq!(lines(src), ["1011 0", "5a"]);
}

#[test]
fn output_port_expression() {
    let src = r#"
module split(.hi(q[7:4]), .lo(q[3:0]), d);
  output [7:0] q;
  input [7:0] d;
  assign q = d;
endmodule
module test;
  wire [3:0] h, l;
  reg [7:0] d = 8'hc3;
  split s(h, l, d);
  initial #1 $display("P|%h %h", h, l);
endmodule
"#;
    assert_eq!(lines(src), ["c 3"]);
}
