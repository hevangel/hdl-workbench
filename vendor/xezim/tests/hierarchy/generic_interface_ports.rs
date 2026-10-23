//! §25.3.2 — GENERIC interface ports (`module m(interface b);`). All cases
//! below are reference-validated.
//!
//! A generic port names no interface, so the parser records the keyword
//! `interface` as the port's type name. Interface-port detection resolved that
//! name against the interface definitions, found nothing, and classified the
//! port as an ordinary data port. Its connection therefore went into the value
//! port map rather than the interface map, and `b.data` inside the child was
//! rewritten to `<connection>.data`.
//!
//! That is why simple cases passed and the real design did not: when the
//! connection is a bare instance name (`.b(bus)`), `<connection>.data` happens
//! to spell the correct signal `bus.data`, so the port worked by accident. As
//! soon as the connection carries a modport suffix — `.b(bus.tx)`, the usual
//! way a generic port is used, since the modport is what gives it direction —
//! the rewrite produced `bus.tx.data`, which names nothing. Every write through
//! the port was dropped and every read came back `x`, silently.
//!
//! The equivalent typed ports (`bus_if.tx b` / `bus_if b`) were unaffected,
//! which is what made this look like a modport bug rather than a port-kind one.

use xezim::simulate;

fn u(sim: &xezim::compiler::Simulator, n: &str) -> u64 {
    sim.get_signal(n)
        .or_else(|| sim.get_signal(&format!("tb.{}", n)))
        .unwrap_or_else(|| panic!("signal not found: {}", n))
        .to_u64()
        .unwrap_or_else(|| panic!("{} not u64-able (x/z?)", n))
}

const IFACE: &str = r#"
interface bus_if;
  logic [7:0] data;
  logic [7:0] ack;
  modport tx (output data, input  ack);
  modport rx (input  data, output ack);
endinterface
"#;

/// The matrix that isolates it: port kind × connection form. Only the generic
/// port with a modport-qualified connection ever failed.
#[test]
fn generic_port_matrix_against_typed_ports() {
    let src = format!(
        "{IFACE}
module s_generic(interface b); initial b.data = 8'h11; endmodule
module s_typed  (bus_if.tx b); initial b.data = 8'h22; endmodule
module s_plain  (bus_if    b); initial b.data = 8'h33; endmodule
module tb;
  bus_if g_mp(), g_pl(), t_mp(), t_pl(), p_mp();
  s_generic a(.b(g_mp.tx));   // generic port, modport-qualified connection
  s_generic b(.b(g_pl));      // generic port, plain connection
  s_typed   c(.b(t_mp.tx));
  s_typed   d(.b(t_pl));
  s_plain   e(.b(p_mp.tx));
  int gen_mp, gen_pl, typ_mp, typ_pl, pln_mp;
  initial begin
    #1;
    gen_mp = g_mp.data; gen_pl = g_pl.data;
    typ_mp = t_mp.data; typ_pl = t_pl.data;
    pln_mp = p_mp.data;
  end
endmodule
"
    );
    let sim = simulate(&src, 50).expect("simulate failed");
    assert_eq!(
        u(&sim, "gen_mp"),
        0x11,
        "generic port, modport-qualified connection"
    );
    assert_eq!(u(&sim, "gen_pl"), 0x11, "generic port, plain connection");
    assert_eq!(u(&sim, "typ_mp"), 0x22, "typed modport port");
    assert_eq!(
        u(&sim, "typ_pl"),
        0x22,
        "typed modport port, plain connection"
    );
    assert_eq!(u(&sim, "pln_mp"), 0x33, "plain interface port");
}

/// Two instances of the same generic-port module must reach their own
/// interface, not share or cross one.
#[test]
fn generic_ports_on_two_instances_stay_distinct() {
    let src = format!(
        "{IFACE}
module sender(interface b);          initial b.data = 8'h77; endmodule
module recv(bus_if.rx b, output logic [7:0] o); always_comb o = b.data; endmodule
module tb;
  bus_if b0(), b1();
  logic [7:0] o0, o1;
  sender s0(.b(b0.tx));
  sender s1(.b(b1.tx));
  recv   r0(.b(b0), .o(o0));
  recv   r1(.b(b1), .o(o1));
endmodule
"
    );
    let sim = simulate(&src, 50).expect("simulate failed");
    assert_eq!(u(&sim, "o0"), 0x77);
    assert_eq!(u(&sim, "o1"), 0x77);
}

/// Reading through a generic port, driving and reading through the same one,
/// passing one down another level, and two on a single module.
#[test]
fn generic_ports_read_nest_and_multiply() {
    let src = format!(
        "{IFACE}
module g_recv(interface b, output logic [7:0] o); always_comb o = b.data; endmodule
module g_duplex(interface b);
  initial b.data = 8'h5a;
  always_comb b.ack = b.data ^ 8'hff;
endmodule
module g_leaf(interface b); initial b.data = 8'h3c; endmodule
module g_mid (interface b); g_leaf u(.b(b)); endmodule
module g_two (interface p, interface q);
  initial begin p.data = 8'h01; q.data = 8'h02; end
endmodule
module tb;
  bus_if w(), x(), z0(), z1();
  logic [7:0] o;
  g_duplex d(.b(w.tx));
  g_recv   r(.b(w.rx), .o(o));
  g_mid    m(.b(x.tx));
  g_two    t(.p(z0.tx), .q(z1));
  int dup, ack, recv, nest, two0, two1;
  initial begin
    #1;
    dup  = w.data; ack = w.ack; recv = o;
    nest = x.data; two0 = z0.data; two1 = z1.data;
  end
endmodule
"
    );
    let sim = simulate(&src, 50).expect("simulate failed");
    assert_eq!(u(&sim, "dup"), 0x5a, "generic port drives");
    assert_eq!(
        u(&sim, "ack"),
        0xa5,
        "generic port reads back what it drove"
    );
    assert_eq!(
        u(&sim, "recv"),
        0x5a,
        "a second generic port reads the same interface"
    );
    assert_eq!(
        u(&sim, "nest"),
        0x3c,
        "generic port handed down to another generic port"
    );
    assert_eq!(
        (u(&sim, "two0"), u(&sim, "two1")),
        (0x01, 0x02),
        "two generic ports on one module"
    );
}
