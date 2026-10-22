//! §6.7.1 / §10.3.3 / §28.16: a delay on a net declaration is a NET delay:
//! it applies to every driver of the net (a separate `assign`, a port, a
//! gate, a UDP), after the drivers' own delays and after resolution, bit by
//! bit on a vector. The delay of a declaration WITH an assignment belongs to
//! that assignment instead.
//!
//! `wire #2 w; assign w = a;` used to track `a` undelayed, at the top level
//! and in a sub-module alike. Expected values are the reference simulator's.

use xezim::simulate;

fn out(src: &str) -> Vec<String> {
    let sim = simulate(src, 1000).expect("simulate failed");
    sim.output
        .iter()
        .map(|o| o.message.clone())
        .filter(|m| m.starts_with("T|"))
        .collect()
}

#[test]
fn net_delay_with_a_declaration_assignment() {
    let o = out(r#"
module top;
  logic a = 0;
  wire #2 wi = a;
  initial begin
    #10 a = 1;
    #1 $display("T|t=%0t wi=%b", $time, wi);
    #1 $display("T|t=%0t wi=%b", $time, wi);
    $finish;
  end
endmodule
"#);
    assert_eq!(o, ["T|t=11 wi=0", "T|t=12 wi=1"], "{o:?}");
}

#[test]
fn net_delay_applies_to_a_separate_continuous_assign() {
    let o = out(r#"
module sub (input logic a, output wire y);
  wire #3 w;
  assign w = a;
  assign y = w;
endmodule
module top;
  logic a = 0;
  wire #2 w;
  assign w = a;
  wire ys;
  sub s (.a(a), .y(ys));
  initial begin
    #10 a = 1;
    #1 $display("T|t=%0t w=%b ys=%b", $time, w, ys);
    #1 $display("T|t=%0t w=%b ys=%b", $time, w, ys);
    #1 $display("T|t=%0t w=%b ys=%b", $time, w, ys);
    $finish;
  end
endmodule
"#);
    assert_eq!(
        o,
        ["T|t=11 w=0 ys=0", "T|t=12 w=1 ys=0", "T|t=13 w=1 ys=1"],
        "{o:?}"
    );
}

/// §10.3.3: `wire #2 w1; assign #3 w1 = a;` delays `a` twice, as two inertial
/// stages — a 4-unit pulse survives `#3` and then `#2` (one `#5` would swallow
/// it), while `w4`'s 2-unit pulse is shorter than its `#3` net delay and is
/// swallowed. A declaration assignment's delay (`wor #2 w3 = a;`) belongs to
/// that assignment, not to the net: the second driver `b` of `w3` is not
/// delayed.
#[test]
fn net_delay_is_its_own_inertial_stage() {
    let o = out(r#"
// net delay + assign delay; pulse filtering; decl-assign plus second driver
module top;
  logic a = 0, b = 0;
  wire #2 w1; assign #3 w1 = a;     // total 5; a pulse of 4 passes assign#3, then net#2
  wire #2 w2 = a;                    // decl assign
  wor  #2 w3 = a; assign w3 = b;     // decl-assign + second driver
  wire #3 w4; assign w4 = a;         // pulse shorter than 3 swallowed
  initial begin
    $monitor("T|t=%0t a=%b b=%b w1=%b w2=%b w3=%b w4=%b", $time, a, b, w1, w2, w3, w4);
    #10 a = 1;
    #2 a = 0;          // pulse width 2 on a (t=10..12)
    #10 a = 1;
    #4 a = 0;          // pulse width 4 (t=22..26)
    #10 b = 1;         // t=36
    #1 b = 0;          // t=37
    #20 $finish;
  end
endmodule
"#);
    assert_eq!(
        o,
        [
            "T|t=0 a=0 b=0 w1=x w2=x w3=x w4=x",
            "T|t=2 a=0 b=0 w1=x w2=0 w3=0 w4=x",
            "T|t=3 a=0 b=0 w1=x w2=0 w3=0 w4=0",
            "T|t=5 a=0 b=0 w1=0 w2=0 w3=0 w4=0",
            "T|t=10 a=1 b=0 w1=0 w2=0 w3=0 w4=0",
            "T|t=12 a=0 b=0 w1=0 w2=1 w3=1 w4=0",
            "T|t=14 a=0 b=0 w1=0 w2=0 w3=0 w4=0",
            "T|t=22 a=1 b=0 w1=0 w2=0 w3=0 w4=0",
            "T|t=24 a=1 b=0 w1=0 w2=1 w3=1 w4=0",
            "T|t=25 a=1 b=0 w1=0 w2=1 w3=1 w4=1",
            "T|t=26 a=0 b=0 w1=0 w2=1 w3=1 w4=1",
            "T|t=27 a=0 b=0 w1=1 w2=1 w3=1 w4=1",
            "T|t=28 a=0 b=0 w1=1 w2=0 w3=0 w4=1",
            "T|t=29 a=0 b=0 w1=1 w2=0 w3=0 w4=0",
            "T|t=31 a=0 b=0 w1=0 w2=0 w3=0 w4=0",
            "T|t=36 a=0 b=1 w1=0 w2=0 w3=1 w4=0",
            "T|t=37 a=0 b=0 w1=0 w2=0 w3=0 w4=0",
        ],
        "{o:?}"
    );
}

/// A delayed parent net driven by a sub-module output port is delayed, and
/// so is everything reading it (`y`). A port's own net delay (`output o; wire
/// #4 o;`) is dropped once the port collapses onto a parent net (§23.3.3.7):
/// `z3` and `u3.o` follow `a` at once.
#[test]
fn net_delay_on_a_net_driven_through_a_port() {
    let o = out(r#"
// sub-module output port drives a delayed parent net; delayed net used as input
module sub (input logic a, output o);
  assign o = a;
endmodule
module sub2 (input i, output o);
  assign o = i;
endmodule
module sub3 (a, o);
  input a;
  output o;
  wire #4 o;
  assign o = a;
endmodule
module top;
  logic a = 0;
  wire #2 w;
  sub u (.a(a), .o(w));
  wire y;
  sub2 u2 (.i(w), .o(y));
  wire z3;
  sub3 u3 (.a(a), .o(z3));
  initial begin
    $monitor("T|t=%0t a=%b w=%b y=%b z3=%b u3.o=%b", $time, a, w, y, z3, u3.o);
    #10 a = 1;
    #10 $finish;
  end
endmodule
"#);
    assert_eq!(
        o,
        [
            "T|t=0 a=0 w=x y=x z3=0 u3.o=0",
            "T|t=2 a=0 w=0 y=0 z3=0 u3.o=0",
            "T|t=10 a=1 w=0 y=0 z3=1 u3.o=1",
            "T|t=12 a=1 w=1 y=1 z3=1 u3.o=1",
        ],
        "{o:?}"
    );
}

/// Several drivers of a delayed `wand`, `wire` and `wor` resolve first and
/// the net delay then applies to the resolved value: at t=71/72 the drivers
/// swap, the resolved `ww` is x throughout and turns x at 73, one delay after
/// it first changed.
#[test]
fn net_delay_acts_on_the_resolved_value() {
    let o = out(r#"
// multiple drivers: wand and wire with two assigns
module top;
  logic a = 1, b = 1, c = 0, d = 0;
  wand #2 wa; assign wa = a; assign wa = b;
  wire #2 ww; assign ww = c; assign ww = d;
  wor  #2 wo; assign wo = c; assign wo = d;
  initial begin
    $monitor("T|t=%0t a=%b b=%b c=%b d=%b wa=%b ww=%b wo=%b", $time, a, b, c, d, wa, ww, wo);
    #10 a = 0;
    #10 a = 1;
    #10 c = 1;
    #10 d = 1;
    #10 c = 0; d = 0;
    #10 c = 1'bz; d = 1'b1;
    #10 c = 1'b1; #1 d = 1'b0; #1 c = 1'b0; d = 1'b1; // resolved pulses
    #10 $finish;
  end
endmodule
"#);
    assert_eq!(
        o,
        [
            "T|t=0 a=1 b=1 c=0 d=0 wa=x ww=x wo=x",
            "T|t=2 a=1 b=1 c=0 d=0 wa=1 ww=0 wo=0",
            "T|t=10 a=0 b=1 c=0 d=0 wa=1 ww=0 wo=0",
            "T|t=12 a=0 b=1 c=0 d=0 wa=0 ww=0 wo=0",
            "T|t=20 a=1 b=1 c=0 d=0 wa=0 ww=0 wo=0",
            "T|t=22 a=1 b=1 c=0 d=0 wa=1 ww=0 wo=0",
            "T|t=30 a=1 b=1 c=1 d=0 wa=1 ww=0 wo=0",
            "T|t=32 a=1 b=1 c=1 d=0 wa=1 ww=x wo=1",
            "T|t=40 a=1 b=1 c=1 d=1 wa=1 ww=x wo=1",
            "T|t=42 a=1 b=1 c=1 d=1 wa=1 ww=1 wo=1",
            "T|t=50 a=1 b=1 c=0 d=0 wa=1 ww=1 wo=1",
            "T|t=52 a=1 b=1 c=0 d=0 wa=1 ww=0 wo=0",
            "T|t=60 a=1 b=1 c=z d=1 wa=1 ww=0 wo=0",
            "T|t=62 a=1 b=1 c=z d=1 wa=1 ww=1 wo=1",
            "T|t=70 a=1 b=1 c=1 d=1 wa=1 ww=1 wo=1",
            "T|t=71 a=1 b=1 c=1 d=0 wa=1 ww=1 wo=1",
            "T|t=72 a=1 b=1 c=0 d=1 wa=1 ww=1 wo=1",
            "T|t=73 a=1 b=1 c=0 d=1 wa=1 ww=x wo=1",
        ],
        "{o:?}"
    );
}

/// Vector nets (one driver, or one per part), `#(rise, fall)` on a scalar and
/// on a vector — whose bits each pick their own delay — and delays given by a
/// parameter, at the top and per instance.
#[test]
fn vector_rise_fall_and_parameterized_net_delays() {
    let o = out(r#"
// vector net, part drivers, rise/fall, parameterized
module sub #(parameter D = 3) (input logic a, output wire y);
  wire #(D) w;
  assign w = a;
  assign y = w;
endmodule
module top;
  parameter P = 4;
  logic [3:0] v = 0;
  logic a = 0;
  wire [3:0] #2 wv; assign wv = v;
  wire [3:0] #2 wp; assign wp[1:0] = v[1:0]; assign wp[3:2] = v[3:2];
  wire #(1,3) wrf; assign wrf = a;
  wire [1:0] #(1,3) wrfv; assign wrfv = {a, 1'b1};
  wire #(P) wpar; assign wpar = a;
  wire ys5, ys7;
  sub #(5) s5 (.a(a), .y(ys5));
  sub #(.D(7)) s7 (.a(a), .y(ys7));
  initial begin
    $monitor("T|t=%0t v=%h wv=%h wp=%h a=%b wrf=%b wrfv=%b wpar=%b ys5=%b ys7=%b", $time, v, wv, wp, a, wrf, wrfv, wpar, ys5, ys7);
    #10 v = 4'h5; a = 1;
    #10 v = 4'ha; a = 0;
    #10 $finish;
  end
endmodule
"#);
    assert_eq!(
        o,
        [
            "T|t=0 v=0 wv=x wp=x a=0 wrf=x wrfv=xx wpar=x ys5=x ys7=x",
            "T|t=1 v=0 wv=x wp=x a=0 wrf=x wrfv=x1 wpar=x ys5=x ys7=x",
            "T|t=2 v=0 wv=0 wp=0 a=0 wrf=x wrfv=x1 wpar=x ys5=x ys7=x",
            "T|t=3 v=0 wv=0 wp=0 a=0 wrf=0 wrfv=01 wpar=x ys5=x ys7=x",
            "T|t=4 v=0 wv=0 wp=0 a=0 wrf=0 wrfv=01 wpar=0 ys5=x ys7=x",
            "T|t=5 v=0 wv=0 wp=0 a=0 wrf=0 wrfv=01 wpar=0 ys5=0 ys7=x",
            "T|t=7 v=0 wv=0 wp=0 a=0 wrf=0 wrfv=01 wpar=0 ys5=0 ys7=0",
            "T|t=10 v=5 wv=0 wp=0 a=1 wrf=0 wrfv=01 wpar=0 ys5=0 ys7=0",
            "T|t=11 v=5 wv=0 wp=0 a=1 wrf=1 wrfv=11 wpar=0 ys5=0 ys7=0",
            "T|t=12 v=5 wv=5 wp=5 a=1 wrf=1 wrfv=11 wpar=0 ys5=0 ys7=0",
            "T|t=14 v=5 wv=5 wp=5 a=1 wrf=1 wrfv=11 wpar=1 ys5=0 ys7=0",
            "T|t=15 v=5 wv=5 wp=5 a=1 wrf=1 wrfv=11 wpar=1 ys5=1 ys7=0",
            "T|t=17 v=5 wv=5 wp=5 a=1 wrf=1 wrfv=11 wpar=1 ys5=1 ys7=1",
            "T|t=20 v=a wv=5 wp=5 a=0 wrf=1 wrfv=11 wpar=1 ys5=1 ys7=1",
            "T|t=22 v=a wv=a wp=a a=0 wrf=1 wrfv=11 wpar=1 ys5=1 ys7=1",
            "T|t=23 v=a wv=a wp=a a=0 wrf=0 wrfv=01 wpar=1 ys5=1 ys7=1",
            "T|t=24 v=a wv=a wp=a a=0 wrf=0 wrfv=01 wpar=0 ys5=1 ys7=1",
            "T|t=25 v=a wv=a wp=a a=0 wrf=0 wrfv=01 wpar=0 ys5=0 ys7=1",
            "T|t=27 v=a wv=a wp=a a=0 wrf=0 wrfv=01 wpar=0 ys5=0 ys7=0",
        ],
        "{o:?}"
    );
}

/// `trireg` holds its charge, `tri` floats, `tri0` pulls to 0 — each through
/// the net delay — and `#(1,2,5)` uses the turn-off delay for a change to z.
#[test]
fn net_delays_on_tri_trireg_tri0_and_turn_off() {
    let o = out(r#"
// trireg / tri / tri0 with delay; rise/fall/off to z
module top;
  logic a = 0, en = 1;
  trireg #2 tr; assign tr = en ? a : 1'bz;
  tri #2 t; assign t = en ? a : 1'bz;
  tri0 #2 t0; assign t0 = en ? a : 1'bz;
  wire #(1,2,5) woff; assign woff = en ? a : 1'bz;
  initial begin
    $monitor("T|t=%0t a=%b en=%b tr=%b t=%b t0=%b woff=%b", $time, a, en, tr, t, t0, woff);
    #10 a = 1;
    #10 en = 0;
    #10 a = 0;
    #10 en = 1;
    #10 $finish;
  end
endmodule
"#);
    assert_eq!(
        o,
        [
            "T|t=0 a=0 en=1 tr=x t=x t0=x woff=x",
            "T|t=2 a=0 en=1 tr=0 t=0 t0=0 woff=0",
            "T|t=10 a=1 en=1 tr=0 t=0 t0=0 woff=0",
            "T|t=11 a=1 en=1 tr=0 t=0 t0=0 woff=1",
            "T|t=12 a=1 en=1 tr=1 t=1 t0=1 woff=1",
            "T|t=20 a=1 en=0 tr=1 t=1 t0=1 woff=1",
            "T|t=22 a=1 en=0 tr=1 t=z t0=0 woff=1",
            "T|t=25 a=1 en=0 tr=1 t=z t0=0 woff=z",
            "T|t=30 a=0 en=0 tr=1 t=z t0=0 woff=z",
            "T|t=40 a=0 en=1 tr=1 t=z t0=0 woff=z",
            "T|t=42 a=0 en=1 tr=0 t=0 t0=0 woff=0",
        ],
        "{o:?}"
    );
}

/// §10.3.3: `wire #2 x = a, y = b;` delays each declaration assignment, not
/// the nets: `assign y = a;` drives `y` undelayed beside the delayed `b`.
#[test]
fn declaration_assignment_delays_stay_per_driver() {
    let o = out(r#"
module top;
  logic a = 0, b = 0;
  wire #2 x = a, y = b;
  assign y = a;
  initial begin
    $monitor("T|t=%0t a=%b b=%b x=%b y=%b", $time, a, b, x, y);
    #10 a = 1;
    #10 b = 1;
    #10 $finish;
  end
endmodule
"#);
    assert_eq!(
        o,
        [
            "T|t=0 a=0 b=0 x=x y=x",
            "T|t=2 a=0 b=0 x=0 y=0",
            "T|t=10 a=1 b=0 x=0 y=x",
            "T|t=12 a=1 b=0 x=1 y=x",
            "T|t=20 a=1 b=1 x=1 y=x",
            "T|t=22 a=1 b=1 x=1 y=1",
        ],
        "{o:?}"
    );
}

/// A delayed driver that re-evaluates to the value already in flight keeps
/// that update's time (`c ^ d` stays 0 from t=10 to 12). Two drivers with
/// their own delays (`#3` and `#1`) each delay their own value before the net
/// resolves them. An assign's `#(1,3)` on a vector picks one delay for the
/// whole value. A driven `tri0` reads x until its driver's value arrives.
#[test]
fn delayed_drivers_keep_their_schedules() {
    let o = out(r#"
module top;
  logic c = 1, d = 0, a = 0, b = 0;
  wire #2 wn;  assign wn = c ^ d;          // net delay; value stays 1 while operands swap
  wire wa;     assign #2 wa = c ^ d;       // assign delay, same
  wire w2;     assign #3 w2 = a; assign #1 w2 = b;   // two drivers, different delays
  wire [1:0] v2; assign #(1,3) v2 = {a, 1'b1};       // vector rise/fall on an assign
  tri0 t0; assign #2 t0 = a;               // tri0 driven through an assign delay
  initial begin
    $monitor("T|t=%0t c=%b d=%b a=%b b=%b wn=%b wa=%b w2=%b v2=%b t0=%b", $time, c, d, a, b, wn, wa, w2, v2, t0);
    #10 c = 0; d = 0;    // c^d: 1 -> 0
    #1  c = 1; d = 1;    // still 0
    #1  c = 1; d = 0;    // -> 1
    #10 a = 1; b = 1;
    #10 a = 0;
    #10 b = 0;
    #10 $finish;
  end
endmodule
"#);
    assert_eq!(
        o,
        [
            "T|t=0 c=1 d=0 a=0 b=0 wn=x wa=x w2=x v2=xx t0=x",
            "T|t=1 c=1 d=0 a=0 b=0 wn=x wa=x w2=x v2=01 t0=x",
            "T|t=2 c=1 d=0 a=0 b=0 wn=1 wa=1 w2=x v2=01 t0=0",
            "T|t=3 c=1 d=0 a=0 b=0 wn=1 wa=1 w2=0 v2=01 t0=0",
            "T|t=10 c=0 d=0 a=0 b=0 wn=1 wa=1 w2=0 v2=01 t0=0",
            "T|t=11 c=1 d=1 a=0 b=0 wn=1 wa=1 w2=0 v2=01 t0=0",
            "T|t=12 c=1 d=0 a=0 b=0 wn=0 wa=0 w2=0 v2=01 t0=0",
            "T|t=14 c=1 d=0 a=0 b=0 wn=1 wa=1 w2=0 v2=01 t0=0",
            "T|t=22 c=1 d=0 a=1 b=1 wn=1 wa=1 w2=0 v2=01 t0=0",
            "T|t=23 c=1 d=0 a=1 b=1 wn=1 wa=1 w2=x v2=11 t0=0",
            "T|t=24 c=1 d=0 a=1 b=1 wn=1 wa=1 w2=x v2=11 t0=1",
            "T|t=25 c=1 d=0 a=1 b=1 wn=1 wa=1 w2=1 v2=11 t0=1",
            "T|t=32 c=1 d=0 a=0 b=1 wn=1 wa=1 w2=1 v2=11 t0=1",
            "T|t=33 c=1 d=0 a=0 b=1 wn=1 wa=1 w2=1 v2=01 t0=1",
            "T|t=34 c=1 d=0 a=0 b=1 wn=1 wa=1 w2=1 v2=01 t0=0",
            "T|t=35 c=1 d=0 a=0 b=1 wn=1 wa=1 w2=x v2=01 t0=0",
            "T|t=42 c=1 d=0 a=0 b=0 wn=1 wa=1 w2=x v2=01 t0=0",
            "T|t=43 c=1 d=0 a=0 b=0 wn=1 wa=1 w2=0 v2=01 t0=0",
        ],
        "{o:?}"
    );
}

/// A port's net delay applies while the port stays a net of its own: an
/// input fed from a variable (`u4.a`), an open output (`u3n.o`). An output
/// collapsed onto a parent net (`u3`, `u5`) takes the parent net's (lack of)
/// delay instead.
#[test]
fn port_net_delays_and_port_collapsing() {
    let o = out(r#"
module sub3 (a, o);
  input a;
  output o;
  wire #4 o;
  assign o = a;
endmodule
module sub4 (a, y);
  input a;
  wire #3 a;
  output y;
  assign y = a;
endmodule
module sub5 (a, y);
  input a;
  output y;
  wire #4 y;
  wire #2 m;
  assign m = a;
  assign y = m;
endmodule
module top;
  logic a = 0;
  wire z3, y4, y5;
  sub3 u3 (.a(a), .o(z3));
  sub3 u3n (.a(a), .o());
  sub4 u4 (.a(a), .y(y4));
  sub5 u5 (.a(a), .y(y5));
  initial begin
    $monitor("T|t=%0t a=%b z3=%b u3.o=%b u3n.o=%b y4=%b u4.a=%b y5=%b u5.m=%b", $time, a, z3, u3.o, u3n.o, y4, u4.a, y5, u5.m);
    #10 a = 1;
    #10 $finish;
  end
endmodule
"#);
    assert_eq!(
        o,
        [
            "T|t=0 a=0 z3=0 u3.o=0 u3n.o=x y4=x u4.a=x y5=x u5.m=x",
            "T|t=2 a=0 z3=0 u3.o=0 u3n.o=x y4=x u4.a=x y5=0 u5.m=0",
            "T|t=3 a=0 z3=0 u3.o=0 u3n.o=x y4=0 u4.a=0 y5=0 u5.m=0",
            "T|t=4 a=0 z3=0 u3.o=0 u3n.o=0 y4=0 u4.a=0 y5=0 u5.m=0",
            "T|t=10 a=1 z3=1 u3.o=1 u3n.o=0 y4=0 u4.a=0 y5=0 u5.m=0",
            "T|t=12 a=1 z3=1 u3.o=1 u3n.o=0 y4=0 u4.a=0 y5=1 u5.m=1",
            "T|t=13 a=1 z3=1 u3.o=1 u3n.o=0 y4=1 u4.a=1 y5=1 u5.m=1",
            "T|t=14 a=1 z3=1 u3.o=1 u3n.o=1 y4=1 u4.a=1 y5=1 u5.m=1",
        ],
        "{o:?}"
    );
}

/// §28.16: a vector net's delay schedules each bit on its own — bit 0 of
/// `wn2` changes at 12 and bit 1 at 13 — where an assign delay reschedules
/// the whole value (`wa2` changes once, at 13). A 1-unit pulse on one bit is
/// still swallowed.
#[test]
fn vector_net_delay_is_per_bit() {
    let o = out(r#"
module top;
  logic [1:0] v = 0;
  wire [1:0] #2 wn2; assign wn2 = v;
  wire [1:0] wa2; assign #2 wa2 = v;
  wire [3:0] #3 wn4; assign wn4 = {v, v};
  initial begin
    $monitor("T|t=%0t v=%b wn2=%b wa2=%b wn4=%b", $time, v, wn2, wa2, wn4);
    #10 v = 2'b01;
    #1  v = 2'b11;
    #10 v = 2'b10;
    #1  v = 2'b00;
    #10 v = 2'b01;
    #1  v = 2'b00;   // bit0 pulse of 1 < 2
    #10 $finish;
  end
endmodule
"#);
    assert_eq!(
        o,
        [
            "T|t=0 v=00 wn2=xx wa2=xx wn4=xxxx",
            "T|t=2 v=00 wn2=00 wa2=00 wn4=xxxx",
            "T|t=3 v=00 wn2=00 wa2=00 wn4=0000",
            "T|t=10 v=01 wn2=00 wa2=00 wn4=0000",
            "T|t=11 v=11 wn2=00 wa2=00 wn4=0000",
            "T|t=12 v=11 wn2=01 wa2=00 wn4=0000",
            "T|t=13 v=11 wn2=11 wa2=11 wn4=0101",
            "T|t=14 v=11 wn2=11 wa2=11 wn4=1111",
            "T|t=21 v=10 wn2=11 wa2=11 wn4=1111",
            "T|t=22 v=00 wn2=11 wa2=11 wn4=1111",
            "T|t=23 v=00 wn2=10 wa2=11 wn4=1111",
            "T|t=24 v=00 wn2=00 wa2=00 wn4=1010",
            "T|t=25 v=00 wn2=00 wa2=00 wn4=0000",
            "T|t=32 v=01 wn2=00 wa2=00 wn4=0000",
            "T|t=33 v=00 wn2=00 wa2=00 wn4=0000",
        ],
        "{o:?}"
    );
}

/// An input port collapsed onto a parent net drops its own `#3` (`e1`), or
/// takes the parent's `#1` (`e3`); an output collapsed onto a delayed parent
/// net reads that net's delayed value from inside (`e2.o`); one collapsed onto
/// a bit of a parent vector drops its `#4` too (`e4`).
#[test]
fn collapsed_ports_take_the_external_net_delay() {
    let o = out(r#"
module so (a, o);
  input a;
  output o;
  wire #4 o;
  assign o = a;
endmodule
module si (a, y);
  input a;
  wire #3 a;
  output y;
  assign y = a;
endmodule
module top;
  logic a = 0;
  wire aw; assign aw = a;
  wire #1 aw1; assign aw1 = a;
  wire #2 w2;
  wire [1:0] pw;
  wire y1, y3;
  si e1 (.a(aw), .y(y1));
  so e2 (.a(a), .o(w2));
  si e3 (.a(aw1), .y(y3));
  so e4 (.a(a), .o(pw[0]));
  initial begin
    $monitor("T|t=%0t a=%b y1=%b e1.a=%b w2=%b e2.o=%b y3=%b e3.a=%b pw=%b e4.o=%b", $time, a, y1, e1.a, w2, e2.o, y3, e3.a, pw, e4.o);
    #10 a = 1;
    #10 $finish;
  end
endmodule
"#);
    assert_eq!(
        o,
        [
            "T|t=0 a=0 y1=0 e1.a=0 w2=x e2.o=x y3=x e3.a=x pw=z0 e4.o=0",
            "T|t=1 a=0 y1=0 e1.a=0 w2=x e2.o=x y3=0 e3.a=0 pw=z0 e4.o=0",
            "T|t=2 a=0 y1=0 e1.a=0 w2=0 e2.o=0 y3=0 e3.a=0 pw=z0 e4.o=0",
            "T|t=10 a=1 y1=1 e1.a=1 w2=0 e2.o=0 y3=0 e3.a=0 pw=z1 e4.o=1",
            "T|t=11 a=1 y1=1 e1.a=1 w2=0 e2.o=0 y3=1 e3.a=1 pw=z1 e4.o=1",
            "T|t=12 a=1 y1=1 e1.a=1 w2=1 e2.o=1 y3=1 e3.a=1 pw=z1 e4.o=1",
        ],
        "{o:?}"
    );
}

/// The net delay adds a stage behind a gate (`buf`, and `buf #1`), a UDP, a
/// concatenation target, an instance's delayed assign, a vector net in an
/// instance, and nets declared in generate blocks.
#[test]
fn net_delay_with_gates_udps_concatenations_and_generates() {
    let o = out(r#"
primitive inv_p (o, i);
  output o; input i;
  table 0 : 1; 1 : 0; endtable
endprimitive
module sub (input logic a, output wire y, output wire [1:0] yv);
  wire #3 w;
  assign #1 w = a;           // own delay + net delay inside an instance
  wire [1:0] #2 wv;
  assign wv = {a, ~a};
  assign y = w;
  assign yv = wv;
endmodule
module top;
  logic a = 0, b = 0;
  wire #2 g1; buf bg1 (g1, a);          // gate driving a delayed net
  wire #2 g2; buf #1 bg2 (g2, a);       // gate with its own delay
  wire #2 u1; inv_p ui (u1, a);         // UDP driving a delayed net
  wire #2 c1, c2; assign {c1, c2} = {a, b};   // concatenation target
  wire ys; wire [1:0] yv;
  sub s (.a(a), .y(ys), .yv(yv));
  genvar i;
  for (i = 0; i < 2; i++) begin : gb
    wire #(i+1) gw;
    assign gw = a;
  end
  initial begin
    $monitor("T|t=%0t a=%b b=%b g1=%b g2=%b u1=%b c1=%b c2=%b ys=%b yv=%b gb0=%b gb1=%b", $time, a, b, g1, g2, u1, c1, c2, ys, yv, gb[0].gw, gb[1].gw);
    #10 a = 1;
    #10 b = 1;
    #10 $finish;
  end
endmodule
"#);
    assert_eq!(
        o,
        [
            "T|t=0 a=0 b=0 g1=x g2=x u1=x c1=x c2=x ys=x yv=xx gb0=x gb1=x",
            "T|t=1 a=0 b=0 g1=x g2=x u1=x c1=x c2=x ys=x yv=xx gb0=0 gb1=x",
            "T|t=2 a=0 b=0 g1=0 g2=x u1=1 c1=0 c2=0 ys=x yv=01 gb0=0 gb1=0",
            "T|t=3 a=0 b=0 g1=0 g2=0 u1=1 c1=0 c2=0 ys=x yv=01 gb0=0 gb1=0",
            "T|t=4 a=0 b=0 g1=0 g2=0 u1=1 c1=0 c2=0 ys=0 yv=01 gb0=0 gb1=0",
            "T|t=10 a=1 b=0 g1=0 g2=0 u1=1 c1=0 c2=0 ys=0 yv=01 gb0=0 gb1=0",
            "T|t=11 a=1 b=0 g1=0 g2=0 u1=1 c1=0 c2=0 ys=0 yv=01 gb0=1 gb1=0",
            "T|t=12 a=1 b=0 g1=1 g2=0 u1=0 c1=1 c2=0 ys=0 yv=10 gb0=1 gb1=1",
            "T|t=13 a=1 b=0 g1=1 g2=1 u1=0 c1=1 c2=0 ys=0 yv=10 gb0=1 gb1=1",
            "T|t=14 a=1 b=0 g1=1 g2=1 u1=0 c1=1 c2=0 ys=1 yv=10 gb0=1 gb1=1",
            "T|t=20 a=1 b=1 g1=1 g2=1 u1=0 c1=1 c2=0 ys=1 yv=10 gb0=1 gb1=1",
            "T|t=22 a=1 b=1 g1=1 g2=1 u1=0 c1=1 c2=1 ys=1 yv=10 gb0=1 gb1=1",
        ],
        "{o:?}"
    );
}

/// The stage nets that carry a net delay (a driver side, per-bit stages, a
/// delayed driver's own net) are not design objects: a dump lists the
/// design's nets only.
#[test]
fn net_delay_stages_stay_out_of_dumps() {
    xezim::compiler::simulator::set_wave_enabled(true);
    let mut path = std::env::temp_dir();
    path.push(format!("xezim_net_delay_stages_{}.vcd", std::process::id()));
    let _ = std::fs::remove_file(&path);
    let src = r#"
module top;
  logic a = 0, b = 0;
  logic [1:0] v = 0;
  wire [1:0] #2 wv; assign wv = v;
  wire #2 w2; assign #1 w2 = a; assign w2 = b;
  initial begin
    $dumpfile("{VCD}");
    $dumpvars(0, top);
    #10 a = 1; v = 2'b01;
    #10 $finish;
  end
endmodule
"#
    .replace("{VCD}", path.to_str().unwrap());
    simulate(&src, 1000).expect("simulate failed");
    let vcd = std::fs::read_to_string(&path).expect("no VCD written");
    let _ = std::fs::remove_file(&path);
    let mut vars: Vec<&str> = vcd
        .lines()
        .filter(|l| l.starts_with("$var"))
        .filter_map(|l| l.split_whitespace().nth(4))
        .collect();
    vars.sort();
    assert_eq!(vars, ["a", "b", "v", "w2", "wv"], "{vcd}");
}
