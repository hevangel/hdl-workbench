//! §23.3.3: a port connection is a continuous assignment. An input port
//! whose actual COMPUTES something (`.p(a & b)`, `.raddr(addr % 128)`) is a
//! net of its own, driven once per change of the expression; only a rename
//! (a net, a constant select of one, a literal, a concatenation of those) is
//! substituted into the child.
//!
//! Substituting the expression instead made every read of the port
//! re-evaluate it — a decoder term fed to 128 rows ran 128 times per address
//! change — and made `@(p)` wake on any operand change rather than on a
//! change of `p`. Expected values are the reference simulator's.

use xezim::simulate;

fn u(sim: &xezim::compiler::Simulator, n: &str) -> u64 {
    sim.get_signal(n)
        .or_else(|| sim.get_signal(&format!("top.{}", n)))
        .unwrap_or_else(|| panic!("signal not found: {}", n))
        .to_u64()
        .unwrap_or_else(|| panic!("{} not u64-able (x/z?)", n))
}

/// §9.4.2: `@(p)` waits for a change of `p` itself. `a` rising while `b`
/// is 0 leaves `a & b` at 0, and `v` 3 -> 7 leaves `v % 4` at 3: neither
/// wakes the child. (Events at time 0 are left out: the port net's first
/// update races the processes' first wait.)
#[test]
fn event_control_on_an_expression_port() {
    let src = r#"
module kid(input p, input [3:0] q, input real r);
  integer np = 0, nq = 0; real rr = 0.0;
  always @(p) if ($time > 0) np = np + 1;
  always @(q) if ($time > 0) nq = nq + 1;
  initial #20 rr = r;
endmodule
module top;
  reg a = 0, b = 0; reg [3:0] v = 4'h3; real x = 1.5;
  kid u_k(.p(a & b), .q(v % 4'd4), .r(x * 2.0));
  integer rx;
  initial begin
    #1 a = 1;
    #1 b = 1;
    #1 a = 0;
    #1 v = 4'h7;
    #1 v = 4'h6;
    #20 rx = $rtoi(u_k.rr * 10.0);
  end
endmodule
"#;
    let sim = simulate(src, 1000).expect("simulate");
    assert_eq!(u(&sim, "u_k.np"), 2, "p changes at t=2 and t=3 only");
    assert_eq!(u(&sim, "u_k.nq"), 1, "q changes at t=5 only");
    assert_eq!(u(&sim, "rx"), 30, "a real expression actual");
}

/// A two-level address decoder whose select and row-address ports are
/// expressions, including a clock port fed `~clk`.
#[test]
fn decoder_through_expression_ports() {
    let src = r#"
module flop(input clk, input en, input [3:0] d, output reg [3:0] q);
  always @(posedge clk) if (en) q <= d;
endmodule
module dec(input sel, input [1:0] ra, input clk, input [3:0] d,
           output [3:0] q0, output [3:0] q1, output [3:0] q2, output [3:0] q3);
  flop f0(.clk(clk), .en(sel & (ra == 0)), .d(d), .q(q0));
  flop f1(.clk(clk), .en(sel & (ra == 1)), .d(d + 4'd1), .q(q1));
  flop f2(.clk(~clk), .en(sel & (ra == 2)), .d(d ^ 4'hf), .q(q2));
  flop f3(.clk(clk), .en(sel & (ra == 3)), .d({d[1:0], d[3:2]}), .q(q3));
endmodule
module top;
  reg clk = 0; reg [3:0] addr = 0; reg [3:0] d = 0;
  always #5 clk = ~clk;
  wire [3:0] a0, a1, a2, a3, b0, b1, b2, b3;
  dec u0(.sel(addr / 4 == 0), .ra(addr[1:0]), .clk(clk), .d(d),
         .q0(a0), .q1(a1), .q2(a2), .q3(a3));
  dec u1(.sel(addr / 4 == 1), .ra(addr % 4), .clk(clk), .d(d),
         .q0(b0), .q1(b1), .q2(b2), .q3(b3));
  integer i;
  initial begin
    for (i = 0; i < 8; i = i + 1) begin
      @(negedge clk); addr = i; d = i * 3 + 1;
      @(posedge clk); @(negedge clk);
    end
    #1 $finish;
  end
endmodule
"#;
    let sim = simulate(src, 10_000).expect("simulate");
    let got: Vec<u64> = ["a0", "a1", "a2", "a3", "b0", "b1", "b2", "b3"]
        .iter()
        .map(|n| u(&sim, n))
        .collect();
    assert_eq!(got, vec![0x1, 0x5, 0x8, 0xa, 0xd, 0x1, 0xc, 0x9]);
}
