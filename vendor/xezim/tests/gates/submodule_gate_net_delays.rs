//! §28.9 / §10.3.1: gate delays and net-declaration delays inside an INLINED
//! sub-module instance.
//!
//! The per-instance inlining pushed each gate (and each `wire #d w = expr;`)
//! as a deferred continuous assign with no delay at all, so `and #3` inside a
//! child ran undelayed while the same gate in the top module waited 3 units.
//! A `#(rise, fall)` pair and the §28.4 `buf` z→x rule were lost the same
//! way. The delays now travel unevaluated like an `assign #d` delay does, so
//! a parameter they name resolves per instance, in the child's timeunit.
//! A UDP instance delay naming a parameter had the same gap. Expected lines
//! are the reference simulator's.

use xezim::simulate;

fn lines(src: &str, prefix: &str) -> Vec<String> {
    let sim = simulate(src, 1000).expect("simulate");
    sim.output
        .iter()
        .map(|o| o.message.clone())
        .filter(|m| m.starts_with(prefix))
        .collect()
}

#[test]
fn gate_and_net_delays_survive_inlining_per_instance() {
    let src = r#"
`timescale 1ns/1ns
module sub #(parameter D = 3, parameter W = 2)
            (input a, input b, output y, output z, output c, output m);
  and #D g1 (y, a, b);
  wire #W w = a & b;
  assign z = w;
  assign #(D+1) c = a | b;
  nand #(1,2) g2 (m, a, b);
endmodule

module top;
  reg a, b;
  wire yt, y1, z1, c1, m1, y2, z2, c2, m2;
  and #3 gt (yt, a, b);
  sub u1 (.a(a), .b(b), .y(y1), .z(z1), .c(c1), .m(m1));
  sub #(.D(5), .W(4)) u2 (.a(a), .b(b), .y(y2), .z(z2), .c(c2), .m(m2));
  initial begin
    $monitor("D|%0t yt=%b y1=%b z1=%b c1=%b m1=%b | y2=%b z2=%b c2=%b m2=%b",
             $time, yt, y1, z1, c1, m1, y2, z2, c2, m2);
    a = 0; b = 0;
    #10 a = 1; b = 1;
    #10 a = 0;
    #10 $finish;
  end
endmodule
"#;
    assert_eq!(
        lines(src, "D|"),
        [
            "D|0 yt=x y1=x z1=x c1=x m1=x | y2=x z2=x c2=x m2=x",
            "D|1 yt=x y1=x z1=x c1=x m1=1 | y2=x z2=x c2=x m2=1",
            "D|2 yt=x y1=x z1=0 c1=x m1=1 | y2=x z2=x c2=x m2=1",
            "D|3 yt=0 y1=0 z1=0 c1=x m1=1 | y2=x z2=x c2=x m2=1",
            "D|4 yt=0 y1=0 z1=0 c1=0 m1=1 | y2=x z2=0 c2=x m2=1",
            "D|5 yt=0 y1=0 z1=0 c1=0 m1=1 | y2=0 z2=0 c2=x m2=1",
            "D|6 yt=0 y1=0 z1=0 c1=0 m1=1 | y2=0 z2=0 c2=0 m2=1",
            "D|12 yt=0 y1=0 z1=1 c1=0 m1=0 | y2=0 z2=0 c2=0 m2=0",
            "D|13 yt=1 y1=1 z1=1 c1=0 m1=0 | y2=0 z2=0 c2=0 m2=0",
            "D|14 yt=1 y1=1 z1=1 c1=1 m1=0 | y2=0 z2=1 c2=0 m2=0",
            "D|15 yt=1 y1=1 z1=1 c1=1 m1=0 | y2=1 z2=1 c2=0 m2=0",
            "D|16 yt=1 y1=1 z1=1 c1=1 m1=0 | y2=1 z2=1 c2=1 m2=0",
            "D|21 yt=1 y1=1 z1=1 c1=1 m1=1 | y2=1 z2=1 c2=1 m2=1",
            "D|22 yt=1 y1=1 z1=0 c1=1 m1=1 | y2=1 z2=1 c2=1 m2=1",
            "D|23 yt=0 y1=0 z1=0 c1=1 m1=1 | y2=1 z2=1 c2=1 m2=1",
            "D|24 yt=0 y1=0 z1=0 c1=1 m1=1 | y2=1 z2=0 c2=1 m2=1",
            "D|25 yt=0 y1=0 z1=0 c1=1 m1=1 | y2=0 z2=0 c2=1 m2=1",
        ]
    );
}

/// The child's delays count ITS timeunit (10ns here), and a real-valued
/// parameter override rounds to the child's precision.
#[test]
fn inlined_delays_count_the_child_timeunit() {
    let src = r#"
`timescale 10ns/1ns
module sub #(parameter real D = 0.5) (input a, output y, output w_o, output c);
  buf #D g1 (y, a);
  wire #(0.3) w = a;
  assign w_o = w;
  assign #(D) c = a;
endmodule
`timescale 1ns/1ns
module top;
  reg a;
  wire y1, w1, c1, y2, w2, c2;
  sub u1 (.a(a), .y(y1), .w_o(w1), .c(c1));
  sub #(.D(1.2)) u2 (.a(a), .y(y2), .w_o(w2), .c(c2));
  initial begin
    $monitor("T|%0t y1=%b w1=%b c1=%b y2=%b w2=%b c2=%b", $time, y1, w1, c1, y2, w2, c2);
    a = 0;
    #50 a = 1;
    #50 $finish;
  end
endmodule
"#;
    assert_eq!(
        lines(src, "T|"),
        [
            "T|0 y1=x w1=x c1=x y2=x w2=x c2=x",
            "T|3 y1=x w1=0 c1=x y2=x w2=0 c2=x",
            "T|5 y1=0 w1=0 c1=0 y2=x w2=0 c2=x",
            "T|12 y1=0 w1=0 c1=0 y2=0 w2=0 c2=0",
            "T|53 y1=0 w1=1 c1=0 y2=0 w2=1 c2=0",
            "T|55 y1=1 w1=1 c1=1 y2=0 w2=1 c2=0",
            "T|62 y1=1 w1=1 c1=1 y2=1 w2=1 c2=1",
        ]
    );
}

/// §28.4: a `buf` passes z through as x. The top-level lowering marked its
/// output gate-driven; the inlined one did not, so the child drove z.
#[test]
fn inlined_buf_turns_z_into_x() {
    let src = r#"
module sub (input a, output y);
  buf g1 (y, a);
endmodule
module top;
  reg a;
  wire yt, y1;
  buf gt (yt, a);
  sub u1 (.a(a), .y(y1));
  initial begin
    a = 1'bz;
    #1 $display("Z|yt=%b y1=%b", yt, y1);
    a = 1;
    #1 $display("Z|yt=%b y1=%b", yt, y1);
    $finish;
  end
endmodule
"#;
    assert_eq!(lines(src, "Z|"), ["Z|yt=x y1=x", "Z|yt=1 y1=1"]);
}

/// §29.8: a UDP instance delay naming a parameter (`#(D)`) is evaluated in
/// the enclosing instance's scope. Inlined, `D` missed the flattened `u.D`
/// and the instance ran undelayed; a literal delay was already right.
#[test]
fn inlined_udp_delay_parameter_resolves_per_instance() {
    let src = r#"
`timescale 1ns/1ns
primitive inv_p (o, i);
  output o; input i;
  table 0 : 1; 1 : 0; endtable
endprimitive
module sub #(parameter D = 2) (input a, output y1, output y2);
  inv_p #3 u1 (y1, a);
  inv_p #(D) u2 (y2, a);
endmodule
module top;
  reg a;
  wire t1, s1, s2, p1, p2;
  inv_p #3 ut (t1, a);
  sub u (.a(a), .y1(s1), .y2(s2));
  sub #(.D(5)) v (.a(a), .y1(p1), .y2(p2));
  initial begin
    $monitor("U|%0t t1=%b s1=%b s2=%b p1=%b p2=%b", $time, t1, s1, s2, p1, p2);
    a = 0;
    #10 a = 1;
    #10 $finish;
  end
endmodule
"#;
    assert_eq!(
        lines(src, "U|"),
        [
            "U|0 t1=x s1=x s2=x p1=x p2=x",
            "U|2 t1=x s1=x s2=1 p1=x p2=x",
            "U|3 t1=1 s1=1 s2=1 p1=1 p2=x",
            "U|5 t1=1 s1=1 s2=1 p1=1 p2=1",
            "U|12 t1=1 s1=1 s2=0 p1=1 p2=1",
            "U|13 t1=0 s1=0 s2=0 p1=0 p2=1",
            "U|15 t1=0 s1=0 s2=0 p1=0 p2=0",
        ]
    );
}
