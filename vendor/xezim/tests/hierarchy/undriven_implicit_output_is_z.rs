//! §6.6 / §23.2.2.3: an output port with no data type is a net. When nothing
//! drives it, the port and the parent net it connects to float at `z`.
//!
//! xezim used to start such a port at `x`, whether the port was ANSI,
//! non-ANSI or a vector. A variable output (`output logic`, `output reg`,
//! `output var`, or `output o; reg o;`) still starts at `x`. Expected values
//! are the reference simulator's.

#[test]
fn undriven_implicit_net_outputs_float_at_z() {
    let sim = xezim::simulate(
        r#"
module c1 (output o);
endmodule
module c2 (o);
  output o;
endmodule
module c3 (output [1:0] o);
endmodule
module top;
  wire w1, w2;
  wire [1:0] w3;
  c1 u1 (.o(w1));
  c2 u2 (.o(w2));
  c3 u3 (.o(w3));
  initial #1 $display("T|w1=%b w2=%b w3=%b u1.o=%b", w1, w2, w3, u1.o);
endmodule
"#,
        100,
    )
    .expect("simulate");
    let o: Vec<String> = sim.output.iter().map(|o| o.message.clone()).collect();
    assert_eq!(o, ["T|w1=z w2=z w3=zz u1.o=z"], "{o:?}");
}

fn out(src: &str) -> Vec<String> {
    let sim = xezim::simulate(src, 100).expect("simulate");
    sim.output
        .iter()
        .map(|o| o.message.clone())
        .filter(|m| m.starts_with("T|"))
        .collect()
}

/// Net outputs float at z: an implicit one, `output wire`, `output tri`, a
/// signed or non-ANSI vector, a non-ANSI port whose `wire` redeclaration keeps
/// it a net; a vector driven in part is z in the undriven bits. Variable
/// outputs start at x. A module's own undriven net and a hierarchical read of
/// the port agree.
#[test]
fn undriven_output_ports_by_kind() {
    let o = out(r#"
module c1 (output o);
  wire inner;
endmodule
module c2 (output wire o);
endmodule
module c3 (output [3:0] o);
  assign o[1:0] = 2'b10;
endmodule
module c4 (output tri o);
endmodule
module c5 (output logic o);
endmodule
module c6 (output reg o);
endmodule
module c7 (o);
  output o;
  reg o;
endmodule
module c8 (output var o);
endmodule
module c9 (o, p);
  output [1:0] o;
  output p;
  wire p;
endmodule
module c10 (output signed [1:0] o);
endmodule
module c11 (output o);
  assign o = 1'b1;
endmodule
module top;
  wire w1, w2, w4, w5, w6, w7, w8, p9, w11;
  wire [3:0] w3;
  wire [1:0] w9, w10;
  wire d;
  c1 u1 (.o(w1));
  c2 u2 (.o(w2));
  c3 u3 (.o(w3));
  c4 u4 (.o(w4));
  c5 u5 (.o(w5));
  c6 u6 (.o(w6));
  c7 u7 (.o(w7));
  c8 u8 (.o(w8));
  c9 u9 (.o(w9), .p(p9));
  c10 u10 (.o(w10));
  c11 u11 (.o(w11));
  initial begin
    #1 $display("T|w1=%b w2=%b w3=%b w4=%b w5=%b w6=%b w7=%b w8=%b w9=%b p9=%b w10=%b w11=%b d=%b", w1, w2, w3, w4, w5, w6, w7, w8, w9, p9, w10, w11, d);
    $display("T|u1.o=%b u1.inner=%b u2.o=%b u3.o=%b u5.o=%b u7.o=%b u9.o=%b u9.p=%b", u1.o, u1.inner, u2.o, u3.o, u5.o, u7.o, u9.o, u9.p);
  end
endmodule
"#);
    assert_eq!(
        o,
        [
            "T|w1=z w2=z w3=zz10 w4=z w5=x w6=x w7=x w8=x w9=zz p9=z w10=zz w11=1 d=z",
            "T|u1.o=z u1.inner=z u2.o=z u3.o=zz10 u5.o=x u7.o=x u9.o=zz u9.p=z",
        ],
        "{o:?}"
    );
}

/// The top module's own ports: an undriven net output floats at z, a
/// variable output is x.
#[test]
fn undriven_ports_of_the_top_module() {
    let o = out(r#"
module top (output o, output [1:0] v, output logic l);
  initial #1 $display("T|o=%b v=%b l=%b", o, v, l);
endmodule
"#);
    assert_eq!(o, ["T|o=z v=zz l=x",], "{o:?}");
}

/// A driven output reaches its parent at time 0 as an ordinary change, and a
/// sibling left undriven stays z.
#[test]
fn a_driven_output_port_changes_from_its_start_value() {
    let o = out(r#"
// edge at time 0 from the initial value: an always block on the parent net
module c1 (input i, output o);
  assign o = i;
endmodule
module c0 (output o);
endmodule
module top;
  logic i = 1;
  wire w, u;
  c1 k (.i(i), .o(w));
  c0 k0 (.o(u));
  initial $monitor("T|t=%0t w=%b u=%b k.o=%b", $time, w, u, k.o);
  always @(w) $display("T|w changed to %b", w);
  always @(posedge w) $display("T|posedge w");
endmodule
"#);
    assert_eq!(
        o,
        ["T|w changed to 1", "T|posedge w", "T|t=0 w=1 u=z k.o=1",],
        "{o:?}"
    );
}
