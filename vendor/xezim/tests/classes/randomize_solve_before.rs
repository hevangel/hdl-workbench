//! §18.5.10 — variable ordering. Without `solve … before` every solution
//! is equally likely, so in `s -> d == 0` (`s` 1 bit, `d` 32 bits) `s` is 1
//! for one solution in 2^32 + 1; with `solve s before d`, `s` is drawn
//! first, uniformly over its feasible values, and is 1 half the time. Each
//! test draws a few thousand solutions; bounds are about four standard
//! deviations, and every expectation was cross-checked against the
//! reference simulator.

use xezim::simulate;

fn counts(src: &str) -> Vec<(String, i64)> {
    let sim = simulate(src, 100).expect("simulate failed");
    let mut out = Vec::new();
    for o in &sim.output {
        for tok in o.message.split_whitespace() {
            if let Some((k, v)) = tok.split_once('=') {
                if let Ok(n) = v.parse() {
                    out.push((k.to_string(), n));
                }
            }
        }
    }
    out
}

fn get(c: &[(String, i64)], k: &str) -> i64 {
    c.iter()
        .find(|(n, _)| n == k)
        .unwrap_or_else(|| panic!("missing {k}: {c:?}"))
        .1
}

fn within(c: &[(String, i64)], k: &str, lo: i64, hi: i64) {
    let v = get(c, k);
    assert!(
        (lo..=hi).contains(&v),
        "{k}={v} outside [{lo}, {hi}]: {c:?}"
    );
}

/// Ordered sets go to the joint solver, which decides the ordered variable
/// first. The ordering was ignored there, and the per-variable trials
/// repaired `x < y` by moving `y`, which left `y` skewed high and `x` flat.
#[test]
fn ordered_variable_is_drawn_first() {
    let c = counts(
        r#"
class B;  rand bit s; rand bit [31:0] d; constraint c { s -> d == 0; solve s before d; } endclass
class Ff; rand bit [1:0] a; rand bit [7:0] b;
  constraint c { (a == 0) -> b == 0; (a == 1) -> b < 2; solve a before b; } endclass
class Gg; rand bit [3:0] x; rand bit [3:0] y; constraint c { x < y; solve y before x; } endclass
class Dd; rand bit s; rand bit [31:0] d; constraint c { s dist {0 := 1, 1 := 3}; s -> d == 0; solve s before d; } endclass
module top;
  initial begin
    automatic B b = new; automatic Ff f = new; automatic Gg g = new; automatic Dd d = new;
    automatic int fails, bad, bs, fa[4], gy[16], gx0, gx14, ds;
    for (int i = 0; i < 4000; i++) begin
      if (!b.randomize()) fails++; bs += b.s; if (b.s && b.d != 0) bad++;
      if (!f.randomize()) fails++; fa[f.a]++;
      if (!g.randomize()) fails++; gy[g.y]++; gx0 += (g.x == 0); gx14 += (g.x == 14); if (g.x >= g.y) bad++;
      if (!d.randomize()) fails++; ds += d.s;
    end
    $display("fails=%0d bad=%0d bs=%0d ds=%0d", fails, bad, bs, ds);
    $display("fa0=%0d fa1=%0d fa2=%0d fa3=%0d", fa[0], fa[1], fa[2], fa[3]);
    $display("gy0=%0d gy1=%0d gy8=%0d gy15=%0d gx0=%0d gx14=%0d", gy[0], gy[1], gy[8], gy[15], gx0, gx14);
  end
endmodule
"#,
    );
    assert_eq!(get(&c, "fails"), 0, "{c:?}");
    assert_eq!(get(&c, "bad"), 0, "{c:?}");
    // 1/2 (2000, sd ~32)
    within(&c, "bs", 1840, 2160);
    // the dist on the ordered variable: 3/4 (3000, sd ~27)
    within(&c, "ds", 2870, 3130);
    // a uniform over 0..3 (1000, sd ~27), not weighted by the b values left
    for k in ["fa0", "fa1", "fa2", "fa3"] {
        within(&c, k, 880, 1120);
    }
    // y uniform over its feasible 1..15 (267, sd ~16), then x uniform below
    // it: P(x=0) = H(15)/15 (885, sd ~26), P(x=14) = 1/225 (18, sd ~4)
    assert_eq!(get(&c, "gy0"), 0, "{c:?}");
    for k in ["gy1", "gy8", "gy15"] {
        within(&c, k, 190, 345);
    }
    within(&c, "gx0", 780, 990);
    within(&c, "gx14", 3, 40);
}

/// The joint solver without an ordering: `s` is weighed by the solutions
/// each value leaves, so it (almost) never comes up 1; with the ordering it
/// is 1 half the time. The array constraint keeps the set in the joint
/// solver, which drew `s` uniformly either way.
#[test]
fn joint_solver_weighs_an_unordered_antecedent() {
    let c = counts(
        r#"
class J1;
  rand bit s; rand bit [31:0] d; rand bit [7:0] a[5];
  constraint c { s -> d == 0; }
  constraint h { foreach (a[i]) a[i] inside {[1:20]}; unique {a}; a.sum() with (int'(item) * (item.index + 1)) == 150; }
endclass
class J2;
  rand bit s; rand bit [31:0] d; rand bit [7:0] a[5];
  constraint c { s -> d == 0; solve s before d; }
  constraint h { foreach (a[i]) a[i] inside {[1:20]}; unique {a}; a.sum() with (int'(item) * (item.index + 1)) == 150; }
endclass
module top;
  initial begin
    automatic J1 j1 = new; automatic J2 j2 = new;
    automatic int fails, s1, s2;
    for (int i = 0; i < 2000; i++) begin
      if (!j1.randomize()) fails++; s1 += j1.s;
      if (!j2.randomize()) fails++; s2 += j2.s;
    end
    $display("fails=%0d s1=%0d s2=%0d", fails, s1, s2);
  end
endmodule
"#,
    );
    assert_eq!(get(&c, "fails"), 0, "{c:?}");
    assert!(get(&c, "s1") <= 1, "{c:?}");
    // 1/2 (1000, sd ~22)
    within(&c, "s2", 880, 1120);
}

/// The per-variable trials without an ordering: an antecedent with no range
/// of its own (`bit s`) was drawn uniformly and the consequent repaired, so
/// `s` came up 1 half the time. It is now weighed over its whole domain by
/// the solutions each value leaves. The `randc` member keeps F1 and F2 on
/// the trials; A has neither a dist nor an ordering, so it stays there too.
#[test]
fn trials_weigh_an_unranged_antecedent() {
    let c = counts(
        r#"
class A;  rand bit s; rand bit [31:0] d; constraint c { s -> d == 0; } endclass
class F1; randc bit [1:0] rc; rand bit s; rand bit [31:0] d; constraint c { s -> d == 0; } endclass
class F2; randc bit [1:0] rc; rand bit s; rand bit [31:0] d; constraint c { s -> d == 0; solve s before d; } endclass
module top;
  initial begin
    automatic A a = new; automatic F1 f1 = new; automatic F2 f2 = new;
    automatic int fails, bad, sa, s1, s2;
    for (int i = 0; i < 4000; i++) begin
      if (!a.randomize()) fails++; sa += a.s; if (a.s && a.d != 0) bad++;
      if (!f1.randomize()) fails++; s1 += f1.s;
      if (!f2.randomize()) fails++; s2 += f2.s; if (f2.s && f2.d != 0) bad++;
    end
    $display("fails=%0d bad=%0d sa=%0d s1=%0d s2=%0d", fails, bad, sa, s1, s2);
  end
endmodule
"#,
    );
    assert_eq!(get(&c, "fails"), 0, "{c:?}");
    assert_eq!(get(&c, "bad"), 0, "{c:?}");
    assert!(get(&c, "sa") <= 1, "{c:?}");
    assert!(get(&c, "s1") <= 1, "{c:?}");
    // the ordering keeps it 1/2 (2000, sd ~32)
    within(&c, "s2", 1840, 2160);
}
