//! §6.11.1 / §7.9 / §10.4 / §10.9.2 / §11.5.2 — storage semantics found by
//! the ch10/12/13 differential audit. Reference-validated.
//!
//! Five defects fixed together:
//!  * ≥2-D unpacked element reads lost signedness — `int a2[2][2];
//!    a2[0][0] = -1; a2[0][0] < 0` was FALSE (element storage for 2-D/N-D
//!    arrays and array-of-queue elements never inherited the declared
//!    element type's signedness; 1-D did).
//!  * part-select and bit-select WRITES to elements of block-local /
//!    subroutine-local unpacked arrays were silently dropped (the handlers
//!    all required a compact-table id, which runtime-declared arrays don't
//!    have).
//!  * foreach over an int-keyed associative array parsed keys as u64 —
//!    key -7 iterated as k=0 and read a phantom element.
//!  * `'{default:}` did not recurse into STRUCT-typed members (array
//!    members recursed fine), so old member values survived.

use xezim::simulate;

fn i(sim: &xezim::compiler::Simulator, n: &str) -> i64 {
    let v = sim
        .get_signal(n)
        .or_else(|| sim.get_signal(&format!("tb.{}", n)))
        .unwrap_or_else(|| panic!("signal not found: {}", n));
    let raw = v.to_u64().unwrap_or_else(|| panic!("{} is x/z", n));
    raw as u32 as i32 as i64
}

const SRC: &str = r#"
module tb;
  int a2 [2][2];
  int aq [2][$];
  int r_2d, r_2d_lt, r_aq, r_aq_lt;
  logic [7:0] r_ps, r_bit;
  int asc [int];
  int r_k0, r_v0, r_k1, r_v1;
  typedef struct {
    struct { int a; int b; } inner;
    int c;
  } n_t;
  n_t n;
  int r_na, r_nb, r_nk;
  initial begin
    a2[0][0] = -1;
    r_2d = a2[0][0]; r_2d_lt = (a2[0][0] < 0);
    aq[0].push_back(-8);
    r_aq = aq[0][0]; r_aq_lt = (aq[0][0] < 0);
    begin
      logic [7:0] larr [2];
      larr[1] = 8'h00;
      larr[1][7:4] = 4'hB;
      larr[1][2] = 1'b1;
      r_ps = larr[1];
    end
    asc[-7] = 70; asc[3] = 30;
    begin
      int step;
      step = 0;
      foreach (asc[k]) begin
        if (step == 0) begin r_k0 = k; r_v0 = asc[k]; end
        else begin r_k1 = k; r_v1 = asc[k]; end
        step++;
      end
    end
    n.inner.a = 111; n.inner.b = 22; n.c = 5;
    n = '{default: 2};
    r_na = n.inner.a; r_nb = n.inner.b; r_nk = n.c;
  end
endmodule
"#;

#[test]
fn procedural_storage_semantics() {
    let sim = simulate(SRC, 50).expect("simulate failed");
    assert_eq!(i(&sim, "r_2d"), -1, "2-D int element value");
    assert_eq!(i(&sim, "r_2d_lt"), 1, "2-D int element signed compare");
    assert_eq!(i(&sim, "r_aq"), -8, "array-of-queue element value");
    assert_eq!(
        i(&sim, "r_aq_lt"),
        1,
        "array-of-queue element signed compare"
    );
    assert_eq!(
        i(&sim, "r_ps"),
        0xb4,
        "block-local part+bit-select writes land"
    );
    assert_eq!(i(&sim, "r_k0"), -7, "foreach assoc first key numeric order");
    assert_eq!(i(&sim, "r_v0"), 70, "foreach assoc value at negative key");
    assert_eq!(i(&sim, "r_k1"), 3, "foreach assoc second key");
    assert_eq!(i(&sim, "r_v1"), 30, "foreach assoc second value");
    assert_eq!(
        i(&sim, "r_na"),
        2,
        "default recurses into struct member (a)"
    );
    assert_eq!(
        i(&sim, "r_nb"),
        2,
        "default recurses into struct member (b)"
    );
    assert_eq!(i(&sim, "r_nk"), 2, "default fills scalar member");
}
