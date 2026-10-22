//! §7.12 array locator and reduction methods — two defects found by an LRM
//! sweep against a reference simulator.
//!
//! 1. **A locator call used directly as an argument evaluated to 0.** The
//!    locator methods (`unique`, `find…`, `min`, `max`) RETURN a queue, and
//!    only the ASSIGNMENT path materialized that queue. Used inline the call
//!    fell through to scalar evaluation:
//!
//!    ```systemverilog
//!    r = q.unique();  $display("%p", r);   // '{5, 3, 9}  — right
//!    $display("%p", q.unique());           // 0           — wrong
//!    ```
//!
//!    Assigning first is the common style, which is why this stayed hidden.
//!
//! 2. **`sum with` / `product with` ignored the expression's width.** §7.12.3
//!    makes the result type the type of the `with` expression, so a 1-bit
//!    predicate accumulates modulo 2. `q.sum with (item > 4)` over six
//!    elements with three matches must be 1, not 3.
//!
//! NOT changed, deliberately: the ORDER of `unique()` / `unique_index()`.
//! xezim returns first-occurrence order; the reference simulator returns them
//! sorted by value. §7.12.1 does not mandate an order and the commonly
//! documented behaviour is source order, so matching the reference here would
//! mean adopting a sort the LRM does not require. Tests below therefore assert
//! on CONTENT, never on ordering.

use xezim::simulate;

fn u(sim: &xezim::compiler::Simulator, n: &str) -> u64 {
    sim.get_signal(n)
        .or_else(|| sim.get_signal(&format!("top.{}", n)))
        .unwrap_or_else(|| panic!("signal not found: {}", n))
        .to_u64()
        .unwrap_or_else(|| panic!("{} not u64-able (x/z?)", n))
}

fn outs(sim: &xezim::compiler::Simulator) -> Vec<String> {
    sim.output.iter().map(|o| o.message.clone()).collect()
}

/// A locator used inline must print its queue, and agree with the same call
/// assigned to a variable first.
#[test]
fn inline_locator_call_yields_its_queue() {
    let src = r#"
module top;
  int q[$] = '{5, 3, 9, 3};
  int r[$];
  initial begin
    r = q.unique();
    $display("ASSIGNED %p", r);
    $display("INLINE %p", q.unique());
    $display("FIND %p", q.find with (item > 4));
    $display("MIN %p", q.min());
    $display("MAX %p", q.max());
    $display("FIDX %p", q.find_index with (item == 3));
  end
endmodule
"#;
    let sim = simulate(src, 20).expect("simulate failed");
    let o = outs(&sim);
    let get = |tag: &str| {
        o.iter()
            .find(|s| s.starts_with(tag))
            .unwrap_or_else(|| panic!("missing {tag}: {o:?}"))
            .clone()
    };
    let assigned = get("ASSIGNED ");
    let inline = get("INLINE ");
    assert_eq!(
        assigned.trim_start_matches("ASSIGNED "),
        inline.trim_start_matches("INLINE "),
        "inline and assigned forms must agree"
    );
    assert!(
        inline.contains('{'),
        "inline locator prints a queue: {inline}"
    );
    assert_eq!(get("FIND "), "FIND '{5, 9}", "find with a filter");
    assert_eq!(get("MIN "), "MIN '{3}", "min returns a one-element queue");
    assert_eq!(get("MAX "), "MAX '{9}", "max likewise");
    assert_eq!(get("FIDX "), "FIDX '{1, 3}", "index form returns positions");
}

/// `unique()` keeps every distinct value exactly once — asserted on content
/// and size, since the ORDER is not specified.
#[test]
fn unique_returns_each_distinct_value_once() {
    let src = r#"
module top;
  int q[$] = '{5, 3, 9, 3, 1, 9};
  int r[$];
  int n, has1, has3, has5, has9, dup;
  initial begin
    r = q.unique();
    n = r.size();
    has1 = 0; has3 = 0; has5 = 0; has9 = 0;
    foreach (r[i]) begin
      if (r[i] == 1) has1++;
      if (r[i] == 3) has3++;
      if (r[i] == 5) has5++;
      if (r[i] == 9) has9++;
    end
    dup = (has1 > 1) || (has3 > 1) || (has5 > 1) || (has9 > 1);
  end
endmodule
"#;
    let sim = simulate(src, 20).expect("simulate failed");
    assert_eq!(u(&sim, "n"), 4, "four distinct values");
    assert_eq!(u(&sim, "has1"), 1);
    assert_eq!(u(&sim, "has3"), 1);
    assert_eq!(u(&sim, "has5"), 1);
    assert_eq!(u(&sim, "has9"), 1);
    assert_eq!(u(&sim, "dup"), 0, "no value appears twice");
}

/// §7.12.3: a `with` clause makes the result type the EXPRESSION's type, so a
/// 1-bit predicate accumulates modulo 2.
#[test]
fn sum_with_accumulates_in_the_expression_type() {
    let src = r#"
module top;
  int q[$] = '{5, 3, 9, 3, 1, 9};
  int one_bit, widened, casted, plain;
  initial begin
    one_bit = q.sum with (item > 4);                   // 1-bit -> 3 mod 2 = 1
    widened = q.sum with (item > 4 ? 32'd1 : 32'd0);   // 32-bit -> 3
    casted  = q.sum with (int'(item > 4));             // 32-bit -> 3
    plain   = q.sum();                                 // element type
  end
endmodule
"#;
    let sim = simulate(src, 20).expect("simulate failed");
    assert_eq!(u(&sim, "one_bit"), 1, "a 1-bit predicate wraps at 1 bit");
    assert_eq!(
        u(&sim, "widened"),
        3,
        "an explicitly 32-bit expression does not"
    );
    assert_eq!(u(&sim, "casted"), 3, "nor does an int' cast");
    assert_eq!(u(&sim, "plain"), 30, "no with clause: the element type");
}

/// The guard: reductions without a `with` clause, and the non-accumulating
/// reductions, are unchanged.
#[test]
fn other_reductions_are_unchanged() {
    let src = r#"
module top;
  int q[$] = '{5, 3, 9, 3, 1, 9};
  int s, p, mn, mx, a, o, x;
  initial begin
    s = q.sum(); p = q.product();
    mn = q.min()[0]; mx = q.max()[0];
    a = q.and(); o = q.or(); x = q.xor();
  end
endmodule
"#;
    let sim = simulate(src, 20).expect("simulate failed");
    assert_eq!(u(&sim, "s"), 30, "sum");
    assert_eq!(u(&sim, "p"), 3645, "product");
    assert_eq!(u(&sim, "mn"), 1, "min");
    assert_eq!(u(&sim, "mx"), 9, "max");
    assert_eq!(u(&sim, "a"), 1, "and");
    assert_eq!(u(&sim, "o"), 15, "or");
    assert_eq!(u(&sim, "x"), 4, "xor");
}

/// §7.12.4: `item.index` (or `<iterator>.index`) in a reduction's `with`
/// clause is the element's index; it read 0. Cross-checked against the
/// reference simulator.
#[test]
fn reduction_with_reads_item_index() {
    let src = r#"
module top;
  int a[4] = '{5, 6, 7, 8};
  int q[$] = {1, 2, 3};
  initial begin
    $display("S1=%0d S2=%0d S3=%0d", a.sum() with (item * item.index), q.sum(x) with (x * x.index), a.sum() with (item.index));
  end
endmodule
"#;
    let sim = simulate(src, 20).expect("simulate failed");
    let o = outs(&sim);
    assert!(o.iter().any(|l| l.contains("S1=44 S2=8 S3=6")), "{o:?}");
}
