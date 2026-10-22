//! Two ivtest-derived fixes, both reference-validated:
//!
//! 1. §13.4.3: a non-constant default argument (`function k(int i, int j = a+b)`)
//!    is evaluated at CALL time, so the reads inside the default expression are
//!    part of the CALLER's sensitivity. A continuous assign `wire x = k(1)`
//!    never re-evaluated after `a`/`b` changed because the dependency collector
//!    only walked the function BODY, not the port defaults.
//!
//! 2. §7.11 / §20.7: `$left/$right/$size/$low/$high` on a STRING variable track
//!    the string's CURRENT length ($left 0, $right len-1, $size len) — the
//!    operand fell through to the packed-vector branch and reported the
//!    1024-bit string placeholder width (L=1023, S=1024).

use xezim::simulate;

fn u(sim: &xezim::compiler::Simulator, n: &str) -> u64 {
    sim.get_signal(n)
        .or_else(|| sim.get_signal(&format!("tb.{}", n)))
        .unwrap_or_else(|| panic!("signal not found: {}", n))
        .to_u64()
        .unwrap_or_else(|| panic!("{} not u64-able (x/z?)", n))
}

/// A CA calling a function with a non-constant default argument must re-evaluate
/// when the default's operands change.
#[test]
fn default_argument_reads_drive_ca_sensitivity() {
    let src = r#"
module tb;
  integer a, b;
  function integer k(integer i, integer j = a + b);
    k = i + j;
  endfunction
  wire [31:0] x = k(1);
  wire [31:0] y = k(2);
  int rx, ry, direct, explicit;
  initial begin
    a = 1; b = 2;
    #1;
    rx = x;               // 1 + (1+2) = 4
    ry = y;               // 2 + (1+2) = 5
    direct = k(3);        // 3 + 3 = 6
    explicit = k(3, 4);   // 3 + 4 = 7
    a = 10;               // default now reads 12
    #1;
    rx = rx + x;          // 4 + (1+12) = 17
  end
endmodule
"#;
    let sim = simulate(src, 50).expect("simulate failed");
    assert_eq!(
        u(&sim, "ry"),
        5,
        "CA with default arg stale after operand init"
    );
    assert_eq!(u(&sim, "direct"), 6);
    assert_eq!(
        u(&sim, "explicit"),
        7,
        "explicit arg must override the default"
    );
    assert_eq!(
        u(&sim, "rx"),
        17,
        "CA must re-fire when a default-arg operand changes"
    );
}

/// A side-effecting queue method inside a COMPOUND rvalue must run exactly
/// once: `s = s - q.pop_back()` popped TWICE (once in the width probe, once
/// for the value), draining the queue and subtracting the wrong element.
#[test]
fn queue_pop_in_compound_rvalue_runs_once() {
    let src = r#"
module tb;
  reg [31:0] q[$];
  reg [31:0] s32;
  reg [36:0] s37;   // wider than the element: forces the width probe
  int si, sz1, sz2, sz3;
  initial begin
    q.push_front(10); q.push_front(20); q.push_front(30); q.push_front(40);
    s32 = 100;
    s32 = s32 - q.pop_back();   // pops 10
    sz1 = q.size();
    si = 100;
    si = si - q.pop_back();     // pops 20
    sz2 = q.size();
    s37 = 100;
    s37 = s37 - q.pop_back();   // pops 30
    sz3 = q.size();
  end
endmodule
"#;
    let sim = simulate(src, 50).expect("simulate failed");
    assert_eq!(u(&sim, "s32"), 90, "pop_back must pop exactly once");
    assert_eq!(u(&sim, "sz1"), 3);
    assert_eq!(u(&sim, "si"), 80);
    assert_eq!(u(&sim, "sz2"), 2);
    assert_eq!(
        u(&sim, "s37"),
        70,
        "wide LHS width-probe must not evaluate the pop"
    );
    assert_eq!(u(&sim, "sz3"), 1);
}

/// Array-query system functions on string variables report the current length.
#[test]
fn array_queries_on_string_variables() {
    let src = r#"
module tb;
  int l, r, s, lo, hi, el, neg, dneg;
  int d[];
  initial begin
    string m = "13 characters";
    string e = "";
    l = $left(m); r = $right(m); s = $size(m);
    lo = $low(m); hi = $high(m);
    el = $left(e);
    neg = ($right(e) == -1);   // empty string: -1, signed, not clamped
    dneg = ($high(d) == -1);   // empty dynamic array likewise
  end
endmodule
"#;
    let sim = simulate(src, 50).expect("simulate failed");
    assert_eq!(u(&sim, "l"), 0, "$left(string) is 0");
    assert_eq!(u(&sim, "r"), 12, "$right(string) is len-1");
    assert_eq!(u(&sim, "s"), 13, "$size(string) is len");
    assert_eq!(u(&sim, "lo"), 0);
    assert_eq!(u(&sim, "hi"), 12);
    assert_eq!(u(&sim, "el"), 0, "empty string: $left 0");
    assert_eq!(u(&sim, "neg"), 1, "empty string: $right is -1 (signed int)");
    assert_eq!(u(&sim, "dneg"), 1, "empty dynamic array: $high is -1");
}
