//! §7.4.1 / §6.24.1 — an ELEMENT of a multi-dimensional PACKED array read
//! inside an explicit SIZE CAST, in a submodule. Reference-validated.
//!
//! `eval_expr_ctx` (the context-width evaluator, which only an explicit size
//! cast routes through) had no case for a packed multi-D element select and
//! fell through to a one-BIT read, yielding X. Every other spelling of the
//! same read was correct, which is what made it so hard to see:
//!
//! ```text
//!   narrow = arr[1];          // 01        correct
//!   wide   = arr[1];          // 00000001  correct (implicit widening)
//!   cat    = {6'b0, arr[1]};  // 00000001  correct
//!   cast   = 8'(arr[1]);      // 0000000x  WRONG
//! ```
//!
//! In real RTL the visible symptom was `1 << arr[i]` and `arr[i] + 1` going X
//! inside a submodule, poisoning everything downstream — a lane-enable mask
//! read all-X while the array it came from printed perfectly.
//!
//! The 1-bit-element case matters just as much: a parameterized design
//! produces `[N-1:0][0:0]` whenever the inner dimension collapses (here
//! `$clog2(2) - 1 == 0`), and that shape took the same wrong path.

use xezim::simulate;

fn u(sim: &xezim::compiler::Simulator, n: &str) -> u64 {
    sim.get_signal(n)
        .or_else(|| sim.get_signal(&format!("tb.{}", n)))
        .unwrap_or_else(|| panic!("signal not found: {}", n))
        .to_u64()
        .unwrap_or_else(|| panic!("{} is x/z", n))
}

#[test]
fn packed_2d_element_survives_a_size_cast_in_a_submodule() {
    let src = r#"
module sub (input logic [1:0] seed);
  logic [3:0][1:0] arr;
  logic [7:0] w_implicit, w_cast, w_concat, w_shift, w_add;
  logic [1:0] narrow;
  always_comb begin
    arr = '{default:'d0};
    for (int i = 0; i < 4; i++) arr[i] = seed + i;
    narrow     = arr[1];
    w_implicit = arr[1];
    w_concat   = {6'b0, arr[1]};
    w_cast     = 8'(arr[1]);
    w_shift    = 8'(1 << arr[1]);
    w_add      = 8'(arr[1] + 1);
  end
endmodule
module tb;
  logic [1:0] seed;
  sub u(seed);
  int r_narrow, r_impl, r_cat, r_cast, r_shift, r_add;
  initial begin
    seed = 2'b00;
    #1;
    r_narrow = u.narrow; r_impl = u.w_implicit; r_cat = u.w_concat;
    r_cast = u.w_cast; r_shift = u.w_shift; r_add = u.w_add;
  end
endmodule
"#;
    let sim = simulate(src, 50).expect("simulate failed");
    // arr = {3,2,1,0} packed, so arr[1] == 1.
    assert_eq!(u(&sim, "r_narrow"), 1, "same-width read");
    assert_eq!(u(&sim, "r_impl"), 1, "implicit widening");
    assert_eq!(u(&sim, "r_cat"), 1, "inside a concatenation");
    assert_eq!(u(&sim, "r_cast"), 1, "inside an explicit size cast");
    assert_eq!(u(&sim, "r_shift"), 2, "as a shift amount inside a cast");
    assert_eq!(u(&sim, "r_add"), 2, "in arithmetic inside a cast");
}

/// A collapsed inner dimension (`[N-1:0][0:0]`) — what a parameterized design
/// produces when `$clog2(2) - 1 == 0`.
#[test]
fn one_bit_packed_element_survives_a_size_cast() {
    let src = r#"
module sub #(parameter P_N = 2, parameter P_LN = $clog2(P_N)) (input logic [7:0] ptr);
  logic [P_N-1:0][P_LN-1:0] idx;
  logic [P_N-1:0] en;
  always_comb begin
    en  = '0;
    idx = '{default:'d0};
    for (int i = 0; i < P_N; i++) begin
      idx[i] = ptr[P_LN-1:0] + i;
      en |= (1 << idx[i]);
    end
  end
endmodule
module tb;
  logic [7:0] ptr;
  sub u(ptr);
  int r_en, r_idx;
  initial begin
    ptr = 8'h00;
    #1;
    r_en = u.en; r_idx = u.idx;
  end
endmodule
"#;
    let sim = simulate(src, 50).expect("simulate failed");
    assert_eq!(u(&sim, "r_idx"), 0b10, "idx[0]=0, idx[1]=1");
    assert_eq!(
        u(&sim, "r_en"),
        0b11,
        "both lanes enable; a 1-bit element must not read X"
    );
}
