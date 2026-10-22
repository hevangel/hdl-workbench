//! §23.3.3 — a module's INPUT port net driven from INSIDE the module by a
//! nested instance's output.
//!
//! A module's own statements are rewritten through the instance's
//! formal→actual port map, but a nested instantiation's connection ACTUALS
//! were rewritten with an EMPTY map — they only picked up the instance prefix.
//! So `module_A(input p); drv u(.o(p));` tied `drv`'s output to the FORMAL's
//! own signal (`u_a.p`) instead of the parent net, and the value never
//! propagated up: the parent read `z` while the driver sat one level down.
//!
//! That is the shape a bound interface takes — `bind module_A xmrs_if u(.d(p))`
//! with `assign d = 1'b0` inside the interface — so a checker driving a DUT
//! net through a bind silently drove nothing, and every hierarchical sample
//! below the port read `z` too.
//!
//! Reference behaviour, pinned below: the drive reaches the parent net AND
//! every reader down the hierarchy, and a genuine conflict (parent driving 1,
//! child driving 0) resolves to `x` on BOTH sides rather than one side
//! silently winning.

use xezim::simulate;

/// A child instance's output drives the enclosing module's input-port net;
/// the parent net and the deep readers must all see it.
const DRIVE_UP: &str = r#"
interface drv_if (output logic o);
  assign o = 1'b0;
endinterface
module leaf_cell (input v);
endmodule
module module_A (input v);
  drv_if u_i (.o(v));          // drives the port net from inside
  leaf_cell u_b (.v(v));
  leaf_cell u_c (.v(v));
endmodule
module mid (input v);
  module_A u_a (.v(v));
endmodule
module tb;
  wire w;                       // undriven by the testbench itself
  mid u_m (.v(w));
  int ok;
  initial begin
    #10;
    ok = (w === 1'b0)
      && (u_m.u_a.u_b.v === 1'b0)
      && (u_m.u_a.u_c.v === 1'b0);
  end
endmodule
"#;

/// Parent drives 1, child drives 0 — a real conflict resolves to x on both
/// sides (before, the parent's driver silently won on both).
const CONFLICT: &str = r#"
module drv_mod (output logic o);
  assign o = 1'b0;
endmodule
module module_A (input v);
  drv_mod u_m (.o(v));
endmodule
module tb;
  wire w;
  assign w = 1'b1;
  module_A u_a (.v(w));
  int ok;
  initial begin
    #10;
    ok = (u_a.v === 1'bx) && (w === 1'bx);
  end
endmodule
"#;

/// Same drive-up, but the inner instance connects with `.*` instead of named
/// ports, and the driven port sits one level further up (`core.flag_sig` is the
/// undriven parent wire). The implicit-connection paths (`.*` and `.p`) built
/// their actual as `prefix + name` directly, bypassing the enclosing instance's
/// formal->actual map — so the connection bound to the FORMAL's own signal and
/// the drive never reached the parent net. Explicit connections were unaffected,
/// which is what made this look like a bind bug rather than a wildcard one.
const WILDCARD_DRIVE_UP: &str = r#"
interface ctrl_if (output logic flag_sig, output logic ack_sig);
  assign flag_sig = 1'b0;
  assign ack_sig  = 1'b0;
endinterface
module glue (input wire flag_in, input wire ack_in, output logic out_en);
  assign out_en = ~flag_in & ~ack_in;
endmodule
module engine (input flag_sig, input ack_sig);
  glue u_glue (.flag_in(flag_sig), .ack_in(ack_sig), .out_en());
endmodule
bind engine ctrl_if ctrl_if_inst (.*);      // wildcard connection
module core;
  wire flag_sig;                            // NO driver but the bound interface
  wire ack_sig;
  engine u_engine (.flag_sig(flag_sig), .ack_sig(ack_sig));
endmodule
module tb;
  core u_core ();
  int ok;
  initial begin
    #10;
    ok = (u_core.flag_sig === 1'b0)                       // reached the parent wire
      && (u_core.u_engine.flag_sig === 1'b0)              // and the formal
      && (u_core.u_engine.u_glue.flag_in === 1'b0)        // and a deeper reader
      && (u_core.u_engine.u_glue.out_en === 1'b1);        // and downstream logic
  end
endmodule
"#;

fn ok_flag(sim: &xezim::compiler::Simulator) -> u64 {
    sim.get_signal("ok")
        .or_else(|| sim.get_signal("tb.ok"))
        .expect("signal 'ok' not found")
        .to_u64()
        .unwrap_or(0)
}

#[test]
fn inner_driver_reaches_parent_net_and_deep_readers() {
    let sim = simulate(DRIVE_UP, 1000).expect("simulate failed");
    assert_eq!(
        ok_flag(&sim),
        1,
        "the parent net and/or a deep reader did not see the driver inside the port"
    );
}

#[test]
fn parent_and_child_driver_conflict_resolves_x_on_both_sides() {
    let sim = simulate(CONFLICT, 1000).expect("simulate failed");
    assert_eq!(
        ok_flag(&sim),
        1,
        "a 0-vs-1 net conflict across the port did not resolve to x on both sides"
    );
}

#[test]
fn wildcard_connected_inner_driver_reaches_parent_net() {
    let sim = simulate(WILDCARD_DRIVE_UP, 1000).expect("simulate failed");
    assert_eq!(
        ok_flag(&sim),
        1,
        "a `.*`-connected bound interface driving an input port did not reach the parent net"
    );
}
