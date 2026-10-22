//! No-parens array query methods (`.size`, `.num`) on dynamic arrays and
//! queues — both as **constraint operands** and as **expression values**.
//!
//! IEEE 1800-2017 §7.5.2, §7.9.1, §7.10.2.1:
//!   * `.size()` / `.num()` may be written WITHOUT parentheses: `da.size`,
//!     `q.size`, `aa.num`. The LRM's own examples use the no-parens form
//!     (e.g. §7.5.2 `if (addr.size)` ...).
//!   * For a constrained-random class, the size of a `rand` collection must be
//!     solvable from a no-parens constraint operand such as
//!     `q.size inside {[1:N]}`, which the parser lowers to a
//!     `MemberAccess(Ident(q), size)` (NOT a `Call`).
//!
//! These exercise both the *constraint solver* path (`is_size_call` must
//! recognise the MemberAccess shape) and the *expression evaluator*
//! (`MemberAccess` arm of `eval_expr_ctx` must yield the live size for a class
//! queue / dynamic-array property).

use xezim::simulate;

fn u(sim: &xezim::compiler::Simulator, n: &str) -> u64 {
    sim.get_signal(n)
        .or_else(|| sim.get_signal(&format!("tb.{}", n)))
        .unwrap_or_else(|| panic!("signal not found: {}", n))
        .to_u64()
        .unwrap_or_else(|| panic!("{} not u64-able", n))
}

/// §18.4 — a `rand` queue/dynamic-array constrained by its no-parens `.size`
/// operand. `q.size inside {[1:11]}` parses to `MemberAccess(Ident(q), size)`;
/// the solver must treat it as a size call (sized first, then the elements
/// exist).
#[test]
fn noparen_size_in_rand_constraint() {
    const SRC: &str = "class Cq;
  rand bit [7:0] q[$];
  constraint sizing { q.size inside {[1:11]}; }
  constraint values { foreach (q[i]) { q[i] == (i + 1); } }
endclass

class Cd;
  rand bit [7:0] da[];
  constraint sizing { da.size inside {[2:6]}; }
  constraint values { foreach (da[i]) { da[i] == (i * 2); } }
endclass

module tb;
  int q_bad = 0;
  int da_bad = 0;
  initial begin
    Cq cq = new();
    Cd cd = new();
    int st;
    repeat (40) begin
      st = cq.randomize();
      if (st != 1) q_bad++;
      if (cq.q.size < 1 || cq.q.size > 11) q_bad++;
      foreach (cq.q[i])
        if (cq.q[i] != (i + 1)) q_bad++;

      st = cd.randomize();
      if (st != 1) da_bad++;
      if (cd.da.size < 2 || cd.da.size > 6) da_bad++;
      foreach (cd.da[i])
        if (cd.da[i] != (i * 2)) da_bad++;
    end
  end
endmodule
";
    let sim = simulate(SRC, 100).expect("simulate failed");
    assert_eq!(
        u(&sim, "q_bad"),
        0,
        "§7.10.2.1 no-parens queue `.size` as constraint operand and value"
    );
    assert_eq!(
        u(&sim, "da_bad"),
        0,
        "§7.5.2 no-parens dynamic-array `.size` as constraint operand and value"
    );
}

/// §7.9.1 / §7.10.2.1 — no-parens `.num`/`.size` read from INSIDE a class
/// method (the MemberAccess arm of `eval_expr_ctx`): the parser lowers
/// `aa.num` to `MemberAccess(Ident(aa), num)`, which must yield the live key
/// count / element count of the property.
#[test]
fn noparen_num_size_inside_method() {
    const SRC: &str = "class Counter;
  bit [7:0] aa[int];
  bit [7:0] q[$];
  function int aa_count(); return aa.num; endfunction
  function int q_count();  return q.size; endfunction
endclass

module tb;
  int failures = 0;
  initial begin
    Counter c = new();
    c.aa[10] = 8'h11;
    c.aa[20] = 8'h22;
    c.aa[30] = 8'h33;
    c.q.push_back(8'h01);
    c.q.push_back(8'h02);
    if (c.aa_count() != 3) failures++;
    if (c.q_count() != 2) failures++;
    c.aa.delete(20);
    c.q.pop_back();
    if (c.aa_count() != 2) failures++;
    if (c.q_count() != 1) failures++;
  end
endmodule
";
    let sim = simulate(SRC, 100).expect("simulate failed");
    assert_eq!(
        u(&sim, "failures"),
        0,
        "§7.9.1/§7.10.2.1 no-parens `.num`/`.size` read inside a class method"
    );
}
