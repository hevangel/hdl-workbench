//! §6.7.1 — a net declared with a TYPEDEF'D element type and packed dims.
//! Reference-validated.
//!
//! Two failures:
//!  * `wire burst_t [0:0][1:0] x;` (BARE typedef) was a hard parse error —
//!    the type name was taken as the declarator, so the real name tripped
//!    "expected Semicolon". The net parser excluded `[` from its type
//!    lookahead to preserve `wire foo [3:0];` (a net NAMED foo); the
//!    disambiguator now looks PAST the balanced bracket groups — an
//!    identifier there means the first token was a type.
//!  * The scoped form parsed but registered no element metadata, so
//!    `$bits(x[0][0])` read 1 where the identical VARIABLE declaration read
//!    146. The net arm now mirrors the variable arm's typedef-array
//!    registration (full dims carry the struct width as the innermost entry).

use xezim::simulate;

fn u(sim: &xezim::compiler::Simulator, n: &str) -> u64 {
    sim.get_signal(n)
        .or_else(|| sim.get_signal(&format!("tb.{}", n)))
        .unwrap_or_else(|| panic!("signal not found: {}", n))
        .to_u64()
        .unwrap_or_else(|| panic!("{} is x/z", n))
}

const SRC: &str = r#"
package P;
  typedef struct packed {
    logic [1:0][63:0] lanes;
    logic [1:0][7:0]  mask;
    logic [1:0]       en;
  } burst_t;   // 146
endpackage
module tb;
  import P::*;
  wire P::burst_t [0:0][1:0] w_qual;
  wire burst_t    [0:0][1:0] w_bare;    // was a parse error
  logic [31:0] a, b, c, d;
  assign a = $bits(w_qual);
  assign b = $bits(w_qual[0][0]);
  assign c = $bits(w_bare);
  assign d = $bits(w_bare[0][0]);

  // Value flow through the net, element-wise — widths alone can lie.
  P::burst_t drv0, drv1;
  assign drv0 = {64'hAAAA_AAAA_AAAA_AAA1, 64'hBBBB_BBBB_BBBB_BBB0, 8'hC1, 8'hC0, 2'b10};
  assign drv1 = {64'hDDDD_DDDD_DDDD_DDD1, 64'hEEEE_EEEE_EEEE_EEE0, 8'hF1, 8'hF0, 2'b01};
  assign w_bare = {drv1, drv0};
  logic [63:0] lane00, lane11;
  logic [7:0]  m10;
  assign lane00 = w_bare[0][0].lanes[0];
  assign lane11 = w_bare[0][1].lanes[1];
  assign m10    = w_bare[0][1].mask[0];
  initial #1;
endmodule
"#;

#[test]
fn typedef_named_wire_with_packed_dims_parses_and_sizes() {
    let sim = simulate(SRC, 50).expect("simulate failed — the bare form used to be a parse error");
    assert_eq!(u(&sim, "a"), 292, "scoped whole");
    assert_eq!(u(&sim, "b"), 146, "scoped element");
    assert_eq!(u(&sim, "c"), 292, "bare whole");
    assert_eq!(u(&sim, "d"), 146, "bare element");
}

#[test]
fn values_flow_through_typedef_wire_elements() {
    let sim = simulate(SRC, 50).expect("simulate failed");
    assert_eq!(u(&sim, "lane00"), 0xBBBB_BBBB_BBBB_BBB0);
    assert_eq!(u(&sim, "lane11"), 0xDDDD_DDDD_DDDD_DDD1);
    assert_eq!(u(&sim, "m10"), 0xF0);
}
