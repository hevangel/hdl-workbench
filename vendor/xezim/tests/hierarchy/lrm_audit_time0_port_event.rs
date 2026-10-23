//! IEEE 1800-2023 §23.3.3 / §10.3: an input port is a continuous assignment
//! from its actual, so a declaration-initialized actual reaches the port as
//! a time-0 update event. A level-sensitive `always` in the instance that
//! reads the port runs once at time 0, while one on the variable itself in
//! the declaring scope does not (§6.8). Expected values come from the
//! reference simulator.
use xezim::simulate;

fn t_lines(sim: &xezim::compiler::Simulator) -> Vec<String> {
    sim.output
        .iter()
        .map(|o| o.message.trim_end().to_string())
        .filter(|l| l.starts_with("T|"))
        .collect()
}

/// The audit probe: ANSI and non-ANSI children of declaration-initialized
/// actuals, and a procedurally initialized one.
#[test]
fn decl_initialized_inputs_trigger_child_always_at_time0() {
    const SRC: &str = r#"
module nansi1(a, b, y);
  input [3:0] a;
  input b;
  output [3:0] y;
  reg [3:0] y;
  always @* y = a ^ {4{b}};
endmodule
module ansi1(input [3:0] a, input b, output logic [3:0] y);
  always @* y = a ^ {4{b}};
endmodule
module rna2;
  logic [3:0] a = 4'h3; logic b = 1;
  logic [3:0] a2; logic b2;
  wire [3:0] y1, y2, y3;
  nansi1 u1(a, b, y1);
  ansi1 u2(a, b, y2);
  nansi1 u3(a2, b2, y3);
  initial begin a2 = 4'h3; b2 = 1; end
  initial #1 $display("T|r1|init-decl: nonansi=%h ansi=%h  init-proc: nonansi=%h", y1, y2, y3);
endmodule
"#;
    let sim = simulate(SRC, 1000000).expect("sim");
    assert_eq!(
        t_lines(&sim),
        ["T|r1|init-decl: nonansi=c ansi=c  init-proc: nonansi=c",],
    );
}

/// A still-x actual makes no event; a port fed by a continuous assignment
/// does; the own `always @(v)` does not.
#[test]
fn time0_port_event_shapes() {
    const SRC: &str = r#"
module kid(input [3:0] a, input [3:0] u);
  logic [3:0] y, yu;
  always @* y = a ^ 4'h1;
  always @(u) yu = u;
endmodule
module kidw(input wire [3:0] a, output logic [3:0] y);
  always @(a) y = a + 1;
endmodule
module t0c;
  logic [3:0] a = 4'h3; logic [3:0] u;
  logic [3:0] v = 4'h5, vy;
  wire [3:0] wa; assign wa = a;
  logic [3:0] y2;
  kid k(.a(a), .u(u));
  kidw kw(.a(wa), .y(y2));
  always @(v) vy = v;
  initial #1 $display("T|r1|y=%h yu=%h y2=%h vy=%h", k.y, k.yu, y2, vy);
endmodule
"#;
    let sim = simulate(SRC, 1000000).expect("sim");
    assert_eq!(t_lines(&sim), ["T|r1|y=2 yu=x y2=4 vy=x",],);
}
