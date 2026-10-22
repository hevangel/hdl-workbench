//! §19.3 / §19.8.1: a covergroup with both constructor formals and
//! `with function sample` formals sees both. The sample formals used to be
//! bound in a frame that the constructor formals' frame then hid, so no bin
//! was ever hit. Every expected number was cross-checked against the
//! reference simulator.

use xezim::simulate;

#[test]
fn constructor_and_sample_formals_together() {
    let sim = simulate(
        r#"
module tb;
  int x;
  covergroup s_cg with function sample (int v);
    cp : coverpoint v { bins b0 = {0}; bins b9 = {9}; }
  endgroup
  covergroup c_cg (int lo, int hi);
    cp : coverpoint x { bins in_lo = {[lo:lo+1]}; bins in_hi = {[hi-1:hi]}; }
  endgroup
  covergroup sc_cg (int lo, int hi) with function sample (int v);
    cp : coverpoint v { bins in_lo = {[lo:lo+1]}; bins in_hi = {[hi-1:hi]}; }
  endgroup
  covergroup sc2_cg (int lo, int hi) with function sample (int v);
    cp : coverpoint v { bins in_lo = {[lo:lo]}; bins in_hi = {[hi:hi]}; }
  endgroup
  covergroup sc3_cg (int lo) with function sample (int v);
    cp : coverpoint v { bins b = {lo}; bins z = {5}; }
  endgroup
  s_cg s = new();
  c_cg c = new(0, 9);
  sc_cg sc = new(0, 9);
  sc2_cg sc2 = new(0, 9);
  sc3_cg sc3 = new(0);
  initial begin
    s.sample(0);
    x = 0; c.sample();
    sc.sample(0);
    sc2.sample(0);
    sc3.sample(0);
    $display("s=%0.2f c=%0.2f sc=%0.2f sc2=%0.2f sc3=%0.2f", s.get_inst_coverage(), c.get_inst_coverage(), sc.get_inst_coverage(), sc2.get_inst_coverage(), sc3.get_inst_coverage());
  end
endmodule
"#,
        100_000,
    )
    .expect("simulate failed");
    let l: Vec<String> = sim
        .output
        .iter()
        .map(|o| o.message.trim().to_string())
        .collect();
    assert!(
        l.iter()
            .any(|m| m == "s=50.00 c=50.00 sc=50.00 sc2=50.00 sc3=50.00"),
        "{l:?}"
    );
}
