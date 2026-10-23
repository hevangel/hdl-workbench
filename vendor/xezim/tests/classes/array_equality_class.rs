//! Unpacked array equality / inequality (IEEE 1800-2017 §11.4.5) on
//! fixed-size arrays — including CLASS-PROPERTY arrays reached as
//! `obj.a`, `this.a`, or bare `a` inside a method.
//!
//! Previously the `==`/`!=` handler only matched the case where BOTH
//! operands were bare `Ident`s resolving into `module.arrays` (module-level
//! fixed arrays). A comparison like `rhs_.sa == this.sa` inside a class
//! `do_compare` (the UVM `uvm_object` compare path) parsed the right operand
//! as a `MemberAccess`, so the handler fell through to scalar comparison and
//! returned `1` (equal) regardless of the element values — `do_compare`
//! reported a spurious MISCOMPARE after a correct copy/pack.

use xezim::simulate;

fn u(sim: &xezim::compiler::Simulator, n: &str) -> u64 {
    sim.get_signal(n)
        .or_else(|| sim.get_signal(&format!("tb.{}", n)))
        .unwrap_or_else(|| panic!("signal not found: {}", n))
        .to_u64()
        .unwrap_or_else(|| panic!("{} not u64-able", n))
}

/// §11.4.5 — equality of two class-property fixed arrays, compared from
/// INSIDE a method (`this.sa == rhs.sa`) and across two objects
/// (`a.eq(b)`).
#[test]
fn class_fixed_array_equality() {
    const SRC: &str = "class C;
  int sa[3];
  function bit eq(C rhs);
    return sa == rhs.sa;
  endfunction
  function bit ne(C rhs);
    return sa != rhs.sa;
  endfunction
endclass

module tb;
  int failures = 0;
  initial begin
    C a = new;
    C b = new;
    a.sa[0]=11; a.sa[1]=22; a.sa[2]=33;

    // Identical arrays compare equal / not-not-equal.
    b.sa[0]=11; b.sa[1]=22; b.sa[2]=33;
    if (a.eq(b) != 1) failures++;
    if (a.ne(b) != 0) failures++;

    // Any single element differing breaks equality.
    b.sa[2]=99;
    if (a.eq(b) != 0) failures++;
    if (a.ne(b) != 1) failures++;

    b.sa[0]=11; b.sa[1]=22; b.sa[2]=33;
    b.sa[1]=0;
    if (a.eq(b) != 0) failures++;

    b.sa[0]=11; b.sa[1]=22; b.sa[2]=33;
    b.sa[0]=0;
    if (a.eq(b) != 0) failures++;
  end
endmodule
";
    let sim = simulate(SRC, 100).expect("simulate failed");
    assert_eq!(
        u(&sim, "failures"),
        0,
        "§11.4.5 class fixed-array == / != must compare element-by-element"
    );
}

/// §11.4.5 — module-level fixed arrays (the pre-existing path) must keep
/// working after the handler was generalised.
#[test]
fn module_fixed_array_equality() {
    const SRC: &str = "module tb;
  int x[3];
  int y[3];
  int eq_cnt = 0;
  int ne_cnt = 0;
  initial begin
    x[0]=1; x[1]=2; x[2]=3;
    y[0]=1; y[1]=2; y[2]=3;
    if (x == y) eq_cnt++;
    y[2]=9;
    if (x != y) ne_cnt++;
  end
endmodule
";
    let sim = simulate(SRC, 100).expect("simulate failed");
    assert_eq!(u(&sim, "eq_cnt"), 1, "equal module arrays");
    assert_eq!(u(&sim, "ne_cnt"), 1, "differing module arrays");
}
