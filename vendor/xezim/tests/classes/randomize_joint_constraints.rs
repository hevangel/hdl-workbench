//! §18 — constraint mechanisms cross-checked against the reference
//! simulator. Assertions are seed-independent (counts and invariants, not
//! values), so they hold for any conforming solver.
//!
//! The joint-solve tests are feasible (or provably infeasible) problems in
//! which arithmetic (`sum()`), `unique` and orderings couple many variables
//! at once; per-variable propagation cannot satisfy them, so they exercise
//! the joint solver. Every expectation was cross-checked against the
//! reference simulator.

use xezim::simulate;

fn out(src: &str) -> Vec<String> {
    let sim = simulate(src, 1000).expect("simulate failed");
    sim.output.iter().map(|o| o.message.clone()).collect()
}

fn field(lines: &[String], key: &str) -> i64 {
    let line = lines
        .iter()
        .find(|l| l.contains(key))
        .unwrap_or_else(|| panic!("missing {key}: {lines:?}"));
    let rest = &line[line.find(key).unwrap() + key.len()..];
    rest.split(|c: char| !c.is_ascii_digit())
        .next()
        .unwrap()
        .parse()
        .unwrap()
}

#[test]
fn supported_constraint_mechanisms_hold() {
    let o = out(r#"
class mix;
  rand bit [3:0] x; rand bit [3:0] y; randc bit [2:0] rc; rand bit en;
  constraint c_sb { solve en before x; }
  constraint c_imp { en -> x == 0; }
  constraint c_soft { soft y == 7; }
  constraint c_dist { en dist {1 := 1, 0 := 9}; }
endclass
module tb;
  initial begin
    automatic mix m = new;
    automatic int en1 = 0, soft7 = 0, bad = 0, r;
    automatic int rc_hist[8];
    for (int i = 0; i < 800; i++) begin
      void'(m.randomize());
      if (m.en) begin en1++; if (m.x != 0) bad++; end
      if (m.y == 7) soft7++;
      rc_hist[m.rc]++;
    end
    $display("EN1=%0d SOFT7=%0d BAD=%0d", en1, soft7, bad);
    foreach (rc_hist[i]) if (rc_hist[i] != 100) $display("RANDC_UNEVEN %0d=%0d", i, rc_hist[i]);
    r = m.randomize() with { y == 3; x > 5; };
    $display("INLINE R=%0d Y=%0d XGT5=%0d", r, m.y, m.x > 5);
    r = m.randomize() with { x > 20; };
    $display("INFEASIBLE R=%0d", r);
  end
endmodule
"#);
    // `dist` 1:9 over 800 draws: mean 80, sd ~8.5.
    let en1 = field(&o, "EN1=");
    assert!((40..=130).contains(&en1), "dist weighting off: {o:?}");
    assert_eq!(
        field(&o, "SOFT7="),
        800,
        "an unopposed soft constraint must hold: {o:?}"
    );
    assert_eq!(field(&o, "BAD="), 0, "implication violated: {o:?}");
    assert!(
        !o.iter().any(|l| l.contains("RANDC_UNEVEN")),
        "randc must cycle through all 8 values every 8 draws: {o:?}"
    );
    assert!(
        o.iter().any(|l| l.contains("INLINE R=1 Y=3 XGT5=1")),
        "{o:?}"
    );
    assert!(o.iter().any(|l| l.contains("INFEASIBLE R=0")), "{o:?}");
}

#[test]
fn arithmetic_unique_and_ordering_solve_jointly() {
    let o = out(r#"
class hard;
  rand bit [7:0] a[12];
  constraint c1 { foreach (a[i]) a[i] inside {[1:30]}; }
  constraint c2 { a.sum() with (int'(item)) == 200; }
  constraint c3 { unique {a}; }
  constraint c4 { foreach (a[i]) if (i > 0) a[i] > a[i-1]; }
endclass
module tb;
  initial begin
    automatic hard h = new;
    automatic int ok = 0, valid = 0;
    for (int i = 0; i < 50; i++) begin
      if (h.randomize()) begin
        automatic int s = 0; automatic bit good = 1;
        ok++;
        foreach (h.a[j]) s += h.a[j];
        for (int j = 1; j < 12; j++) if (h.a[j] <= h.a[j-1]) good = 0;
        if (s == 200 && good) valid++;
      end
    end
    $display("OK=%0d VALID=%0d", ok, valid);
  end
endmodule
"#);
    assert!(o.iter().any(|l| l.contains("OK=50 VALID=50")), "{o:?}");
}

#[test]
fn weighted_sum_with_item_index_solves() {
    // §7.12.4: `item.index` inside a `with` clause is the element's index.
    let o = out(r#"
class wsum;
  rand bit [7:0] a[6];
  constraint c1 { foreach (a[i]) a[i] inside {[0:15]}; }
  constraint c2 { a.sum() with (int'(item) * (item.index + 1)) == 100; }
  constraint c3 { a.sum() with (int'(item)) == 30; }
endclass
module tb;
  initial begin
    automatic wsum w = new;
    automatic int ok = 0, valid = 0, changed = 0, prev = -1;
    for (int i = 0; i < 100; i++) begin
      if (w.randomize()) begin
        automatic int s = 0, ws = 0; automatic bit good = 1;
        ok++;
        foreach (w.a[j]) begin s += w.a[j]; ws += w.a[j] * (j + 1); if (w.a[j] > 15) good = 0; end
        if (s == 30 && ws == 100 && good) valid++;
        if (w.a[0] != prev) changed++;
        prev = w.a[0];
      end
    end
    $display("OK=%0d VALID=%0d CHANGED=%0d", ok, valid, changed);
  end
endmodule
"#);
    assert!(o.iter().any(|l| l.contains("OK=100 VALID=100")), "{o:?}");
    assert!(
        field(&o, "CHANGED=") >= 50,
        "solutions must vary between calls: {o:?}"
    );
}

#[test]
fn orderings_between_two_arrays_solve_jointly() {
    let o = out(r#"
class win;
  rand bit [7:0] lo[6];
  rand bit [7:0] hi[6];
  constraint c1 { foreach (lo[i]) { lo[i] inside {[0:200]}; hi[i] inside {[0:200]}; hi[i] >= lo[i] + 20; } }
  constraint c2 { foreach (lo[i]) if (i > 0) lo[i] > hi[i-1]; }
  constraint c3 { lo.sum() with (int'(item)) + hi.sum() with (int'(item)) <= 1300; }
endclass
module tb;
  initial begin
    automatic win w = new;
    automatic int ok = 0, valid = 0;
    for (int i = 0; i < 100; i++) begin
      if (w.randomize()) begin
        automatic int s = 0; automatic bit good = 1;
        ok++;
        foreach (w.lo[j]) begin
          s += w.lo[j] + w.hi[j];
          if (w.lo[j] > 200 || w.hi[j] > 200 || w.hi[j] < w.lo[j] + 20) good = 0;
          if (j > 0 && w.lo[j] <= w.hi[j-1]) good = 0;
        end
        if (s <= 1300 && good) valid++;
      end
    end
    $display("OK=%0d VALID=%0d", ok, valid);
  end
endmodule
"#);
    assert!(o.iter().any(|l| l.contains("OK=100 VALID=100")), "{o:?}");
}

#[test]
fn unique_scalars_with_sum_cover_every_solution() {
    // Exactly 12 assignments satisfy this set (the permutations of
    // {0,1,2,3} with w > z); `w + x + y + z` is summed at 32 bits, not 4.
    let o = out(r#"
class us;
  rand bit [3:0] w, x, y, z;
  constraint c1 { unique {w, x, y, z}; }
  constraint c2 { w + x + y + z == 6; }
  constraint c3 { w > z; }
endclass
module tb;
  initial begin
    automatic us u = new;
    automatic int ok = 0, valid = 0;
    automatic int seen[int];
    for (int i = 0; i < 200; i++) begin
      if (u.randomize()) begin
        ok++;
        if (u.w + u.x + u.y + u.z == 6 && u.w != u.x && u.w != u.y && u.w != u.z &&
            u.x != u.y && u.x != u.z && u.y != u.z && u.w > u.z) valid++;
        seen[{u.w, u.x, u.y, u.z}] = 1;
      end
    end
    $display("OK=%0d VALID=%0d SOLUTIONS=%0d", ok, valid, seen.num());
  end
endmodule
"#);
    assert!(
        o.iter()
            .any(|l| l.contains("OK=200 VALID=200 SOLUTIONS=12")),
        "{o:?}"
    );
}

#[test]
fn dynamic_array_sum_with_random_size() {
    // §18.5.8.1: the size is solved first; the sum is feasible at every size.
    let o = out(r#"
class dq;
  rand bit [7:0] d[];
  rand bit [3:0] n;
  constraint c1 { d.size() == n; n inside {[3:8]}; }
  constraint c2 { foreach (d[i]) d[i] inside {[1:20]}; }
  constraint c3 { d.sum() with (int'(item)) == 40; }
  constraint c4 { unique {d}; }
  constraint c5 { foreach (d[i]) if (i > 0) d[i] > d[i-1]; }
endclass
module tb;
  initial begin
    automatic dq q = new;
    automatic int ok = 0, valid = 0;
    automatic int sizes[int];
    for (int i = 0; i < 100; i++) begin
      if (q.randomize()) begin
        automatic int s = 0; automatic bit good = 1;
        ok++;
        foreach (q.d[j]) begin
          s += q.d[j];
          if (q.d[j] < 1 || q.d[j] > 20) good = 0;
          if (j > 0 && q.d[j] <= q.d[j-1]) good = 0;
        end
        if (s == 40 && good && q.d.size() == q.n) valid++;
        sizes[q.n] = 1;
      end
    end
    $display("OK=%0d VALID=%0d SIZES=%0d", ok, valid, sizes.num());
  end
endmodule
"#);
    assert!(
        o.iter().any(|l| l.contains("OK=100 VALID=100 SIZES=6")),
        "{o:?}"
    );
}

#[test]
fn infeasible_sum_fails_and_keeps_values() {
    // 6 increasing values in [1:30] sum to 21..165; 165 has one solution.
    // A failed randomize() must leave the array as it was.
    let o = out(r#"
class hard;
  rand bit [7:0] a[6];
  rand int unsigned total;
  constraint c1 { foreach (a[i]) a[i] inside {[1:30]}; }
  constraint c2 { a.sum() with (int'(item)) == total; }
  constraint c3 { unique {a}; }
  constraint c4 { foreach (a[i]) if (i > 0) a[i] > a[i-1]; }
endclass
module tb;
  initial begin
    automatic hard h = new;
    automatic int r1 = 0, r2 = 0, r3 = 0, same = 1;
    automatic bit [7:0] snap[6];
    r3 = h.randomize() with { total == 165; };
    snap = h.a;
    r1 = h.randomize() with { total == 166; };
    r2 = h.randomize() with { total == 20; };
    foreach (snap[i]) if (snap[i] != h.a[i]) same = 0;
    $display("HI=%0d LO=%0d EDGE=%0d SAME=%0d A0=%0d A5=%0d", r1, r2, r3, same, h.a[0], h.a[5]);
  end
endmodule
"#);
    assert!(
        o.iter()
            .any(|l| l.contains("HI=0 LO=0 EDGE=1 SAME=1 A0=25 A5=30")),
        "{o:?}"
    );
}
