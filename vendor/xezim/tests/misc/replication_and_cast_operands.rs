//! §11.4.12: a replication count is a known, non-negative constant, and a
//! zero-count replication stands only beside an operand of positive size; a
//! concatenation takes no real operand. §6.24.1: a casting size is positive;
//! `$signed`/`$unsigned` take an integral argument. Each rejected case is also
//! rejected by the reference simulator, and the legal module prints the same
//! line there.

use xezim::simulate;

#[test]
fn illegal_replications_and_casts() {
    for src in [
        "module top; reg [7:0] r; initial r = {0{1'b1}}; endmodule",
        "module top; reg [7:0] r; initial r = {{0{1'b1}}}; endmodule",
        "module top; reg [31:0] r; wire s = 1'b1; initial r = {{2{{0{s}}}}, 16'h0001}; endmodule",
        "module top; parameter wid = 9; wire [31:0] a; assign a = {(wid-16){8'b0}}; endmodule",
        "module top; parameter rep = 4'bx; wire [31:0] b = {rep{8'hab}}; endmodule",
        "module top; parameter real r1 = 1.0; parameter real r2 = 2.0;\n\
         parameter real r3 = {r1, r2}; endmodule",
        "module top; localparam integer N = 0; initial begin int x, y; y = N'(x); end endmodule",
        "module top; localparam integer N = 32'hx; initial begin int x, y; y = N'(x); end endmodule",
        "module top; reg [7:0] ival = $signed(1.0); endmodule",
        "module top; real r1, r2; reg in; assign {r1, r2} = in; endmodule",
    ] {
        assert!(simulate(src, 10).is_err(), "accepted:\n{src}");
    }
}

#[test]
fn legal_replications_and_casts() {
    let src = r#"
module top;
  localparam W = 8;
  localparam Z = 0;
  parameter P = 2;
  reg [15:0] a;
  reg [7:0] b;
  int x;
  initial begin
    b = 8'h3c;
    a = {{(16-W){1'b0}}, b};
    a = {a[15:8], {Z{1'b1}}, b};
    x = 4'(8'hff) + W'(9'h1ff);
    $display("R|%h %0d %0d", a, x, $signed(4'hf));
  end
endmodule
"#;
    let sim = simulate(src, 10).expect("legal design must run");
    assert!(
        sim.output.iter().any(|o| o.message == "R|003c 270 -1"),
        "{:?}",
        sim.output
    );
}
