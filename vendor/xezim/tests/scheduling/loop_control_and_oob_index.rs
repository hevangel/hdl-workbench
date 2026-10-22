//! Four defects surfaced by running the Icarus `ivtest` SystemVerilog
//! regression list, each confirmed against a reference simulator (which passes
//! the corresponding ivtest cases).
//!
//! 1. **§12.7.2 `continue` in `while` / `do…while`.** Only the `for` loop
//!    cleared `continue_flag` between iterations. Left set, the body's first
//!    statement saw the stale flag and skipped the whole block on every pass —
//!    so the loop variable never advanced and the loop spun forever. A
//!    `while` containing a `continue` HUNG the simulator.
//! 2. **§12.7.2 `break` out of a timing-free `forever`.** The zero-delay
//!    livelock guard ran the body to its stall cap and then reported a false
//!    STALL, killing the run — even though the loop exits via `break`. The
//!    statements after the loop have to run too.
//! 3. **§7.4.6 out-of-bounds ELEMENT write.** An index containing x or z
//!    selects nothing, so the write is discarded; folding the index through
//!    `to_u64` turned it into 0 and clobbered element 0 (`a['hx] += f()`).
//!    The right-hand side is still evaluated — its side effects stand.
//! 4. **§20.6.2 `$bits` of a dimensioned typedef.** `typedef int T[3:0]` is
//!    128 bits; the typedef table holds only the ELEMENT width, so `$bits(T)`
//!    answered 32.

use xezim::simulate;

fn u(sim: &xezim::compiler::Simulator, n: &str) -> u64 {
    sim.get_signal(n)
        .or_else(|| sim.get_signal(&format!("top.{}", n)))
        .unwrap_or_else(|| panic!("signal not found: {}", n))
        .to_u64()
        .unwrap_or_else(|| panic!("{} not u64-able (x/z?)", n))
}

/// `continue` in every loop form. Before the fix the `while` case did not
/// terminate at all.
#[test]
fn continue_advances_every_loop_form() {
    let src = r#"
module top;
  integer w_idx, d_idx, r_idx, f_idx, fe_idx;
  int arr[4];
  initial begin
    w_idx = 0;
    while (w_idx < 5) begin
      w_idx += 1;
      if (w_idx < 2) continue;
    end
    d_idx = 0;
    do begin
      d_idx += 1;
      if (d_idx < 2) continue;
    end while (d_idx < 5);
    r_idx = 0;
    repeat (5) begin
      r_idx += 1;
      if (r_idx < 2) continue;
    end
    for (f_idx = 0; f_idx < 5; f_idx = f_idx + 1) begin
      if (f_idx < 2) continue;
    end
    fe_idx = 0;
    foreach (arr[i]) begin
      fe_idx += 1;
      if (i < 2) continue;
    end
  end
endmodule
"#;
    let sim = simulate(src, 50).expect("simulate failed (a hang here means continue regressed)");
    assert_eq!(u(&sim, "w_idx"), 5, "while");
    assert_eq!(u(&sim, "d_idx"), 5, "do-while");
    assert_eq!(u(&sim, "r_idx"), 5, "repeat");
    assert_eq!(u(&sim, "f_idx"), 5, "for");
    assert_eq!(u(&sim, "fe_idx"), 4, "foreach");
}

/// `break` leaves a timing-free `forever` and execution CONTINUES after it,
/// with no false stall report.
#[test]
fn break_exits_a_timing_free_forever_and_resumes_after_it() {
    let src = r#"
module top;
  integer idx, after_ran, second_loop;
  initial begin
    idx = 0;
    forever begin
      idx += 1;
      if (idx >= 2) break;
    end
    after_ran = 1;
    second_loop = 0;
    forever begin
      second_loop += 1;
      if (second_loop < 3) continue;
      break;
    end
  end
endmodule
"#;
    let sim = simulate(src, 50).expect("simulate failed");
    assert_eq!(u(&sim, "idx"), 2, "the forever exits on break");
    assert_eq!(u(&sim, "after_ran"), 1, "and the statements after it run");
    assert_eq!(
        u(&sim, "second_loop"),
        3,
        "continue then break in a forever"
    );
}

/// §7.4.6: an out-of-bounds or x/z element write is discarded, while the
/// right-hand side is still evaluated.
#[test]
fn out_of_bounds_element_writes_are_discarded_but_rhs_runs() {
    let src = r#"
module top;
  logic [39:0] a[1:0];
  integer i;
  logic [39:0] j = 0;
  int a0, a1, calls;
  function logic [39:0] f;
    j++;
    return j;
  endfunction
  initial begin
    a[0] = 23;
    a[1] = 42;
    a[-1]  += f();
    a[2]   += f();
    a['hx] += f();
    i = -1;  a[i] += f();
    i = 2;   a[i] += f();
    i = 'hx; a[i] += f();
    a0 = a[0]; a1 = a[1]; calls = j;
  end
endmodule
"#;
    let sim = simulate(src, 50).expect("simulate failed");
    assert_eq!(
        u(&sim, "a0"),
        23,
        "element 0 untouched by every out-of-range write"
    );
    assert_eq!(u(&sim, "a1"), 42, "and element 1");
    assert_eq!(u(&sim, "calls"), 6, "the RHS ran all six times");
}

/// §20.6.2: unpacked dimensions on a typedef count toward `$bits`.
#[test]
fn bits_of_a_dimensioned_typedef() {
    let src = r#"
module top;
  typedef int T1;
  typedef int T2[3:0];
  typedef logic [7:0] T3[2];
  typedef int T4[1:0][3:0];
  int arr[3:0];
  int b1, b2, b3, b4, bsig, bkw;
  initial begin
    b1 = $bits(T1); b2 = $bits(T2); b3 = $bits(T3); b4 = $bits(T4);
    bsig = $bits(arr);
    bkw  = $bits(int);
  end
endmodule
"#;
    let sim = simulate(src, 50).expect("simulate failed");
    assert_eq!(u(&sim, "b1"), 32, "a plain typedef is unchanged");
    assert_eq!(u(&sim, "b2"), 128, "4 x 32");
    assert_eq!(u(&sim, "b3"), 16, "2 x 8");
    assert_eq!(u(&sim, "b4"), 256, "2 x 4 x 32");
    assert_eq!(u(&sim, "bsig"), 128, "a declared array still works");
    assert_eq!(u(&sim, "bkw"), 32, "and a bare type keyword");
}
