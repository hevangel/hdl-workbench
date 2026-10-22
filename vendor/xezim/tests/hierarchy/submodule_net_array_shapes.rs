//! §6.10 / §7.4 — an aggregate NET declared inside an instantiated SUBMODULE
//! kept none of its shape metadata.
//!
//! The submodule inlining path's `NetDeclaration` arm created a single scalar
//! signal per declarator and ignored `decl.dimensions` and the packed
//! dimensions entirely. So inside a submodule:
//!
//!   * `wire [15:0] pipe [0:2];`   -> `pipe[0]` was a 1-BIT select (`$bits` = 1)
//!   * `wire [1:0][15:0] packed2;` -> `packed2[0]` was a 1-BIT select
//!
//! The identical declarations at TOP level always worked, which is what made
//! this look like a generate/genvar problem: the original reproducer only
//! failed because the array lived in a submodule. A port fed from such an
//! element carried one bit, so every multi-bit path through a pipeline of
//! instances silently collapsed while the 1-bit paths passed.
//!
//! Widths and values below are reference-simulator verified.
//!
//! STILL OPEN (deliberately not asserted here): a 2-D unpacked net in a
//! submodule (`wire [15:0] grid [0:1][0:1];`) reads back X. The shape is now
//! registered, but the element access path does not yet resolve it — a separate
//! gap from the 1-D and packed-multi-D forms fixed here.

use xezim::simulate;

fn get(sim: &xezim::compiler::Simulator, name: &str) -> u64 {
    sim.get_signal(name)
        .or_else(|| sim.get_signal(&format!("tb.{}", name)))
        .unwrap_or_else(|| panic!("signal not found: {}", name))
        .to_u64()
        .unwrap_or_else(|| panic!("signal {} not u64-able", name))
}

const SUB_NET_SHAPES: &str = r#"
module relay #(parameter W = 8) (input [W-1:0] d, output logic [W-1:0] q);
  always_comb q = d;
endmodule

module holder (input [15:0] seed, output [15:0] o_vec, o_packed);
  wire [15:0]      pipe [0:2];   // 1-D unpacked array of vectors
  wire [1:0][15:0] packed2;      // packed multi-D

  assign pipe[0]    = seed;
  assign packed2[0] = seed;

  // Route one element through a real instance port, the path that collapsed.
  relay #(.W(16)) u_relay (.d(pipe[0]), .q(pipe[1]));

  assign o_vec    = pipe[1];
  assign o_packed = packed2[0];
endmodule

module tb;
  logic [15:0] seed;
  wire  [15:0] o_vec, o_packed;
  int  w_elem, w_packed;
  logic [15:0] seen_vec, seen_packed;
  holder dut (.seed(seed), .o_vec(o_vec), .o_packed(o_packed));
  initial begin
    seed = 16'hBEEF;
    #2;
    w_elem      = $bits(dut.pipe[0]);
    w_packed    = $bits(dut.packed2[0]);
    seen_vec    = o_vec;
    seen_packed = o_packed;
  end
endmodule
"#;

#[test]
fn submodule_net_aggregates_keep_their_shape() {
    let sim = simulate(SUB_NET_SHAPES, 100).expect("simulate failed");
    // Element widths: both were 1 before.
    assert_eq!(get(&sim, "w_elem"), 16);
    assert_eq!(get(&sim, "w_packed"), 16);
    // Values, including one crossing an instance port.
    assert_eq!(get(&sim, "seen_vec") & 0xFFFF, 0xBEEF);
    assert_eq!(get(&sim, "seen_packed") & 0xFFFF, 0xBEEF);
}
