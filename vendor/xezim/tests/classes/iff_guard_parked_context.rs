//! `@(posedge clk iff guard)` from a PARKED class task: the guard reads
//! class fields (`vif`, `master_id`) through the waiter's OWN process
//! context. With two instances parked on `vif.gnt[master_id] === 1` and
//! only instance 0 granted, evaluating the guard in the ambient (module)
//! context resolves another instance's fields — instance 1 fires
//! spuriously (`GRANTED id=1 gnt=0`) — the ubus arbitration shape.
//! Reference-verified: only `GRANTED t=15 id=0 gnt=1` fires, TAG_PASS.
use xezim::simulate;

fn messages(src: &str) -> Vec<String> {
    let sim = simulate(src, 1_000).expect("simulate failed");
    sim.output.iter().map(|o| o.message.clone()).collect()
}

const SRC: &str = r#"
interface tb_if(input logic clk);
  logic req = 0, gnt = 0;
endinterface

module top;
  logic clk = 0;
  always #5 clk = ~clk;
  tb_if vif0(.clk(clk)), vif1(.clk(clk));

  // Grant ONLY master 0's vif: under a swapped context BOTH guards read
  // values of the wrong instance.
  always @(negedge clk) begin
    if (vif0.req && !vif0.gnt) vif0.gnt <= 1;
  end

  class arb;
    virtual tb_if vif;
    int master_id;
    function new(virtual tb_if v, int id);
      vif = v; master_id = id;
    endfunction
    task arbitrate();
      vif.req <= 1;
      @(posedge vif.clk iff vif.gnt[master_id] === 1);
      vif.req <= 0;
      $display("GRANTED t=%0t id=%0d gnt=%b", $time, master_id, vif.gnt);
    endtask
  endclass

  class spinner;
    task spin();
      repeat (6) @(negedge clk);
    endtask
  endclass

  initial begin
    arb m0, m1;
    spinner s;
    m0 = new(vif0, 0);
    m1 = new(vif1, 1);
    s = new();
    fork m0.arbitrate(); m1.arbitrate(); s.spin(); join_any
    #1;
    if (vif0.req === 0 && vif0.gnt === 1 && vif1.req === 1)
      $display("TAG_PASS");
    else
      $display("TAG_FAIL");
    $finish;
  end
endmodule
"#;

#[test]
fn iff_guard_evaluates_in_waiter_context() {
    let msgs = messages(SRC);
    assert_eq!(
        msgs.iter().filter(|m| m.starts_with("GRANTED")).count(),
        1,
        "exactly one grant expected, got {msgs:?}"
    );
    assert!(
        msgs.iter().any(|m| m == "GRANTED t=15 id=0 gnt=1"),
        "expected GRANTED t=15 id=0 gnt=1, got {msgs:?}"
    );
    assert!(
        msgs.iter().any(|m| m == "TAG_PASS"),
        "expected TAG_PASS, got {msgs:?}"
    );
}
