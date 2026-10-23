//! §6.20 — a parameter may reference one declared LATER in the same module.
//!
//! Parameters were evaluated where they appeared, so
//!
//!     localparam int L = $clog2(SIZE) - 6;
//!     localparam int SIZE = 4096;
//!
//! saw nothing for `SIZE`, fell back to 0, and produced -6. Read back as
//! unsigned that is 4294967290, so `logic [L-1:0] bus;` became a
//! `[4294967289:0]` packed range, tripped the sane-width cap, and collapsed
//! the whole bus to ONE BIT. The design then simulated wrongly with only a
//! width warning to show for it — this was seen on a real customer run where
//! a write-buffer bus silently became 1 bit wide.
//!
//! Parameters are now resolved to a fixpoint BEFORE the item walk, so
//! declarations sized from them are correct regardless of declaration order.
//!
//! The pre-pass is deliberately conservative — it seeds a parameter only when
//! every name its initializer reads is already resolved, and never touches:
//!   * package/class-scoped references (`pkg::X`), which are not bound until
//!     the walk reaches the import,
//!   * real-valued parameters, and
//!   * unbased-unsized literals (`'1`), which are self-determined per §6.20.2.
//! Each of those has its own handling in the walk, and seeding them would
//! replace a correct value with a wrong one. Every case below is checked
//! against a reference simulator.

use xezim::simulate;

fn get(sim: &xezim::compiler::Simulator, name: &str) -> u64 {
    sim.get_signal(name)
        .or_else(|| sim.get_signal(&format!("tb.{}", name)))
        .unwrap_or_else(|| panic!("signal not found: {}", name))
        .to_u64()
        .unwrap_or_else(|| panic!("signal {} not u64-able", name))
}

const SRC: &str = r#"
`define MACRO_SIZE 4096

package sz_pkg;
  localparam int PKG_SIZE = 4096;
  localparam int PKG_LOG  = $clog2(PKG_SIZE);
endpackage

function automatic int log2_fn(input int v);
  int r; begin r = 0; while ((1 << r) < v) r = r + 1; log2_fn = r; end
endfunction

module tb;
  import sz_pkg::*;
  localparam int LOCAL_SIZE = 4096;

  localparam int L1 = $clog2(LOCAL_SIZE) - 6;        // backward ref
  localparam int L2 = $clog2(sz_pkg::PKG_SIZE) - 6;  // package-scoped
  localparam int L3 = sz_pkg::PKG_LOG - 6;           // package param
  localparam int L4 = $clog2(`MACRO_SIZE) - 6;       // macro
  localparam int L5 = log2_fn(LOCAL_SIZE) - 6;       // user sizing function
  localparam int L6 = $clog2(LOCAL_SIZE/64);
  localparam int L7 = $clog2(LATER_SIZE) - 6;        // FORWARD ref
  localparam int LATER_SIZE = 4096;
  localparam int MID = LOCAL_SIZE / 64;
  localparam int L8  = $clog2(MID);

  // Widths built from each: these collapse when the parameter reads 0.
  logic [L1-1:0] w1; logic [L2-1:0] w2; logic [L3-1:0] w3; logic [L4-1:0] w4;
  logic [L5-1:0] w5; logic [L6-1:0] w6; logic [L7-1:0] w7; logic [L8-1:0] w8;

  int v1, v2, v3, v4, v5, v6, v7, v8;
  int b1, b2, b3, b4, b5, b6, b7, b8;
  initial begin
    #1;
    v1 = L1; v2 = L2; v3 = L3; v4 = L4; v5 = L5; v6 = L6; v7 = L7; v8 = L8;
    b1 = $bits(w1); b2 = $bits(w2); b3 = $bits(w3); b4 = $bits(w4);
    b5 = $bits(w5); b6 = $bits(w6); b7 = $bits(w7); b8 = $bits(w8);
  end
endmodule
"#;

#[test]
fn parameter_may_reference_one_declared_later() {
    let sim = simulate(SRC, 100).expect("simulate failed");
    // Every form evaluates to 6 ...
    for n in ["v1", "v2", "v3", "v4", "v5", "v6", "v7", "v8"] {
        assert_eq!(get(&sim, n) as u32, 6, "{} should be 6", n);
    }
    // ... and every dependent declaration is 6 bits wide. `v7`/`b7` are the
    // forward reference: before the fix they were -6 (4294967290) and 8.
    for n in ["b1", "b2", "b3", "b4", "b5", "b6", "b7", "b8"] {
        assert_eq!(get(&sim, n) as u32, 6, "{} should be 6 bits", n);
    }
}

/// A chain of forward references has to resolve one link per fixpoint pass.
const CHAIN: &str = r#"
module tb;
  localparam int A = B + 1;
  localparam int B = C + 1;
  localparam int C = D + 1;
  localparam int D = 10;
  logic [A-1:0] bus;
  int va, vb, vc, vd, wbits;
  initial begin
    #1;
    va = A; vb = B; vc = C; vd = D; wbits = $bits(bus);
  end
endmodule
"#;

#[test]
fn a_chain_of_forward_references_resolves() {
    let sim = simulate(CHAIN, 100).expect("simulate failed");
    assert_eq!(get(&sim, "vd") as u32, 10);
    assert_eq!(get(&sim, "vc") as u32, 11);
    assert_eq!(get(&sim, "vb") as u32, 12);
    assert_eq!(get(&sim, "va") as u32, 13);
    assert_eq!(get(&sim, "wbits") as u32, 13);
}
