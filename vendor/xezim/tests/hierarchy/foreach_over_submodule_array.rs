//! §12.7.3 — a `foreach` over an array declared inside an instantiated
//! SUBMODULE resolved the array name unscoped.
//!
//! The existing scoping used `name_resolve_hint`, which is only installed while
//! a PROCESS runs. Two things went wrong from there:
//!
//!   * the loop BOUNDS: `foreach_dims` looked up the bare name, missed, and
//!     returned None, so a multi-var `foreach (m[i,j])` never iterated its
//!     rectangle;
//!   * the body WRITE: `m[i][j] = v` resolved the bare base name, found no
//!     registered array, and the write was silently dropped.
//!
//! Both now fall back to the scope hint and then to a UNIQUE suffix match —
//! `<scope>.m` is accepted only when exactly one registered array ends that
//! way, so this can never choose between same-named arrays in sibling
//! instances.
//!
//! The `always_comb` variant needed two more pieces:
//!   * scope inference (`infer_scope_from_rw_sets`) only scanned
//!     `signal_name_to_id`, and an unpacked ARRAY has no entry there (only
//!     per-element names), so an entry whose anchor was an array base got NO
//!     scope hint at all — the array registries are now scanned alongside;
//!   * a compiled always block runs `StmtFallback` insns through the AST
//!     interpreter with no hint installed; blocks that carry a fallback insn
//!     (precomputed `CompiledBlock::has_fallback`) now install the entry's
//!     scope hint around `exec_insns`.

use xezim::simulate;

fn get(sim: &xezim::compiler::Simulator, name: &str) -> u64 {
    sim.get_signal(name)
        .or_else(|| sim.get_signal(&format!("tb.{}", name)))
        .unwrap_or_else(|| panic!("signal not found: {}", name))
        .to_u64()
        .unwrap_or_else(|| panic!("signal {} not u64-able", name))
}

const SRC: &str = r#"
module holder (output [15:0] res_a, res_b);
  logic [15:0] grid [0:1][0:1];
  initial foreach (grid[i, j]) grid[i][j] = 16'hBEEF;
  assign res_a = grid[0][0];
  assign res_b = grid[1][1];   // last element: proves the rectangle iterated
endmodule

module tb;
  wire  [15:0] res_a, res_b;
  logic [15:0] seen_a, seen_b;
  holder dut (.res_a(res_a), .res_b(res_b));
  initial begin
    #3;
    seen_a = res_a;
    seen_b = res_b;
  end
endmodule
"#;

#[test]
fn foreach_over_a_two_dim_submodule_array_writes_every_element() {
    let sim = simulate(SRC, 100).expect("simulate failed");
    assert_eq!(get(&sim, "seen_a") & 0xFFFF, 0xBEEF);
    assert_eq!(get(&sim, "seen_b") & 0xFFFF, 0xBEEF);
}

/// The settle-path variant: the same foreach inside an `always_comb`, plus a
/// plain 1-D foreach. Both dropped their writes before the scope-inference and
/// fallback-hint fixes.
const COMB_SRC: &str = r#"
module holder (input [15:0] seed, output [15:0] res_1d, res_2d);
  logic [15:0] lane [0:3];
  logic [15:0] grid [0:1][0:1];
  always_comb foreach (lane[i])    lane[i]    = seed;
  always_comb foreach (grid[i, j]) grid[i][j] = seed;
  assign res_1d = lane[2];
  assign res_2d = grid[1][0];
endmodule

module tb;
  logic [15:0] seed;
  wire  [15:0] res_1d, res_2d;
  logic [15:0] seen_1d, seen_2d;
  holder dut (.seed(seed), .res_1d(res_1d), .res_2d(res_2d));
  initial begin
    seed = 16'hBEEF;
    #3;
    seen_1d = res_1d;
    seen_2d = res_2d;
  end
endmodule
"#;

#[test]
fn foreach_in_always_comb_inside_a_submodule_writes_its_array() {
    let sim = simulate(COMB_SRC, 100).expect("simulate failed");
    assert_eq!(get(&sim, "seen_1d") & 0xFFFF, 0xBEEF);
    assert_eq!(get(&sim, "seen_2d") & 0xFFFF, 0xBEEF);
}
