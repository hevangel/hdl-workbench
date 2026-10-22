//! Four defects behind a customer clock-model testbench that hung at time 0
//! and then mis-reported clock activity.
//!
//! 1. `@(*)` NEVER SUSPENDED. `event_to_sens` yields an empty list for
//!    Star/ParenStar and the process path took a "just execute body"
//!    fallthrough, so `always @(*)` behaved like `forever`. Harmless for a
//!    purely combinational body, but with any timing control inside
//!    (`always @(*) begin #0; … end` — the "don't sample during delta cycles"
//!    idiom) it became an infinite delta loop and time never advanced.
//!    §9.4.2.2 sensitivity is now inferred from the guarded statement, and a
//!    body that reads nothing parks instead of spinning.
//!
//! 2. An expression STATEMENT recorded no writes, so `cnt++` left `cnt` in the
//!    block's own inferred sensitivity list and an `always @(*)` that
//!    incremented a counter re-triggered itself until the settle limit.
//!
//! 3. §6.10 implicit nets bound by a port connection INSIDE an instantiated
//!    module were never registered as signals. The driver then had no write id
//!    and the reader no read id, so the hop sat outside the settle dependency
//!    graph and updated only when some unrelated change forced a settle —
//!    silently dropping ~90% of the transitions on a fast clock. Declaring the
//!    same net explicitly made the flatline disappear, which is what pointed
//!    at the implicit-net path.
//!
//! 4. `#(.W($bits(some_signal)))` evaluated to 0: instance parameter overrides
//!    are const-evaluated against the parameter map only, and `$bits` of a
//!    SIGNAL found nothing. A port declared `[W-1:0]` then elaborated as
//!    `[-1:0]` — 2 bits — which surfaced as a bogus "port is 2 bit(s) but the
//!    connection is 74 bit(s)" warning even though the runtime width was fine.
//!
//! All expectations reference-simulator verified.

use xezim::simulate;

fn get(sim: &xezim::compiler::Simulator, name: &str) -> u64 {
    sim.get_signal(name)
        .or_else(|| sim.get_signal(&format!("tb.{}", name)))
        .unwrap_or_else(|| panic!("signal not found: {}", name))
        .to_u64()
        .unwrap_or_else(|| panic!("signal {} not u64-able", name))
}

/// (1) + (2): a `@(*)` with a `#0` body and a self-incrementing counter. Before
/// the fix this never returned; the counter also ran away on its own write.
const STAR: &str = r#"
module tb;
  logic p, q;
  integer bumps;
  integer ticks;
  initial begin bumps = 0; ticks = 0; p = 1'b0; q = 1'b1; end
  // Reads nothing that ever changes again -> must fire once, then park.
  always @(*) begin
    #0;
    if (p ^ q) bumps++;
  end
  // Reads nothing at all -> must park immediately, not spin at time 0.
  always @(*) begin #0; end
  logic beat;
  initial beat = 0;
  always #5 beat = ~beat;
  always @(posedge beat) ticks++;
  initial #100 $finish;
endmodule
"#;

#[test]
fn star_with_a_delay_body_suspends_instead_of_spinning() {
    let sim = simulate(STAR, 1000).expect("simulate failed");
    // Time actually advanced: 100 time units of a 10-unit clock = 10 posedges.
    assert_eq!(get(&sim, "ticks"), 10);
    // The counter fired once, not until a settle limit.
    assert_eq!(get(&sim, "bumps"), 1);
}

/// (3): a net that exists ONLY as a port actual one level down. `relay` is
/// undeclared inside `wrapper`; its every transition must reach the sink.
const IMPLICIT: &str = r#"
module wiggler (output logic w);
  initial w = 0;
  always #3 w = ~w;
endmodule

module sink (input wire s, output logic [15:0] seen);
  initial seen = 0;
  always @(s) seen = seen + 1;
endmodule

module wrapper (output logic [15:0] seen_out);
  // `relay` is never declared -> §6.10 implicit net, bound by port actuals.
  wiggler u_w (.w(relay));
  sink    u_s (.s(relay), .seen(seen_out));
endmodule

module tb;
  wire [15:0] seen;
  logic [15:0] snap;
  wrapper u_wrap (.seen_out(seen));
  initial begin
    #100;
    snap = seen;
  end
endmodule
"#;

#[test]
fn implicit_net_from_a_nested_port_connection_carries_every_edge() {
    let sim = simulate(IMPLICIT, 1000).expect("simulate failed");
    // 34 per a reference-simulator run of this exact source (33 toggles plus
    // the initial x->0 settle). Before the fix the implicit net sat outside
    // the dependency graph and this collapsed to a handful.
    assert_eq!(get(&sim, "snap") & 0xFFFF, 34);
}

/// (4): `$bits(<signal>)` as an instance parameter override.
const PARAM_BITS: &str = r#"
package widths_pkg;
  typedef struct packed { logic [63:0] pd; logic [7:0] tag; logic v; logic e; } lt;
endpackage

module gauge #(parameter W = 32) (input logic [W-1:0] s, output int w_out);
  assign w_out = W;
endmodule

module tb;
  widths_pkg::lt struct_sig;      // 74 bits
  logic [9:0]    plain_sig;       // 10 bits
  int w_struct, w_plain;
  gauge #(.W($bits(struct_sig))) u_a (.s(struct_sig), .w_out(w_struct));
  gauge #(.W($bits(plain_sig)))  u_b (.s(plain_sig),  .w_out(w_plain));
  int seen_struct, seen_plain;
  initial begin
    #1;
    seen_struct = w_struct;
    seen_plain  = w_plain;
  end
endmodule
"#;

#[test]
fn bits_of_a_signal_resolves_in_an_instance_parameter_override() {
    let sim = simulate(PARAM_BITS, 100).expect("simulate failed");
    assert_eq!(get(&sim, "seen_struct"), 74);
    assert_eq!(get(&sim, "seen_plain"), 10);
}
