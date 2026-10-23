//! §9.2.2.2 — `always_comb` re-fires on ELEMENT writes to queues, dynamic
//! arrays, and associative arrays. Reference-validated.
//!
//! Elements of these collections have no dependency edge of their own — a comb
//! read of `q[i]` depends on the collection's `.size` proxy. Whole-container
//! operations (`push_back`, `= new[n]`) resize, which marks the proxy, so they
//! woke readers — but a DIRECT element write (`q[0] = v`, `aa["k"] = v`) only
//! marked the element itself, and the block held its stale value for the rest
//! of the run. Associative arrays were worse: no proxy existed at all, so a
//! comb read of `aa[k]` registered NO dependency.
//!
//! The counters below are written only by their own always_comb, so they prove
//! the block does or does not EXECUTE — not merely what it read.

use xezim::simulate;

fn u(sim: &xezim::compiler::Simulator, n: &str) -> u64 {
    sim.get_signal(n)
        .or_else(|| sim.get_signal(&format!("tb.{}", n)))
        .unwrap_or_else(|| panic!("signal not found: {}", n))
        .to_u64()
        .unwrap_or_else(|| panic!("{} not u64-able (x/z?)", n))
}

#[test]
fn element_writes_wake_comb_readers() {
    let src = r#"
module tb;
  int q[$];
  int da[];
  int aa[string];
  int r_q, n_q, r_da, n_da, r_aa, n_aa;
  always_comb begin n_q++;  r_q  = (q.size()  > 0) ? q[0]   : -1; end
  always_comb begin n_da++; r_da = (da.size() > 0) ? da[0]  : -1; end
  always_comb begin n_aa++; r_aa = aa.exists("k") ? aa["k"] : -1; end
  int q_after, aa_after, da_after;
  initial begin
    q.push_back(11);
    #1 q[0] = 12;
    #1 q_after = r_q;
    da = new[1];
    #1 da[0] = 7;
    #1 da_after = r_da;
    aa["k"] = 9;
    #1 aa_after = r_aa;
  end
endmodule
"#;
    let sim = simulate(src, 100).expect("simulate failed");
    assert_eq!(
        u(&sim, "q_after"),
        12,
        "a queue element write re-fires the reader"
    );
    assert_eq!(
        u(&sim, "da_after"),
        7,
        "a dynamic-array element write re-fires the reader"
    );
    assert_eq!(
        u(&sim, "aa_after"),
        9,
        "an associative element write re-fires the reader"
    );
}

/// A fixed array with a literal `[N]` dimension must NOT be disturbed — that
/// shape also passes through the associative registration on its way to being
/// fixed, and giving it a proxy broke its name resolution.
#[test]
fn fixed_arrays_are_untouched() {
    let src = r#"
module tb;
  logic [9:0] pos [4];
  int r, n;
  always_comb begin n++; r = pos[0] + pos[1]; end
  int after;
  initial begin
    pos[0] = 10'd3;
    pos[1] = 10'd4;
    #1 after = r;
  end
endmodule
"#;
    let sim = simulate(src, 100).expect("simulate failed");
    assert_eq!(
        u(&sim, "after"),
        7,
        "fixed-array comb sensitivity still works"
    );
}
