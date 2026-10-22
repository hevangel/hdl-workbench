//! §19.8 coverage queries and sampling control: the type-scoped
//! `cg_type::get_coverage()` and `cg_type::cp::get_coverage()` (they used
//! to return 0), the `(covered, total)` output arguments (never written),
//! `start()` / `stop()` (no effect), and `get_inst_coverage()` under
//! `merge_instances`. Every expected number
//! was cross-checked against the reference simulator.

use xezim::simulate;

fn lines(src: &str) -> Vec<String> {
    let sim = simulate(src, 100_000).expect("simulate failed");
    sim.output
        .iter()
        .map(|o| o.message.trim().to_string())
        .collect()
}

fn assert_line(lines: &[String], want: &str) {
    assert!(
        lines.iter().any(|l| l == want),
        "expected `{want}`, got {lines:?}"
    );
}

#[test]
fn type_scoped_queries() {
    let l = lines(
        r#"
module tb;
  bit [1:0] x;
  covergroup cg;
    cp : coverpoint x { bins b0 = {0}; bins b1 = {1}; }
  endgroup
  cg c = new();
  initial begin
    x = 0; c.sample();
    $display("inst=%0.2f type=%0.2f static=%0.2f", c.get_inst_coverage(), c.get_coverage(), cg::get_coverage());
    $display("static cp=%0.2f", cg::cp::get_coverage());
  end
endmodule
"#,
    );
    assert_line(&l, "inst=50.00 type=50.00 static=50.00");
    assert_line(&l, "static cp=50.00");
}

#[test]
fn stop_and_start() {
    let l = lines(
        r#"
module tb;
  bit [1:0] x;
  covergroup cg;
    cp : coverpoint x { bins b0 = {0}; bins b1 = {1}; }
  endgroup
  cg c = new();
  initial begin
    x = 0; c.sample();
    c.stop();
    x = 1; c.sample();
    $display("after stop: %0.2f", c.get_inst_coverage());
    c.start();
    c.sample();
    $display("after start: %0.2f", c.get_inst_coverage());
  end
endmodule
"#,
    );
    assert_line(&l, "after stop: 50.00");
    assert_line(&l, "after start: 100.00");
}

/// `covered` / `total` are summed over the coverpoints and crosses (the
/// cross over an uncovered variable and that variable's implicit
/// coverpoint included); a coverpoint stopped on its own.
#[test]
fn counts_and_item_stop() {
    let l = lines(
        r#"
module top;
  bit [1:0] a, b, x, y;
  covergroup cg_var;
    cp_a : coverpoint a { bins a0 = {0}; bins a1 = {1}; }
    axb : cross cp_a, b;
  endgroup
  covergroup cg;
    option.goal = 90;
    cpx : coverpoint x { bins b0 = {0}; bins b1 = {1}; option.weight = 2; }
    cpy : coverpoint y;
  endgroup
  covergroup cg2; cpx : coverpoint x; endgroup
  cg_var v = new();
  cg c = new();
  cg2 d = new();
  int cov, tot;
  real r;
  initial begin
    a = 0; b = 0; v.sample();
    a = 1; b = 0; v.sample();
    r = v.get_coverage(cov, tot);
    $display("get_coverage(ref) r=%0.2f cov=%0d tot=%0d", r, cov, tot);
    x = 0; y = 0; c.sample(); d.sample();
    c.cpy.stop();
    y = 1; c.sample();
    c.cpy.start();
    d.stop(); x = 1; d.sample(); d.start(); x = 2; d.sample();
    void'(c.get_inst_coverage(cov, tot)); $display("cg %0d/%0d %0.2f", cov, tot, c.get_inst_coverage());
    void'(c.cpy.get_inst_coverage(cov, tot)); $display("cpy %0d/%0d", cov, tot);
    $display("d %0.2f static %0.2f %0.2f", d.get_inst_coverage(), cg2::get_coverage(), cg::cpx::get_coverage());
  end
endmodule
"#,
    );
    assert_line(&l, "get_coverage(ref) r=50.00 cov=5 tot=14");
    assert_line(&l, "cg 2/6 41.67");
    assert_line(&l, "cpy 1/4");
    assert_line(&l, "d 50.00 static 50.00 50.00");
}

/// Under `merge_instances` an instance reports its type's coverage, and
/// without it a type's counts are the instances' summed.
#[test]
fn merge_instances_and_type_counts() {
    let l = lines(
        r#"
module top;
  bit [1:0] x;
  covergroup cg00; cp: coverpoint x { bins b0={0}; bins b1={1}; bins b2={2}; bins b3={3}; } endgroup
  covergroup cg01; type_option.merge_instances = 1; cp: coverpoint x { bins b0={0}; bins b1={1}; bins b2={2}; bins b3={3}; } endgroup
  cg00 a00 = new(), b00 = new();
  cg01 a01 = new(), b01 = new();
  int c, t;
  initial begin
    x = 0; a00.sample(); a01.sample();
    x = 1; a00.sample(); a01.sample();
    x = 1; b00.sample(); b01.sample();
    x = 2; b00.sample(); b01.sample();
    $display("00 a %0.2f b %0.2f t %0.2f", a00.get_inst_coverage(), b00.get_inst_coverage(), a00.get_coverage());
    $display("01 a %0.2f b %0.2f t %0.2f cp %0.2f", a01.get_inst_coverage(), b01.get_inst_coverage(), a01.get_coverage(), a01.cp.get_inst_coverage());
    void'(a01.get_inst_coverage(c, t)); $display("01 inst counts %0d/%0d", c, t);
    void'(a00.get_coverage(c, t)); $display("00 type counts %0d/%0d", c, t);
    $display("all %0.2f", $get_coverage());
  end
endmodule
"#,
    );
    assert_line(&l, "00 a 50.00 b 50.00 t 50.00");
    assert_line(&l, "01 a 75.00 b 75.00 t 75.00 cp 75.00");
    assert_line(&l, "01 inst counts 3/4");
    assert_line(&l, "00 type counts 4/8");
    assert_line(&l, "all 62.50");
}
