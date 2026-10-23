//! §19.7 instance options read as `cg.option.<name>`: the covergroup's own
//! setting, else the default (reads used to return x). An option written at
//! run time is stored but, as in the reference simulator, does not change
//! the coverage. Every expected value was cross-checked against the
//! reference simulator.

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
fn read_instance_options() {
    let l = lines(
        r#"
module top;
  bit [1:0] x, y;
  covergroup cg;
    option.goal = 90;
    cpx : coverpoint x { bins b0 = {0}; bins b1 = {1}; option.weight = 2; }
    cpy : coverpoint y;
  endgroup
  covergroup cg2; cpx : coverpoint x; endgroup
  cg c = new();
  cg2 d = new();
  initial begin
    $display("goal %0d at_least %0d weight %0d abm %0d per_instance %0d", c.option.goal,
             c.option.at_least, c.option.weight, c.option.auto_bin_max, c.option.per_instance);
    $display("d goal %0d", d.option.goal);
    x = 0; y = 0; c.sample(); c.sample();
    y = 1; c.sample();
    $display("before %0.2f", c.get_inst_coverage());
    c.option.at_least = 2;
    $display("after at_least=2: %0.2f", c.get_inst_coverage());
    $display("at_least now %0d", c.option.at_least);
  end
endmodule
"#,
    );
    assert_line(&l, "goal 90 at_least 1 weight 1 abm 64 per_instance 0");
    assert_line(&l, "d goal 100");
    assert_line(&l, "before 50.00");
    assert_line(&l, "after at_least=2: 50.00");
    assert_line(&l, "at_least now 2");
}
