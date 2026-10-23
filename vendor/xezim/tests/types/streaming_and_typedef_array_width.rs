//! Three defects found by one customer testbench (a monitor submodule fed by
//! streaming-reversed elements of a packed struct-typedef array):
//!
//! 1. `collect_expr_reads` had no `StreamOp` arm, so a continuous
//!    `assign dst = {<<{src}};` had an EMPTY read set — it sampled X at the
//!    time-0 settle and never re-fired when `src` changed. Procedural
//!    streaming was always fine; only the dependency collection was blind.
//!
//! 2. A submodule VARIABLE of struct-typedef-array type registered no element
//!    metadata under its scoped name, so `g[0]` inside the submodule was a
//!    1-bit select (ports and top-level declarations were covered by earlier
//!    fixes; the submodule variable arm was not).
//!
//! 3. The submodule width shortcut took the TYPEDEF's width for any
//!    `TypeReference`, ignoring packed dims on the reference — `some_t [1:0] g`
//!    was sized at ONE element, so writes to `g[1]` were clamped away and it
//!    read X. The top-level arms already guarded this with
//!    `dimensions.is_empty()`; the inline arm now does too.
//!
//! All values reference-simulator verified.

use xezim::simulate;

fn get(sim: &xezim::compiler::Simulator, name: &str) -> u64 {
    sim.get_signal(name)
        .or_else(|| sim.get_signal(&format!("tb.{}", name)))
        .unwrap_or_else(|| panic!("signal not found: {}", name))
        .to_u64()
        .unwrap_or_else(|| panic!("signal {} not u64-able", name))
}

const SRC: &str = r#"
package mirror_pkg;
  typedef struct packed {
    logic [63:0] pd;
    logic [7:0]  tag;
    logic        v;
    logic        e;
  } line_t;                                    // 74 bits
endpackage
import mirror_pkg::*;

module churn (input line_t [1:0] w);
  line_t [1:0] gen;
  assign gen[0] = {<<{w[0]}};                  // streaming cont-assign of an
  assign gen[1] = {<<{w[1]}};                  // element, inside a submodule
  logic [63:0] lo0, hi0, lo1, hi1;
  always @(*) begin
    lo0 = gen[0][63:0];
    hi0 = {54'b0, gen[0][73:64]};
    lo1 = gen[1][63:0];
    hi1 = {54'b0, gen[1][73:64]};
  end
endmodule

module tb;
  line_t [1:0] arr;
  churn u_c (.w(arr));
  logic [63:0] seen_lo0, seen_hi0, seen_lo1, seen_hi1;
  logic [15:0] plain_rev;
  logic [15:0] plain_src;
  assign plain_rev = {<<{plain_src}};          // stream cont-assign, top level
  initial begin
    arr[0] = {64'hA5A5_A5A5_B4B4_B4B4, 8'h11, 1'b1, 1'b0};
    arr[1] = {64'h5A5A_5A5A_C3C3_C3C3, 8'h22, 1'b0, 1'b1};
    plain_src = 16'h8001;
    #2;
    seen_lo0 = u_c.lo0;
    seen_hi0 = u_c.hi0;
    seen_lo1 = u_c.lo1;
    seen_hi1 = u_c.hi1;
  end
endmodule
"#;

#[test]
fn streaming_cont_assign_of_typedef_array_elements_in_a_submodule() {
    let sim = simulate(SRC, 100).expect("simulate failed");
    // {<<{elem}} = bit-reverse of {pd,tag,v,e}; expected values taken from a
    // reference-simulator run of this exact source, not hand-derived.
    assert_eq!(get(&sim, "seen_lo0"), 0x2d2d_2d2d_a5a5_a5a5);
    assert_eq!(get(&sim, "seen_hi0") & 0x3FF, 0x188);
    // Element 1 was the visible failure: sized away entirely before.
    assert_eq!(get(&sim, "seen_lo1"), 0xc3c3_c3c3_5a5a_5a5a);
    assert_eq!(get(&sim, "seen_hi1") & 0x3FF, 0x244);
    // Top-level streaming cont-assign re-fires on its operand.
    assert_eq!(get(&sim, "plain_rev") & 0xFFFF, 0x8001);
}
