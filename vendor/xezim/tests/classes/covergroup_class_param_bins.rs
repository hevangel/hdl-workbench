//! #273: bins of a covergroup in a parameterized class evaluate with the
//! owning specialization's value parameters (§19.5, §8.25). `bins b[] =
//! {[0:N-1]}` in `group_n #(2)` had no bins, so one valid sample read 0%.
//! Expected values are the reference simulator's.

use xezim::simulate;

#[test]
fn class_param_bin_range_counts_samples() {
    let src = r#"
module top;
  class group_n #(int N = 4);
    covergroup cg with function sample(int i);
      option.per_instance = 1;
      cp: coverpoint i { bins b[] = {[0:N-1]}; }
    endgroup
    function new(); cg = new(); endfunction
    function void sample(int i); cg.sample(i); endfunction
    function real coverage(); return cg.get_inst_coverage(); endfunction
  endclass
  group_n #(2) g;
  initial begin
    g = new(); g.sample(0);
    $display("COVERAGE=%0.2f", g.coverage());
    if (g.coverage() != 50.0) $fatal(1, "coverage should be 50 percent");
    $display("COVERAGE_PASS"); $finish;
  end
endmodule
"#;
    let sim = simulate(src, 100).expect("simulate failed");
    let msgs: Vec<&str> = sim.output.iter().map(|l| l.message.as_str()).collect();
    assert_eq!(msgs, ["COVERAGE=50.00", "COVERAGE_PASS"]);
}

#[test]
fn class_param_bins_per_specialization() {
    // `{N}`, `[N:N*2]`, ignore/illegal bins with N, a bound mixing a class
    // and a module parameter, default and explicit specializations side by
    // side.
    let sim =
        simulate(include_str!("covergroup_class_param_bins.sv"), 100).expect("simulate failed");
    let got: Vec<&str> = sim
        .output
        .iter()
        .filter_map(|l| l.message.strip_prefix("T|"))
        .collect();
    let want = [
        "COVERAGE=50.00",
        "g4=50.00",
        "g5=20.00",
        "g2_full=100.00",
        "def one=100.00 rng=100.00 ign=66.67 ill=75.00 mix=40.00 inst=76.33",
        "s3 one=50.00 rng=100.00 ign=25.00 ill=50.00 mix=50.00 inst=55.00",
        "s42 one=50.00 rng=100.00 ign=40.00 ill=25.00 mix=50.00 inst=53.00",
        "done",
    ];
    assert_eq!(got, want);
}
