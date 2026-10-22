//! Two elaboration gaps found while comparing a customer trace against a
//! reference simulator.
//!
//! 1. §6.10 — an identifier that appears ONLY as an instance PORT ACTUAL was
//!    never implicitly declared. `create_implicit_nets` scanned continuous
//!    assigns only, so a net threaded between two instances existed nowhere:
//!    reading it by name raised "Undeclared identifier", and it was missing
//!    from a `$dumpvars` trace even though the connection itself worked (the
//!    ports were wired to each other directly). A waveform viewer therefore
//!    could not show the net at all.
//!
//! 2. §7.4.2 — packed dimensions written on a STRUCT TYPEDEF
//!    (`some_t [0:0][1:0] arr;`) registered no element width, so `arr[i]`
//!    degraded to a 1-BIT select. The existing registration was gated on
//!    `width > struct_w`, which a single-element `[0:0]` array never satisfies,
//!    and a single element width cannot describe a multi-level packed array
//!    anyway.

use xezim::simulate;

fn get(sim: &xezim::compiler::Simulator, name: &str) -> u64 {
    sim.get_signal(name)
        .or_else(|| sim.get_signal(&format!("tb.{}", name)))
        .unwrap_or_else(|| panic!("signal not found: {}", name))
        .to_u64()
        .unwrap_or_else(|| panic!("signal {} not u64-able", name))
}

/// `link_net` is never declared — it is inferred purely from the two port
/// connections, so referencing it by name only works if §6.10 created it.
const IMPLICIT_PORT_NET: &str = r#"
module pulser (output wire o);
  reg r;
  initial r = 1'b0;
  always #5 r = ~r;
  assign o = r;
endmodule

module relay (input wire i, output wire o);
  assign o = i;
endmodule

module tb;
  wire  end_of_chain;
  logic observed;
  logic matched;
  pulser u_src (.o(link_net));
  relay  u_dst (.i(link_net), .o(end_of_chain));
  initial begin
    #7;
    observed = link_net;              // resolvable only if the net exists
    matched  = (link_net === end_of_chain);
  end
endmodule
"#;

#[test]
fn net_inferred_from_port_connections_is_reachable() {
    let sim = simulate(IMPLICIT_PORT_NET, 100).expect("simulate failed");
    // At t=7 the source has toggled once, so the inferred net is 1 and the
    // far end of the chain agrees with it.
    assert_eq!(get(&sim, "observed") & 1, 1);
    assert_eq!(get(&sim, "matched") & 1, 1);
}

/// `lanes` is 4x16 = 64 bits and `tag` 3, so one element is 67 bits.
const PACKED_TYPEDEF_ARRAY: &str = r#"
package shapes_pkg;
  typedef struct packed {
    logic [3:0] [15:0] lanes;
    logic [2:0]        tag;
  } parcel_t;
endpackage
import shapes_pkg::*;

module tb;
  parcel_t            single;    //  67
  parcel_t [0:0]      one_elem;  //  67 - the [0:0] case that never registered
  parcel_t [0:0][1:0] nested;    // 134
  int w_single, w_one, w_one_sel, w_nested, w_nested_sel, w_nested_sel2;
  initial begin
    w_single      = $bits(single);
    w_one         = $bits(one_elem);
    w_one_sel     = $bits(one_elem[0]);
    w_nested      = $bits(nested);
    w_nested_sel  = $bits(nested[0]);
    w_nested_sel2 = $bits(nested[0][0]);
  end
endmodule
"#;

#[test]
fn packed_dims_on_a_struct_typedef_size_element_selects() {
    let sim = simulate(PACKED_TYPEDEF_ARRAY, 100).expect("simulate failed");
    assert_eq!(get(&sim, "w_single"), 67);
    assert_eq!(get(&sim, "w_one"), 67);
    // Was 1: a `[0:0]` array registered no element width at all.
    assert_eq!(get(&sim, "w_one_sel"), 67);
    assert_eq!(get(&sim, "w_nested"), 134);
    // Selecting the outer dimension keeps the whole `[1:0]` sub-array.
    assert_eq!(get(&sim, "w_nested_sel"), 134);
    assert_eq!(get(&sim, "w_nested_sel2"), 67);
}
