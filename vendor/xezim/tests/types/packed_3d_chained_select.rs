//! §7.4.1 — CHAINED constant element selects on a 3-D packed array,
//! `v[i][j][k]` on `logic [0:0][1:0][1:0]`. Reference-validated.
//!
//! Only the innermost `Index` node has an `Ident` base, so the bytecode
//! compiler's element-select path (which requires an Ident base) handled one
//! level and the outer selects fell through to plain bit selects: `v[0]` gave
//! a 4-bit slice, `[0]` took ONE BIT of it, and the final `[1]` indexed past
//! the end and produced x.
//!
//! The AST interpreter walks the whole chain correctly, so `$display` printed
//! the right bit while the SAME expression in an `assign` or an `if` guard
//! read x. Field symptom: `if (vld[0][0][1])` never fired although waves and
//! prints showed the bit at 1 — half of a checker silently skipped.

use xezim::simulate;

fn u(sim: &xezim::compiler::Simulator, n: &str) -> u64 {
    sim.get_signal(n)
        .or_else(|| sim.get_signal(&format!("tb.{}", n)))
        .unwrap_or_else(|| panic!("signal not found: {}", n))
        .to_u64()
        .unwrap_or_else(|| panic!("{} is x/z — the chained select degraded to a bit select", n))
}

const SRC: &str = r#"
module tb;
  logic clk = 0;
  always #5 clk = ~clk;

  logic [0:0][1:0][1:0] v;

  // Discriminating pattern: [0][0]=2'b10, [0][1]=2'b01, so every leaf bit
  // differs from its neighbours and any offset error flips an expectation.
  logic a000, a001, a010, a011;
  assign a000 = v[0][0][0];
  assign a001 = v[0][0][1];
  assign a010 = v[0][1][0];
  assign a011 = v[0][1][1];

  // Two-level select of a 3-D array: a 2-bit slice, not a bit.
  logic [1:0] mid;
  assign mid = v[0][1];

  // DYNAMIC indices in the chain — the shape a
  //   for (i) for (j) if (vld[i][j][0])
  // checker loop produces. The constant path cannot fold these, so they used
  // to fall through to a plain bit select and read x; testbenches hit this as
  // "loops over multi-dim arrays do not work" and hand-unrolled to dodge it.
  int di, dj;
  logic dyn_full, dyn_mixed;
  logic [1:0] dyn_slice;
  assign dyn_full  = v[di][dj][1];   // every level dynamic but the last
  assign dyn_mixed = v[0][dj][1];    // mix of constant and dynamic
  assign dyn_slice = v[di][dj];      // dynamic chain stopping one level short

  // The guard shape that silently never fired. Gated and snapshotted so the
  // count is race-free: `snapshot` is read one time-unit AFTER the third
  // edge's activation has fully settled, and `stop` freezes the counter —
  // otherwise the count depends on same-timestep process ordering between
  // this block and the stimulus initial, which the LRM leaves undefined.
  int fired, snapshot, stop;
  always @(posedge clk) if (!stop && v[0][0][1]) fired++;

  initial begin
    fired = 0; stop = 0;
    di = 0; dj = 0;
    v = '0;
    v[0][0] = 2'b10;
    v[0][1] = 2'b01;
    repeat (3) @(posedge clk);
    #1;
    snapshot = fired;
    stop = 1;
  end
endmodule
"#;

#[test]
fn chained_selects_read_the_declared_element() {
    let sim = simulate(SRC, 200).expect("simulate failed");
    assert_eq!(u(&sim, "a000"), 0, "v[0][0][0]");
    assert_eq!(u(&sim, "a001"), 1, "v[0][0][1] — read x before the fix");
    assert_eq!(u(&sim, "a010"), 1, "v[0][1][0]");
    assert_eq!(u(&sim, "a011"), 0, "v[0][1][1] — read x before the fix");
    assert_eq!(u(&sim, "mid"), 0b01, "v[0][1] two-level slice");
}

#[test]
fn dynamic_indices_in_a_chain_select_the_declared_element() {
    let sim = simulate(SRC, 200).expect("simulate failed");
    // v[0][0] = 2'b10, v[0][1] = 2'b01  =>  v[0][0][1] is 1, v[0][1] is 2'b01
    assert_eq!(
        u(&sim, "dyn_full"),
        1,
        "v[di][dj][1] with di=dj=0 — read x before the fix"
    );
    assert_eq!(
        u(&sim, "dyn_mixed"),
        1,
        "v[0][dj][1] — constant and dynamic levels mixed"
    );
    assert_eq!(
        u(&sim, "dyn_slice"),
        0b10,
        "v[di][dj] — dynamic chain stopping one level short"
    );
}

#[test]
fn chained_select_in_an_if_guard_fires() {
    let sim = simulate(SRC, 200).expect("simulate failed");
    assert_eq!(
        u(&sim, "snapshot"),
        3,
        "if (v[0][0][1]) must fire every posedge; an x guard never fires \
         while $display of the same expression prints 1"
    );
}
