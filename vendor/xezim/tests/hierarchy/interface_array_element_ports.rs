//! §25.3.3 — connecting an ELEMENT of an interface array to a module port.
//! Reference-validated.
//!
//! Elements are registered as literally-named instances (`barr[0]`), so the
//! targets existed — only the connection map missed them, and the port stayed
//! unbound in both directions with no diagnostic. Three shapes had to be
//! handled, because the same source text parses three different ways:
//!
//! * `barr[0]` is an `Index` node; both insert sites matched a bare `Ident`
//!   only, so nothing was inserted at all.
//! * `barr[0].mp` can be one `Ident` whose path segment carries the select —
//!   the join mapped only segment NAMES and dropped the selects, yielding
//!   `barr.mp`, which the modport strip reduced to `barr`. The index was lost.
//! * `barr[0].mp` can also be a `MemberAccess` over an `Index`.
//!
//! Passing the whole array (`bus_if b[4]`) always worked, which made this look
//! narrower than it was.
//!
//! Note on coverage: a drive through a MODPORT-qualified element connection is
//! deliberately not asserted here. The reference leaves the target x for that
//! combination — and does so for a modport-typed port generally, arrays or not
//! — so it is a separate semantic question from the connection mapping this
//! test covers.

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
  modport snk (input data);
endinterface
"#;

/// An element connection drives and is read back, alongside a whole-instance
/// control and an untouched element.
#[test]
fn interface_array_element_connects_both_ways() {
    let src = format!(
        "{IFACE}
module drv(bus_if b, input logic [7:0] v);     initial b.data = v; endmodule
module mon(bus_if b, output logic [7:0] seen); always_comb seen = b.data; endmodule
module tb;
  bus_if a[3]();
  bus_if sng();
  logic [7:0] seen;
  drv d0(a[0], 8'hA0);
  drv d2(sng,  8'hA2);
  mon m0(a[0], seen);
  int e0, s, rd, untouched;
  initial begin
    #1;
    e0 = a[0].data;
    s  = sng.data;
    rd = seen;
    untouched = $isunknown(a[2].data);
  end
endmodule
"
    );
    let sim = simulate(&src, 50).expect("simulate failed");
    assert_eq!(
        u(&sim, "e0"),
        0xA0,
        "an element connection carries the child's drive"
    );
    assert_eq!(
        u(&sim, "rd"),
        0xA0,
        "and is readable through a second element port"
    );
    assert_eq!(
        u(&sim, "s"),
        0xA2,
        "a whole-instance connection still works"
    );
    assert_eq!(u(&sim, "untouched"), 1, "an unconnected element stays x");
}

/// A MODPORT-qualified element connection, read side — the shape that used to
/// collapse to the array's base name and lose the index.
#[test]
fn modport_qualified_element_reads_the_right_element() {
    let src = format!(
        "{IFACE}
module mon_mp(bus_if.snk b, output logic [7:0] seen); always_comb seen = b.data; endmodule
module mon_pl(bus_if b,     output logic [7:0] seen); always_comb seen = b.data; endmodule
module tb;
  bus_if r[2]();
  logic [7:0] s_mp0, s_mp1, s_pl0;
  mon_mp m0(r[0].snk, s_mp0);
  mon_mp m1(r[1].snk, s_mp1);
  mon_pl p0(r[0],     s_pl0);
  int v_mp0, v_mp1, v_pl0;
  initial begin
    r[0].data = 8'hC0;
    r[1].data = 8'hC1;
    #1;
    v_mp0 = s_mp0; v_mp1 = s_mp1; v_pl0 = s_pl0;
  end
endmodule
"
    );
    let sim = simulate(&src, 50).expect("simulate failed");
    assert_eq!(u(&sim, "v_mp0"), 0xC0, "element 0 through a modport");
    assert_eq!(u(&sim, "v_mp1"), 0xC1, "element 1 — not element 0's value");
    assert_eq!(u(&sim, "v_pl0"), 0xC0, "the plain spelling agrees");
}

/// Element connections inside a generate loop must each reach their own
/// element rather than collapsing onto one.
#[test]
fn interface_array_elements_in_a_generate_loop() {
    let src = format!(
        "{IFACE}
module drv(bus_if b, input logic [7:0] v); initial b.data = v; endmodule
module tb;
  bus_if g[3]();
  genvar i;
  generate
    for (i = 0; i < 3; i++) begin : gen
      drv d(g[i], 8'hB0 + i);
    end
  endgenerate
  int v0, v1, v2;
  initial begin
    #1;
    v0 = g[0].data; v1 = g[1].data; v2 = g[2].data;
  end
endmodule
"
    );
    let sim = simulate(&src, 50).expect("simulate failed");
    assert_eq!(
        (u(&sim, "v0"), u(&sim, "v1"), u(&sim, "v2")),
        (0xB0, 0xB1, 0xB2)
    );
}
