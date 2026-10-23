//! §7.4 — a 2-D unpacked array inside an instantiated SUBMODULE was unusable:
//! writes to `m[i][j]` vanished and reads came back X. Three separate defects
//! stacked on the same declaration, each hidden behind the previous one.
//!
//! 1. The submodule VARIABLE arm sized the declarator with `extract_array_range`,
//!    which reports only the FIRST dimension — a 2-D declarator was registered as
//!    1-D, so storage came out as `m[0]`/`m[1]` and `m[i][j]` addressed nothing.
//!
//! 2. With storage correct, the continuous assign still latched X: the entry
//!    falls to the AST path, whose reads resolve BARE-first, so it depended on a
//!    same-named TOP-level signal while evaluating the scoped one. It sampled X
//!    on the first settle and never re-fired when the port copy arrived. The
//!    scoped id is now registered alongside the bare one (a superset dependency
//!    only costs an extra evaluation; missing one loses the value entirely).
//!
//! 3. The READER then still saw X: `collect_expr_reads` expands `arr[i]` to every
//!    element for a 1-D array, but a 2-D access nests two `Index` nodes, so only
//!    the array BASE name was registered — a name with no signal id, hence no
//!    dependency at all.
//!
//! The same declarations at TOP level always worked, which is what kept pointing
//! the investigation at generate/genvar constructs instead of the submodule.
//! Every value below is reference-simulator verified.

use xezim::simulate;

fn get(sim: &xezim::compiler::Simulator, name: &str) -> u64 {
    sim.get_signal(name)
        .or_else(|| sim.get_signal(&format!("tb.{}", name)))
        .unwrap_or_else(|| panic!("signal not found: {}", name))
        .to_u64()
        .unwrap_or_else(|| panic!("signal {} not u64-able", name))
}

const SRC: &str = r#"
module holder (input [15:0] seed, output [15:0] o_flat, o_grid, o_var);
  wire  [15:0] flat [0:3];        // 1-D control, always worked
  wire  [15:0] grid [0:1][0:1];   // 2-D net
  logic [15:0] vgrid [0:1][0:1];  // 2-D variable

  assign flat[0]    = seed;
  assign grid[0][0] = seed;
  initial vgrid[1][1] = 16'hCAFE;

  assign o_flat = flat[0];
  assign o_grid = grid[0][0];
  assign o_var  = vgrid[1][1];
endmodule

module tb;
  logic [15:0] seed;
  wire  [15:0] o_flat, o_grid, o_var;
  logic [15:0] seen_flat, seen_grid, seen_var, seen_inner;
  holder dut (.seed(seed), .o_flat(o_flat), .o_grid(o_grid), .o_var(o_var));
  initial begin
    seed = 16'hBEEF;
    #2;
    seen_flat  = o_flat;
    seen_grid  = o_grid;
    seen_var   = o_var;
    seen_inner = dut.grid[0][0];   // hierarchical read of the element itself
  end
endmodule
"#;

#[test]
fn two_dim_array_in_a_submodule_carries_values() {
    let sim = simulate(SRC, 100).expect("simulate failed");
    assert_eq!(get(&sim, "seen_flat") & 0xFFFF, 0xBEEF); // control
    // Net 2-D: was X through the port and through a hierarchical read.
    assert_eq!(get(&sim, "seen_grid") & 0xFFFF, 0xBEEF);
    assert_eq!(get(&sim, "seen_inner") & 0xFFFF, 0xBEEF);
    // Variable 2-D written procedurally: storage did not even exist before.
    assert_eq!(get(&sim, "seen_var") & 0xFFFF, 0xCAFE);
}
