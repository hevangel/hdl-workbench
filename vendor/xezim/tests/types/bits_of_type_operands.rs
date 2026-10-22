//! §20.6.2 — `$bits` over TYPE operands, with and without packed dimensions.
//! Reference-validated.
//!
//! Two distinct parse shapes, two distinct failures:
//!
//!   * `$bits(logic [0:0][1:0][1:0])` — a data-type KEYWORD starts the
//!     operand, so it parses to a `TypeLiteral`. No const-eval arm handled
//!     that node, so the call fell through to the 0 default: `localparam int
//!     W = $bits(...)` became 0 and `logic [W-1:0]` declared a degenerate
//!     vector whose upper bits silently vanished.
//!
//!   * `$bits(pkg::t [0:0][1:0])` — a TYPE NAME is not a data-type keyword,
//!     so the parser cannot emit a TypeLiteral and produces a RangeSelect
//!     chain over the scoped name instead. `bits_of_signal_expr` sized that
//!     as a part-select by its own bounds: 2 instead of `$bits(t) * 1 * 2`.
//!     A pipeline stage declared `logic [DATA_BITS-1:0]` then held TWO bits
//!     instead of 292 and dropped every payload while widths elsewhere
//!     looked plausible.
//!
//! The guard added for the second case only fires when the range-chain base
//! names a type that is NOT also a signal, so a real part-select of a signal
//! keeps its ordinary sizing.

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
  typedef struct packed { logic [1:0][63:0] w; } st;
endpackage
typedef logic [7:0] byte_t;
typedef struct packed { logic [1:0][63:0] w; } lst;

module tb;
  localparam int A = $bits(byte_t);            // plain typedef
  localparam int B = $bits(byte_t [1:0]);      // local typedef + dims
  localparam int C = $bits(lst);               // local struct typedef
  localparam int D = $bits(lst [1:0]);         // local struct typedef + dims
  localparam int E = $bits(P::st);             // scoped, no dims
  localparam int F = $bits(P::st [1:0]);       // scoped + dims
  localparam int G = $bits(logic [1:0][7:0]);  // TypeLiteral, multi-dim
  localparam int H = $bits(logic [0:0][1:0][1:0]); // TypeLiteral, 3-D
  // Built-in integer ATOM types parse to a bare Ident (not a typedef, not a
  // parameter), so both lookups missed and the call answered 0.
  localparam int I = $bits(int);
  localparam int J = $bits(byte);
  localparam int K = $bits(shortint);
  localparam int L = $bits(longint);
  localparam int M = $bits(integer);

  // The guard must NOT capture a real part-select of a SIGNAL.
  logic [15:0] sig;
  int ps;
  initial ps = $bits(sig[7:0]);

  int a, b, c, d, e, f, g, h, i_, j_, k_, l_, m_;
  initial begin
    a = A; b = B; c = C; d = D; e = E; f = F; g = G; h = H;
    i_ = I; j_ = J; k_ = K; l_ = L; m_ = M;
  end

  // Localparam widths must be usable as DECLARED widths, not just values —
  // this is the shape that actually broke: a flattening stage.
  logic [F-1:0] flat;
  P::st [1:0]   arr;
  int roundtrip;
  initial begin
    arr = '0;
    arr[1].w[0] = 64'hDEAD_BEEF;
    #1;
    flat = arr;                 // must not truncate
    #1;
    roundtrip = (P::st'(flat[255:128]) === arr[1]);
  end
endmodule
"#;

#[test]
fn bits_of_type_literal_operands() {
    let sim = simulate(SRC, 50).expect("simulate failed");
    assert_eq!(u(&sim, "a"), 8, "$bits(byte_t)");
    assert_eq!(u(&sim, "b"), 16, "$bits(byte_t [1:0]) — typedef with dims");
    assert_eq!(u(&sim, "c"), 128, "$bits(lst)");
    assert_eq!(u(&sim, "d"), 256, "$bits(lst [1:0])");
    assert_eq!(u(&sim, "e"), 128, "$bits(P::st) — scoped typedef");
    assert_eq!(
        u(&sim, "f"),
        256,
        "$bits(P::st [1:0]) — the pipeline-stage shape; 2 meant part-select sizing"
    );
    assert_eq!(u(&sim, "g"), 16, "$bits(logic [1:0][7:0]) — TypeLiteral");
    assert_eq!(
        u(&sim, "h"),
        4,
        "$bits(logic [0:0][1:0][1:0]) — 0 meant the TypeLiteral arm was missing"
    );
}

#[test]
fn bits_of_builtin_atom_types() {
    let sim = simulate(SRC, 50).expect("simulate failed");
    assert_eq!(
        u(&sim, "i_"),
        32,
        "$bits(int) — 0 meant the atom-keyword fallback was missing"
    );
    assert_eq!(u(&sim, "j_"), 8, "$bits(byte)");
    assert_eq!(u(&sim, "k_"), 16, "$bits(shortint)");
    assert_eq!(u(&sim, "l_"), 64, "$bits(longint)");
    assert_eq!(u(&sim, "m_"), 32, "$bits(integer)");
}

#[test]
fn bits_guard_leaves_signal_part_selects_alone() {
    let sim = simulate(SRC, 50).expect("simulate failed");
    assert_eq!(
        u(&sim, "ps"),
        8,
        "$bits(sig[7:0]) is still the part-select width"
    );
}

#[test]
fn flattened_stage_declared_from_bits_holds_the_payload() {
    let sim = simulate(SRC, 50).expect("simulate failed");
    assert_eq!(
        u(&sim, "roundtrip"),
        1,
        "logic [$bits(P::st [1:0])-1:0] must be 256 bits wide; a 2-bit stage drops the payload"
    );
}
