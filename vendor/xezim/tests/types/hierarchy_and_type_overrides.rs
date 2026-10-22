//! Three reference-validated fixes from a resolution/override audit.
//!
//! 1. §23.6 `$root.` — parsed as an unknown SYSTEM CALL, so every
//!    `$root.tb.x` read was a silent 32-bit zero and every write vanished.
//!    Now routed into the hierarchical parser and stripped in resolution
//!    (constant path selects rendered, so `$root.tb.gl[1].x` reaches
//!    for-generate scopes).
//! 2. §6.20.3/§26.3 — a TYPE-parameter override naming a package-scoped type
//!    (`#(.T(pkg::wide_t))`) was silently DROPPED: the converter required a
//!    single-segment identifier, and expression context lowers `pkg::t` to
//!    MemberAccess. The port stayed at the DEFAULT type's width.
//! 3. §6.18 — a SUB-module-local typedef's bare-table entry leaked past its
//!    instance (last-writer-wins, no restore), so a later module with no
//!    local `t` sized its `t v;` from whichever instance inlined last —
//!    order-dependent. Bare inserts now ride the same save/restore rail as
//!    type-parameter overrides.

use xezim::simulate;

fn u(sim: &xezim::compiler::Simulator, n: &str) -> u64 {
    sim.get_signal(n)
        .or_else(|| sim.get_signal(&format!("tb.{}", n)))
        .unwrap_or_else(|| panic!("signal not found: {}", n))
        .to_u64()
        .unwrap_or_else(|| panic!("{} is x/z", n))
}

#[test]
fn root_prefixed_hierarchical_read_and_write() {
    let src = r#"
module b_mod;
  logic [7:0] x = 8'hB1;
endmodule
module tb;
  logic [7:0] x = 8'h01;
  b_mod u_b();
  logic [7:0] r_top, r_b, r_top2, r_b2;
  initial begin
    #1;
    r_top = $root.tb.x;
    r_b   = $root.tb.u_b.x;
    $root.tb.x      = 8'h77;
    $root.tb.u_b.x  = 8'hB7;
    #1;
    r_top2 = x;
    r_b2   = u_b.x;
  end
endmodule
"#;
    let sim = simulate(src, 50).expect("simulate failed");
    assert_eq!(u(&sim, "r_top"), 0x01, "$root read of the top's own signal");
    assert_eq!(u(&sim, "r_b"), 0xB1, "$root read through an instance");
    assert_eq!(
        u(&sim, "r_top2"),
        0x77,
        "$root WRITE landed (was silently dropped)"
    );
    assert_eq!(u(&sim, "r_b2"), 0xB7);
}

#[test]
fn scoped_type_param_override_reaches_port_widths() {
    let src = r#"
package tpk;
  typedef logic [31:0] wide_t;
endpackage
module chan #(parameter type T = logic [7:0]) (input T din, output T dout);
  assign dout = din >> 4;
  logic [31:0] w_in;
  assign w_in = $bits(din);
endmodule
module tb;
  logic [31:0] r;
  chan #(.T(tpk::wide_t)) u_c (.din(32'hDEAD_BEEF), .dout(r));
  logic [31:0] w_port;
  assign w_port = $bits(u_c.dout);
  initial #1;
endmodule
"#;
    let sim = simulate(src, 50).expect("simulate failed");
    let g = |n: &str| sim.get_signal(n).unwrap().to_u64().unwrap();
    assert_eq!(
        g("u_c.w_in"),
        32,
        "port took the OVERRIDE's width, not the default 8"
    );
    assert_eq!(g("w_port"), 32);
    assert_eq!(g("r"), 0x0DEA_DBEE, "32-bit data flows, not an 8-bit slice");
}

#[test]
fn submodule_local_typedef_does_not_leak_past_its_instance() {
    let src = r#"
typedef logic [15:0] t;          // $unit
module m1;
  typedef logic [63:0] t;        // module-LOCAL, must not leak
  t v1 = 64'hFEDC_BA98_7654_3210;
endmodule
module m2;
  t v2 = 16'h1234;               // must be the 16-bit $unit t
  logic [31:0] w2;
  assign w2 = $bits(v2);
endmodule
module tb;
  m1 u1();                        // the leaking order: m1 first
  m2 u2();
  initial #1;
endmodule
"#;
    let sim = simulate(src, 50).expect("simulate failed");
    let g = |n: &str| sim.get_signal(n).unwrap().to_u64().unwrap();
    assert_eq!(
        g("u2.w2"),
        16,
        "m2's t is the $unit 16-bit type — was 64 when m1 inlined first"
    );
    assert_eq!(g("u2.v2"), 0x1234);
    assert_eq!(
        g("u1.v1"),
        0xFEDC_BA98_7654_3210,
        "m1 keeps its own 64-bit local"
    );
}
