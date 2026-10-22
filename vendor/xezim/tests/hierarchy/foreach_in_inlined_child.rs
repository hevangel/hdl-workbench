//! §12.7.3 — a `foreach` inside an instantiated module. Reference-validated.
//!
//! The inlining rewriter that rebases a child's names onto its instance had
//! arms for `for`, `while`, `repeat` and `forever` but none for `foreach`, so
//! the statement passed through untouched: both the array expression and every
//! name in the body kept the child's bare form.
//!
//! What made it hard to spot is that it half-worked. The runtime scope-hint
//! fallbacks rescue the loop bounds and the body's WRITES, so the loop iterates
//! the right number of times and fills the array correctly — an explicit
//! `for` loop afterwards reads all the right values. Only a READ of the array
//! from inside the foreach body fails, because that resolves through the base
//! array name, which no hint reaches. So `foreach (m[i,j]) sum += m[i][j];`
//! silently accumulated x while every other view of the same array was right.

use xezim::simulate;

fn u(sim: &xezim::compiler::Simulator, n: &str) -> u64 {
    sim.get_signal(n)
        .or_else(|| sim.get_signal(&format!("tb.{}", n)))
        .unwrap_or_else(|| panic!("signal not found: {}", n))
        .to_u64()
        .unwrap_or_else(|| panic!("{} not u64-able (x/z?)", n))
}

/// Reads inside a foreach body, in the child and from the parent, against an
/// explicit index loop over the same array.
#[test]
fn foreach_reads_a_multidim_array_in_an_instance() {
    let src = r#"
module leaf;
  int g [2][3];
  int one_d [4];
  int fe_sum, for_sum, fe_const, one_d_sum;
  initial begin
    foreach (g[i,j])   g[i][j] = i * 10 + j;
    foreach (one_d[k]) one_d[k] = k;
    #1;
    fe_sum = 0;  foreach (g[i,j]) fe_sum += g[i][j];
    for_sum = 0;
    for (int a = 0; a < 2; a++)
      for (int b = 0; b < 3; b++) for_sum += g[a][b];
    fe_const = 0; foreach (g[i,j]) fe_const += g[1][1];
    one_d_sum = 0; foreach (one_d[k]) one_d_sum += one_d[k];
  end
endmodule
module tb;
  leaf u();
  int p_fe, p_1d, p_explicit, c_fe, c_for, c_const, c_1d;
  initial begin
    #2;
    p_fe = 0; foreach (u.g[i,j]) p_fe += u.g[i][j];
    p_1d = 0; foreach (u.one_d[k]) p_1d += u.one_d[k];
    p_explicit = 0;
    for (int a = 0; a < 2; a++)
      for (int b = 0; b < 3; b++) p_explicit += u.g[a][b];
    c_fe = u.fe_sum; c_for = u.for_sum; c_const = u.fe_const; c_1d = u.one_d_sum;
  end
endmodule
"#;
    let sim = simulate(src, 50).expect("simulate failed");
    assert_eq!(
        u(&sim, "c_fe"),
        36,
        "child reads its own array inside foreach"
    );
    assert_eq!(u(&sim, "c_for"), 36, "explicit loop agrees");
    assert_eq!(
        u(&sim, "c_const"),
        66,
        "a constant index inside the loop body"
    );
    assert_eq!(u(&sim, "c_1d"), 6, "one dimension was never affected");
    assert_eq!(
        u(&sim, "p_fe"),
        36,
        "parent reads the child's array inside foreach"
    );
    assert_eq!(u(&sim, "p_explicit"), 36);
    assert_eq!(u(&sim, "p_1d"), 6);
}

/// The loop must iterate the full rectangle and bind both variables.
#[test]
fn foreach_binds_both_indices_in_an_instance() {
    let src = r#"
module leaf;
  int g [2][3];
  int iters, last_i, last_j;
  initial begin
    foreach (g[i,j]) g[i][j] = i * 10 + j;
    #1;
    iters = 0;
    foreach (g[i,j]) begin
      iters++; last_i = i; last_j = j;
    end
  end
endmodule
module tb;
  leaf u();
  int n, li, lj, p_n;
  initial begin
    #2;
    n = u.iters; li = u.last_i; lj = u.last_j;
    p_n = 0; foreach (u.g[i,j]) p_n++;
  end
endmodule
"#;
    let sim = simulate(src, 50).expect("simulate failed");
    assert_eq!(u(&sim, "n"), 6);
    assert_eq!((u(&sim, "li"), u(&sim, "lj")), (1, 2));
    assert_eq!(u(&sim, "p_n"), 6);
}
