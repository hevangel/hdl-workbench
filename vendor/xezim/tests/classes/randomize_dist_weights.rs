//! §18.5.4 — `dist` weights. Each test draws a few thousand solutions and
//! checks the weight ratios with bounds of about four standard deviations;
//! every expectation was cross-checked against the reference simulator.

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

/// A dist per `if` branch on the same variable: each branch keeps its own
/// weights. The per-variable trials dealt one schedule per variable and
/// restarted it whenever the other branch's weights came up, and a fresh
/// schedule opens on its heaviest item, so both branches over-drew their
/// heavy value (0.82 instead of 0.75, 0.84 instead of 0.80). The `randc`
/// member keeps this class on the per-variable trials.
#[test]
fn dist_per_branch_keeps_its_weights() {
    let c = counts(
        r#"
class F4;
  randc bit [1:0] rc;
  rand bit m;
  rand bit [3:0] v;
  constraint c { if (m) v dist {0 := 3, 1 := 1}; else v dist {[8:11] :/ 1, 15 := 4}; }
endclass
module top;
  initial begin
    automatic F4 o = new;
    automatic int m1, v0, v1, v15, v8, other, f;
    for (int i = 0; i < 4000; i++) begin
      if (!o.randomize()) f++;
      if (o.m) begin m1++; if (o.v == 0) v0++; else if (o.v == 1) v1++; else other++; end
      else begin if (o.v == 15) v15++; else if (o.v == 8) v8++; else if (o.v < 8 || o.v > 11) other++; end
    end
    $display("fails=%0d other=%0d m1=%0d v0p=%0d v15p=%0d v8p=%0d", f, other, m1,
             v0 * 1000 / m1, v15 * 1000 / (4000 - m1), v8 * 1000 / (4000 - m1));
  end
endmodule
"#,
    );
    assert_eq!(get(&c, "fails"), 0, "{c:?}");
    assert_eq!(get(&c, "other"), 0, "{c:?}");
    // per mille: 750 (sd ~10), 800 (sd ~9), 50 (sd ~5)
    within(&c, "v0p", 710, 790);
    within(&c, "v15p", 765, 835);
    within(&c, "v8p", 30, 70);
}

/// A dist on a variable that other constraints also read. The variable is
/// decided first, drawing an ITEM by its weight among the items the other
/// constraints leave feasible (`:=` weighs every value of the item as
/// declared, `:/` the item as a whole, even when only part of it is
/// feasible), then a feasible value of that item. The weights were ignored
/// once the constraints coupled (every feasible value equally likely), or
/// skewed by the per-variable repair.
#[test]
fn coupled_dist_keeps_its_weights() {
    let c = counts(
        r#"
class C1; rand bit [3:0] x; constraint c { x dist {0 := 1, 1 := 1, 2 := 8}; x != 2; } endclass
class C2; rand bit [3:0] x; rand bit [3:0] y; constraint c { x dist {[0:3] :/ 1, [4:7] :/ 1}; y <= x; } endclass
class C5; rand bit [3:0] x; rand bit [3:0] y; constraint c { x dist {1 := 1, 9 := 3}; x + y == 10; } endclass
class P1; rand bit [7:0] x; constraint c { x dist {[0:9] := 1, [10:209] := 1}; x < 20; } endclass
class P2; rand bit [7:0] x; constraint c { x dist {[0:9] :/ 1, [10:209] :/ 1}; x < 20; } endclass
class P4; rand bit [7:0] x; constraint c { x dist {[0:9] := 1, 10 := 5}; x > 4; } endclass
module top;
  initial begin
    automatic C1 c1 = new; automatic C2 c2 = new; automatic C5 c5 = new;
    automatic P1 p1 = new; automatic P2 p2 = new; automatic P4 p4 = new;
    automatic int f, c1x0, c1bad, c2hi, c2x7, c2bad, c5x9, c5bad, p1lo, p2lo, p4x10;
    for (int i = 0; i < 4000; i++) begin
      if (!c1.randomize()) f++; c1x0 += (c1.x == 0); c1bad += (c1.x > 1);
      if (!c2.randomize()) f++; c2hi += (c2.x >= 4); c2x7 += (c2.x == 7); c2bad += (c2.y > c2.x || c2.x > 7);
      if (!c5.randomize()) f++; c5x9 += (c5.x == 9); c5bad += (c5.x + c5.y != 10 || (c5.x != 1 && c5.x != 9));
      if (!p1.randomize()) f++; p1lo += (p1.x < 10);
      if (!p2.randomize()) f++; p2lo += (p2.x < 10);
      if (!p4.randomize()) f++; p4x10 += (p4.x == 10);
    end
    $display("fails=%0d c1bad=%0d c2bad=%0d c5bad=%0d", f, c1bad, c2bad, c5bad);
    $display("c1x0=%0d c2hi=%0d c2x7=%0d c5x9=%0d p1lo=%0d p2lo=%0d p4x10=%0d",
             c1x0, c2hi, c2x7, c5x9, p1lo, p2lo, p4x10);
  end
endmodule
"#,
    );
    for k in ["fails", "c1bad", "c2bad", "c5bad"] {
        assert_eq!(get(&c, k), 0, "{k}: {c:?}");
    }
    // 2 drops out: 0 and 1 share the rest evenly (2000, sd ~32)
    within(&c, "c1x0", 1850, 2150);
    // x follows its own weights, not the count of y values it leaves
    within(&c, "c2hi", 1840, 2160);
    within(&c, "c2x7", 380, 640);
    // 1:3 (3000, sd ~27)
    within(&c, "c5x9", 2870, 3130);
    // `:=` items weigh 10 and 200 values although only 10 of the second
    // are feasible: 10/210 (190, sd ~13)
    within(&c, "p1lo", 130, 260);
    // `:/` items keep their whole weight: 1:1
    within(&c, "p2lo", 1850, 2150);
    // 5 of 15 (1333, sd ~30)
    within(&c, "p4x10", 1200, 1470);
}

/// The same weights when the joint solver handles the whole set: an array
/// constraint the per-variable trials cannot meet rides along. Zero-weight
/// items drop out without shifting the weights of the others.
#[test]
fn dist_weights_hold_in_the_joint_solver() {
    let c = counts(
        r#"
class J3;
  rand bit [3:0] x;
  rand bit [3:0] y;
  rand bit [3:0] z;
  rand bit [7:0] a[5];
  constraint c { x dist {0 := 1, 1 := 1, 2 := 8}; x != 1; y dist {[0:3] :/ 1, 9 := 3}; }
  constraint d { z dist {0 := 0, 1 := 1, [2:3] :/ 0, 5 := 3}; }
  constraint h { foreach (a[i]) a[i] inside {[1:20]}; unique {a}; a.sum() with (int'(item) * (item.index + 1)) == 150; }
endclass
module top;
  initial begin
    automatic J3 o = new;
    automatic int f, x0, y9, z5, bad;
    for (int i = 0; i < 2000; i++) begin
      if (!o.randomize()) f++;
      x0 += (o.x == 0); y9 += (o.y == 9); z5 += (o.z == 5);
      if (o.x != 0 && o.x != 2 || o.y > 3 && o.y != 9 || o.z != 1 && o.z != 5) bad++;
    end
    $display("fails=%0d bad=%0d x0=%0d y9=%0d z5=%0d", f, bad, x0, y9, z5);
  end
endmodule
"#,
    );
    assert_eq!(get(&c, "fails"), 0, "{c:?}");
    assert_eq!(get(&c, "bad"), 0, "{c:?}");
    // 1:8 (222, sd ~14), 3:1 (1500, sd ~19) and 1:3 (1500)
    within(&c, "x0", 160, 290);
    within(&c, "y9", 1420, 1580);
    within(&c, "z5", 1420, 1580);
}

/// A `:=` range item weighs every one of its values, however many: the
/// per-variable trials capped an item at 4096 values, so `[0:99999] := 1`
/// against `100000 := 100` drew the single value 2.4% of the time instead of
/// 0.1%.
#[test]
fn wide_range_item_weighs_every_value() {
    let c = counts(
        r#"
class S1; rand bit [31:0] x; constraint c { x dist {[0:99999] := 1, 100000 := 100}; } endclass
class S2; rand bit [31:0] x; constraint c { x dist {[0:99999] :/ 1, 100000 := 1}; } endclass
module top;
  initial begin
    automatic S1 s1 = new; automatic S2 s2 = new;
    automatic int f, c1, c2, bad;
    for (int i = 0; i < 4000; i++) begin
      if (!s1.randomize()) f++; c1 += (s1.x == 100000); bad += (s1.x > 100000);
      if (!s2.randomize()) f++; c2 += (s2.x == 100000); bad += (s2.x > 100000);
    end
    $display("fails=%0d bad=%0d c1=%0d c2=%0d", f, bad, c1, c2);
  end
endmodule
"#,
    );
    assert_eq!(get(&c, "fails"), 0, "{c:?}");
    assert_eq!(get(&c, "bad"), 0, "{c:?}");
    // 100/100100 (4, sd 2); `:/` keeps the range at one item: 1:1
    within(&c, "c1", 0, 15);
    within(&c, "c2", 1840, 2160);
}
