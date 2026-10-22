//! §19.6.1 cross bins: `bins`, `ignore_bins` and `illegal_bins` of a cross
//! body, selected with `binsof(cp)`, `binsof(cp.bin)`, `intersect`, `!`,
//! `&&`, `||` and `with`. Only `bins = binsof(cp) intersect {...}` used to
//! count, and `&&` / `||` kept the first term. A user bin is one bin holding
//! the products it selects, an ignore_bins or illegal_bins removes the
//! products it selects, and every other product stays an automatic bin.
//! Every expected number was cross-checked against the reference simulator.

use xezim::simulate;

fn run(src: &str) -> (Vec<String>, u64) {
    let sim = simulate(src, 100_000).expect("simulate failed");
    let lines = sim
        .output
        .iter()
        .map(|o| o.message.trim().to_string())
        .collect();
    (lines, sim.error_count)
}

fn assert_line(lines: &[String], want: &str) {
    assert!(
        lines.iter().any(|l| l == want),
        "expected `{want}`, got {lines:?}"
    );
}

/// 16 products less the 4 with `a == 3`: one of the 12 is hit. The illegal
/// product sampled is an error.
#[test]
fn cross_ignore_and_illegal_bins() {
    let (l, errors) = run(r#"
module tb;
  bit [1:0] a, b;
  covergroup cg_ib;
    cp_a : coverpoint a;
    cp_b : coverpoint b;
    axb : cross cp_a, cp_b {
      ignore_bins a3 = binsof(cp_a) intersect {3};
    }
  endgroup
  covergroup cg_il;
    cp_a : coverpoint a;
    cp_b : coverpoint b;
    axb : cross cp_a, cp_b {
      illegal_bins a3 = binsof(cp_a) intersect {3};
    }
  endgroup
  cg_ib c1 = new(); cg_il c2 = new();
  initial begin
    a = 0; b = 0; c1.sample(); c2.sample();
    a = 3; b = 0; c1.sample(); c2.sample();
    $display("cross ignore_bins : %0.2f", c1.axb.get_inst_coverage());
    $display("cross illegal_bins: %0.2f", c2.axb.get_inst_coverage());
  end
endmodule
"#);
    assert_line(&l, "cross ignore_bins : 8.33");
    assert_line(&l, "cross illegal_bins: 8.33");
    assert_eq!(errors, 1, "{l:?}");
    assert!(
        l.iter().any(|m| m.contains("Illegal cross bin")
            && m.contains("axb.a3")
            && m.contains("(3, 0)")),
        "{l:?}"
    );
}

/// `binsof(cp.bin)` and `&&`: `lo_any` holds (lo, *), `hi_b0` (hi, 0), and
/// the three (hi, 1..3) products stay automatic: 2 of 5 bins hit.
#[test]
fn cross_binsof_bin_and_conjunction() {
    let (l, _) = run(r#"
module tb;
  bit [1:0] a, b;
  covergroup cg_ib;
    cp_a : coverpoint a;
    cp_b : coverpoint b;
    axb : cross cp_a, cp_b {
      ignore_bins a3 = binsof(cp_a) intersect {3};
    }
  endgroup
  covergroup cg_bins;
    cp_a : coverpoint a;
    cp_b : coverpoint b;
    axb : cross cp_a, cp_b {
      bins a_lo = binsof(cp_a) intersect {[0:1]};
      bins a_hi = binsof(cp_a) intersect {[2:3]};
    }
  endgroup
  covergroup cg_dot;
    cp_a : coverpoint a { bins lo = {[0:1]}; bins hi = {[2:3]}; }
    cp_b : coverpoint b;
    axb : cross cp_a, cp_b {
      bins lo_any = binsof(cp_a.lo);
      bins hi_b0  = binsof(cp_a.hi) && binsof(cp_b) intersect {0};
    }
  endgroup
  cg_ib c1 = new(); cg_bins c2 = new(); cg_dot c3 = new();
  initial begin
    for (int i = 0; i < 4; i++) begin
      a = i; b = 0; c1.sample(); c2.sample(); c3.sample();
    end
    $display("ignore_bins cross : %0.2f", c1.axb.get_inst_coverage());
    $display("intersect bins    : %0.2f", c2.axb.get_inst_coverage());
    $display("binsof(cp.bin)    : %0.2f", c3.axb.get_inst_coverage());
  end
endmodule
"#);
    assert_line(&l, "ignore_bins cross : 25.00");
    assert_line(&l, "intersect bins    : 100.00");
    assert_line(&l, "binsof(cp.bin)    : 40.00");
}

/// `&&` of two `binsof ... intersect` terms and a `with` filter.
#[test]
fn cross_and_and_with() {
    let (l, _) = run(r#"
module tb;
  bit [1:0] a, b;
  covergroup cg_and;
    cp_a : coverpoint a;
    cp_b : coverpoint b;
    axb : cross cp_a, cp_b {
      bins both0 = binsof(cp_a) intersect {0} && binsof(cp_b) intersect {0};
    }
  endgroup
  covergroup cg_with;
    cp_a : coverpoint a;
    cp_b : coverpoint b;
    axb : cross cp_a, cp_b {
      bins eq = binsof(cp_a) with (cp_a == 0);
    }
  endgroup
  cg_and c1 = new(); cg_with c2 = new();
  initial begin
    a = 0; b = 0; c1.sample(); c2.sample();
    $display("&& %0.2f with %0.2f", c1.axb.get_inst_coverage(), c2.axb.get_inst_coverage());
  end
endmodule
"#);
    assert_line(&l, "&& 6.25 with 7.69");
}

/// `!`, `||`, an empty user bin (not counted), a `with` over multi-value
/// bins, and ignore / user / illegal bins together, with the bin counts.
#[test]
fn cross_negation_disjunction_and_counts() {
    let (l, errors) = run(r#"
module top;
  bit [1:0] a, b;
  covergroup cg1;
    cp_a : coverpoint a; cp_b : coverpoint b;
    axb : cross cp_a, cp_b {
      bins x = !binsof(cp_a) intersect {0};
      bins y = binsof(cp_a) intersect {0} || binsof(cp_b) intersect {3};
    }
  endgroup
  covergroup cg2;
    cp_a : coverpoint a { bins lo = {0}; bins mid = {1}; ignore_bins hi = {[2:3]}; }
    cp_b : coverpoint b;
    axb : cross cp_a, cp_b {
      bins none = binsof(cp_a) intersect {3};
      bins z = binsof(cp_a) intersect {0};
    }
  endgroup
  covergroup cg3;
    cp_a : coverpoint a { bins lo = {[0:1]}; bins hi = {[2:3]}; }
    cp_b : coverpoint b { bins lo = {[0:1]}; bins hi = {[2:3]}; }
    axb : cross cp_a, cp_b { bins w = binsof(cp_a) with (cp_a + cp_b == 3); }
  endgroup
  covergroup cg4;
    cp_a : coverpoint a; cp_b : coverpoint b;
    axb : cross cp_a, cp_b {
      ignore_bins ig = binsof(cp_a) intersect {1};
      bins b0 = binsof(cp_a) intersect {[0:1]};
      illegal_bins bad = binsof(cp_a) intersect {3} && binsof(cp_b) intersect {3};
    }
  endgroup
  cg1 c1 = new(); cg2 c2 = new(); cg3 c3 = new(); cg4 c4 = new();
  int cov, tot;
  initial begin
    a = 0; b = 0; c1.sample(); c2.sample(); c3.sample(); c4.sample();
    a = 1; b = 2; c1.sample(); c2.sample(); c3.sample(); c4.sample();
    a = 2; b = 3; c1.sample(); c2.sample(); c3.sample(); c4.sample();
    a = 3; b = 3; c4.sample();
    $display("cg1 %0.2f cg2 %0.2f cg3 %0.2f cg4 %0.2f", c1.axb.get_inst_coverage(), c2.axb.get_inst_coverage(), c3.axb.get_inst_coverage(), c4.axb.get_inst_coverage());
    void'(c1.axb.get_inst_coverage(cov, tot)); $display("x1 %0d/%0d", cov, tot);
    void'(c2.axb.get_inst_coverage(cov, tot)); $display("x2 %0d/%0d", cov, tot);
    void'(c3.axb.get_inst_coverage(cov, tot)); $display("x3 %0d/%0d", cov, tot);
    void'(c4.axb.get_inst_coverage(cov, tot)); $display("x4 %0d/%0d", cov, tot);
    $display("grp %0.2f %0.2f %0.2f %0.2f", c1.get_inst_coverage(), c2.get_inst_coverage(), c3.get_inst_coverage(), c4.get_inst_coverage());
  end
endmodule
"#);
    assert_line(&l, "cg1 100.00 cg2 40.00 cg3 100.00 cg4 25.00");
    assert_line(&l, "x1 2/2");
    assert_line(&l, "x2 2/5");
    assert_line(&l, "x3 3/3");
    assert_line(&l, "x4 2/8");
    assert_line(&l, "grp 83.33 71.67 100.00 66.67");
    assert_eq!(errors, 1, "{l:?}");
}
