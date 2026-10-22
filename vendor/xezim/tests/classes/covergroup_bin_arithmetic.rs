//! §19.11 — coverage computation over explicit, array, illegal and ignore
//! bins and a cross. The expected numbers are both the reference
//! simulator's and a hand count: `cp_op` has 4 bins (`low` + `high[4..6]`;
//! value 7 is illegal) and all are hit; `cp_mode` has 3 bins (value 3
//! ignored), all hit; the cross has 12 bins, 11 hit (`high[6] x 2` never
//! occurs) = 91.67%; the group is the mean of the three, 97.22%.
//!
//! xezim used to report 72.50 for the group and for every coverpoint and
//! the cross: ignore/illegal values stayed in array and automatic bins, a
//! cross counted raw value tuples instead of bin tuples, and a per-item
//! query returned the group's value.

use xezim::simulate;

#[test]
fn explicit_array_illegal_and_ignore_bins_with_cross() {
    let sim = simulate(
        r#"
module tb;
  logic clk = 0; always #5 clk = ~clk;
  logic [2:0] op; logic [1:0] mode;
  covergroup cg @(posedge clk);
    option.per_instance = 1;
    cp_op: coverpoint op { bins low = {[0:3]}; bins high[] = {[4:7]}; illegal_bins bad = {7}; }
    cp_mode: coverpoint mode { ignore_bins ig = {3}; }
    x: cross cp_op, cp_mode;
  endgroup
  cg c = new;
  initial begin
    op = 0; mode = 0;
    repeat (20) @(negedge clk) begin op = (op + 1) % 7; mode = (mode + 1) % 3; end
    $display("COV cg=%0.2f op=%0.2f mode=%0.2f x=%0.2f", c.get_coverage(), c.cp_op.get_coverage(),
             c.cp_mode.get_coverage(), c.x.get_coverage());
    $finish;
  end
endmodule
"#,
        1000,
    )
    .expect("simulate failed");
    let o: Vec<String> = sim.output.iter().map(|o| o.message.clone()).collect();
    assert!(
        o.iter()
            .any(|l| l.contains("COV cg=97.22 op=100.00 mode=100.00 x=91.67")),
        "{o:?}"
    );
}

/// Automatic bins above `auto_bin_max`, array bins minus `ignore_bins`, a
/// weighted coverpoint, a cross with a variable that has no coverpoint
/// (its implicit coverpoint counts in the group mean), and type coverage
/// of instances that bin differently (constructor arguments): with the
/// default `type_option.merge_instances = 0` it is the instances' average.
/// Every number is the reference simulator's.
#[test]
fn auto_weighted_implicit_and_per_instance_coverage() {
    let sim = simulate(
        r#"
module tb;
  logic clk = 0; always #5 clk = ~clk;
  logic [7:0] big; logic [3:0] a; logic [1:0] b; logic c;
  covergroup cg1 @(posedge clk);
    cp_big: coverpoint big;                                   // 256 values -> 64 auto bins
    cp_a: coverpoint a { bins lo[] = {[0:7]}; bins hi = {[8:15]}; ignore_bins ig = {3, 5}; }
    cp_b: coverpoint b { option.weight = 3; }
    xab: cross cp_a, cp_b;
    xac: cross cp_a, c;                                        // crosses a raw variable
  endgroup
  covergroup cg2 (int lo, int hi) @(posedge clk);
    cp: coverpoint a { bins r[] = {[lo:hi]}; illegal_bins bad = {15}; }
  endgroup
  cg1 g1 = new;
  cg2 g2a = new(0, 3), g2b = new(10, 15);
  initial begin
    big = 0; a = 0; b = 0; c = 0;
    for (int i = 0; i < 40; i++) begin
      @(negedge clk);
      big = i * 7; a = i % 13; b = (i / 3) % 3; c = i[0];
    end
    $display("COV2 g1=%0.2f big=%0.2f a=%0.2f b=%0.2f xab=%0.2f xac=%0.2f",
      g1.get_coverage(), g1.cp_big.get_coverage(), g1.cp_a.get_coverage(), g1.cp_b.get_coverage(),
      g1.xab.get_coverage(), g1.xac.get_coverage());
    $display("COV2 g2a_inst=%0.2f g2b_inst=%0.2f g2_type=%0.2f g2a_cp_inst=%0.2f",
      g2a.get_inst_coverage(), g2b.get_inst_coverage(), g2a.get_coverage(), g2a.cp.get_inst_coverage());
    $finish;
  end
endmodule
"#,
        1000,
    )
    .expect("simulate failed");
    let o: Vec<String> = sim.output.iter().map(|o| o.message.clone()).collect();
    assert!(
        o.iter()
            .any(|l| l.contains("COV2 g1=80.64 big=59.38 a=100.00 b=75.00 xab=60.71 xac=100.00")),
        "{o:?}"
    );
    assert!(
        o.iter()
            .any(|l| l
                .contains("COV2 g2a_inst=100.00 g2b_inst=60.00 g2_type=80.00 g2a_cp_inst=100.00")),
        "{o:?}"
    );
}

/// §19.3 / §19.11: a `ref` constructor formal reads its actual's current
/// value at every sample (it was captured at `new`, so every sample saw the
/// construction-time value), a bin bound may be an expression of a formal
/// (`[lo:lo+3]`), and type coverage averages the instances — also when
/// their bins are identical. Every number is the reference simulator's.
#[test]
fn ref_formals_formal_bounds_and_instance_average() {
    let sim = simulate(
        r#"
module tb;
  bit [3:0] v1, v2;
  covergroup cg (ref bit [3:0] x); cp: coverpoint x; endgroup
  cg a = new(v1), b = new(v2);
  covergroup cgb (int lo); cp: coverpoint v1 { bins r[] = {[lo:lo+3]}; } endgroup
  cgb p = new(0), q = new(0);
  initial begin
    v1 = 1; a.sample(); v1 = 2; a.sample();
    $display("M1 a=%0.2f b=%0.2f type=%0.2f", a.get_inst_coverage(), b.get_inst_coverage(), a.get_coverage());
    v2 = 7; b.sample(); v2 = 8; b.sample();
    $display("M2 a=%0.2f b=%0.2f type=%0.2f", a.get_inst_coverage(), b.get_inst_coverage(), a.get_coverage());
    v1 = 0; p.sample();
    $display("M3 p=%0.2f q=%0.2f type=%0.2f", p.get_inst_coverage(), q.get_inst_coverage(), p.get_coverage());
    $finish;
  end
endmodule
"#,
        1000,
    )
    .expect("simulate failed");
    let o: Vec<String> = sim.output.iter().map(|o| o.message.clone()).collect();
    for want in [
        "M1 a=12.50 b=0.00 type=6.25",
        "M2 a=12.50 b=12.50 type=12.50",
        "M3 p=25.00 q=0.00 type=12.50",
    ] {
        assert!(
            o.iter().any(|l| l.contains(want)),
            "missing `{want}`: {o:?}"
        );
    }
}
