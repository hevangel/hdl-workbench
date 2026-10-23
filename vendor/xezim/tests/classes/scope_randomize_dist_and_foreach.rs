//! `std::randomize(...) with { ... }` — four defects from two user testbenches
//! (EDA Playground repros), all reference-validated.
//!
//! 1. `en -> kind dist {...}` parsed the `->` as the EXPRESSION operator
//!    LogImplies before `dist` was visible, so the item became a dist over the
//!    1-bit implication RESULT — nothing targeted it, the weights were dropped,
//!    and only set-membership survived via resampling. The constraint parser
//!    now peels top-level LogImplies back into constraint implications.
//! 2. The parenthesized `(cond) -> (kind dist {...})` form — nonstandard, but
//!    a reference simulator warns and accepts it — was a hard parse error.
//!    The paren primary now captures the dist body in constraint context.
//! 3. `list.size() == N` on a dynamic-array target did nothing (§18.5.9
//!    `new[N]` semantics): the call reported success with the array left at
//!    its old size.
//! 4. `foreach (list[i])` over a DYNAMIC array iterated the registered
//!    sentinel bounds (0, -1) — an empty loop, so the body applied to nothing.
//!    And `foreach (vec[i])` over a PACKED vector's bits (guarding per-bit
//!    dist/equality on another target's bit-selects) was unsupported.

use xezim::simulate;

fn u(sim: &xezim::compiler::Simulator, n: &str) -> u64 {
    sim.get_signal(n)
        .or_else(|| sim.get_signal(&format!("top.{}", n)))
        .unwrap_or_else(|| panic!("signal not found: {}", n))
        .to_u64()
        .unwrap_or_else(|| panic!("{} not u64-able (x/z?)", n))
}

/// Implication into dist, both spellings, weights honored: 60 on value 1 vs
/// 20 spread over [2:15] must bias hard toward 1.
#[test]
fn implication_into_dist_keeps_its_weights() {
    let src = r#"
module top;
  int c1a, chia, c1b, chib, zero_seen, st_all;
  initial begin
    bit [3:0] kind;
    bit en;
    int i, st;
    en = 1; st_all = 1;
    for (i = 0; i < 200; i++) begin
      st = std::randomize(kind) with { en -> kind dist { 1 := 60, [2:15] :/ 20 }; };
      if (st != 1) st_all = 0;
      if (kind == 0) zero_seen++;
      else if (kind == 1) c1a++; else chia++;
    end
    for (i = 0; i < 200; i++) begin
      st = std::randomize(kind) with { (en == 1) -> ( kind dist { 1 := 60, [2:15] :/ 20 } ); };
      if (st != 1) st_all = 0;
      if (kind == 1) c1b++; else chib++;
    end
  end
endmodule
"#;
    let sim = simulate(src, 20).expect("simulate failed");
    assert_eq!(u(&sim, "st_all"), 1, "every call reports success");
    assert_eq!(u(&sim, "zero_seen"), 0, "0 is outside the dist set");
    assert!(
        u(&sim, "c1a") > u(&sim, "chia"),
        "unparenthesized: biased toward 1"
    );
    assert!(
        u(&sim, "c1b") > u(&sim, "chib"),
        "parenthesized: biased toward 1"
    );
}

/// A FALSE guard leaves the variable unconstrained — the dist must not apply.
#[test]
fn a_false_guard_disables_the_dist() {
    let src = r#"
module top;
  int zero_seen;
  initial begin
    bit [3:0] kind;
    bit en;
    int i, st;
    en = 0;
    for (i = 0; i < 200; i++) begin
      st = std::randomize(kind) with { en -> kind dist { 1 := 60, [2:15] :/ 20 }; };
      if (kind == 0) zero_seen++;
    end
  end
endmodule
"#;
    let sim = simulate(src, 20).expect("simulate failed");
    assert!(
        u(&sim, "zero_seen") > 0,
        "with en=0 the full 4-bit range is legal"
    );
}

/// §18.5.9: `list.size() == N` sizes a dynamic-array target.
#[test]
fn size_constraint_sizes_a_dynamic_array_target() {
    let src = r#"
module top;
  int st, sz;
  initial begin
    int unsigned list[];
    st = std::randomize(list) with { list.size() == 4; };
    sz = list.size();
  end
endmodule
"#;
    let sim = simulate(src, 20).expect("simulate failed");
    assert_eq!(u(&sim, "st"), 1);
    assert_eq!(u(&sim, "sz"), 4, "the size constraint must size the array");
}

/// foreach over a dynamic array, with an index-dependent if/else body.
#[test]
fn foreach_over_a_dynamic_array_with_index_conditions() {
    let src = r#"
module top;
  int st, ok_range, ok_index;
  initial begin
    int unsigned list[];
    int i;
    list = new[7];
    st = std::randomize(list) with {
      foreach (list[i]) {
        if (i >= 6) { list[i] == 3; }
        else        { list[i] inside { [0:7] }; }
      }
    };
    ok_range = 1; ok_index = 1;
    for (i = 0; i < 7; i++) begin
      if (i >= 6) begin if (list[i] != 3) ok_index = 0; end
      else if (list[i] > 7) ok_range = 0;
    end
  end
endmodule
"#;
    let sim = simulate(src, 20).expect("simulate failed");
    assert_eq!(u(&sim, "st"), 1);
    assert_eq!(u(&sim, "ok_range"), 1, "else-branch inside range applies");
    assert_eq!(u(&sim, "ok_index"), 1, "if(i>=6) equality applies");
}

/// foreach over a PACKED vector's bits guarding per-bit constraints on
/// another target: bits outside `possible` must be 0, every call.
#[test]
fn foreach_over_packed_bits_with_per_bit_dist() {
    let src = r#"
module top;
  int st_all, illegal;
  initial begin
    bit [3:0] pattern;
    bit [3:0] possible;
    int n, st;
    possible = 4'b1011;
    st_all = 1;
    for (n = 0; n < 40; n++) begin
      st = std::randomize(pattern) with {
        foreach (possible[i]) {
          if (possible[i]) { pattern[i] dist { 0 := 98, 1 := 2 }; }
          else             { pattern[i] == 0; }
        }
      };
      if (st != 1) st_all = 0;
      if ((pattern & ~possible) != 0) illegal++;
    end
  end
endmodule
"#;
    let sim = simulate(src, 20).expect("simulate failed");
    assert_eq!(u(&sim, "st_all"), 1);
    assert_eq!(u(&sim, "illegal"), 0, "bits outside `possible` stay 0");
}

/// The guards: a PLAIN dist keeps its bias, and a fixed-size foreach still
/// solves — neither may be disturbed by the new arms.
#[test]
fn plain_dist_and_fixed_foreach_are_unchanged() {
    let src = r#"
module top;
  int c1, chi, ok_fixed, st;
  initial begin
    bit [3:0] kind;
    int arr[4];
    int i;
    for (i = 0; i < 200; i++) begin
      st = std::randomize(kind) with { kind dist { 1 := 60, [2:15] :/ 20 }; };
      if (kind == 1) c1++; else chi++;
    end
    st = std::randomize(arr) with { foreach (arr[i]) arr[i] == i + 5; };
    ok_fixed = 1;
    for (i = 0; i < 4; i++) if (arr[i] != i + 5) ok_fixed = 0;
  end
endmodule
"#;
    let sim = simulate(src, 20).expect("simulate failed");
    assert!(u(&sim, "c1") > u(&sim, "chi"), "plain dist still biased");
    assert_eq!(u(&sim, "ok_fixed"), 1, "fixed-array foreach still pins");
}

/// The CLASS-constraint path handles the same per-bit foreach: a constraint
/// block iterating a packed state vector's bits and guarding a rand
/// property's bit-selects. This is a separate solver from the inline path —
/// rand COLLECTIONS own class foreach handling, so a packed vector fell
/// through every arm and `constraint_unmodeled` excused it from the
/// satisfaction check.
#[test]
fn class_constraint_foreach_over_packed_bits() {
    let src = r#"
module top;
  class C;
    rand bit [3:0] pattern;
    bit [3:0] possible;
    constraint c_bits {
      foreach (possible[i]) {
        if (possible[i]) { pattern[i] dist { 0 := 98, 1 := 2 }; }
        else             { pattern[i] == 0; }
      }
    }
  endclass
  int st_all, illegal, n;
  initial begin
    C c;
    c = new();
    c.possible = 4'b1011;
    st_all = 1;
    for (n = 0; n < 40; n++) begin
      if (c.randomize() != 1) st_all = 0;
      if ((c.pattern & ~c.possible) != 0) illegal++;
    end
  end
endmodule
"#;
    let sim = simulate(src, 20).expect("simulate failed");
    assert_eq!(u(&sim, "st_all"), 1);
    assert_eq!(
        u(&sim, "illegal"),
        0,
        "bits outside `possible` stay 0 in the class path"
    );
}
