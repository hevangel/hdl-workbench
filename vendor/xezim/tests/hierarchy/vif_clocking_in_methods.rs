//! §14.13 / §14.4 / §25.9 — clocking-block waits and drives through a virtual
//! interface inside CLASS METHODS. Reference-validated.
//!
//! Inside a subroutine body the parser emits `MemberAccess` chains where
//! module scope has one flat identifier, and two clocking paths matched only
//! the flat spelling:
//!
//! * the event-sensitivity collector — `@(vif.cb)` collected NO identifiers,
//!   armed the wait on nothing, and returned immediately in the same timestep;
//! * the NBA clocking-drive check — `vif.cb.data <= v` skipped the §14.4
//!   deferral and drove a phantom name, so the output net never changed.
//!
//! Both are now normalized to the flat form up front. The aliasing itself was
//! already proven correct — the clocking KEY resolved fine — which is what
//! distinguished this from the earlier vif findings.

use xezim::simulate;

fn u(sim: &xezim::compiler::Simulator, n: &str) -> u64 {
    sim.get_signal(n)
        .or_else(|| sim.get_signal(&format!("tb.{}", n)))
        .unwrap_or_else(|| panic!("signal not found: {}", n))
        .to_u64()
        .unwrap_or_else(|| panic!("{} not u64-able (x/z?)", n))
}

#[test]
fn clocking_wait_and_drive_through_a_vif_method() {
    let src = r#"
interface bus_if(input logic clk);
  logic [7:0] data;
  clocking cb @(posedge clk);
    output data;
  endclocking
endinterface
class Drv;
  virtual bus_if vif;
  function new(virtual bus_if v); vif = v; endfunction
  task w_cb();   @(vif.cb);          endtask
  task w_edge(); @(posedge vif.clk); endtask
  task d_cb();   vif.cb.data <= 8'h11; endtask
  task d_plain(); vif.data = 8'h22;  endtask
endclass
module tb;
  logic clk = 0;
  always #5 clk = ~clk;
  bus_if bi(clk);
  Drv dr;
  int t_edge, t_cb, data_after_plain, data_after_cb;
  initial begin
    dr = new(bi);
    dr.w_edge();
    t_edge = $time;
    dr.d_plain();
    #1 data_after_plain = bi.data;
    dr.w_cb();                       // must block until the NEXT cb event
    t_cb = $time;
    dr.d_cb();                       // §14.4: lands at the next cb edge
    @(posedge clk);
    #1 data_after_cb = bi.data;
  end
endmodule
"#;
    let sim = simulate(src, 500).expect("simulate failed");
    assert_eq!(
        u(&sim, "t_edge"),
        5,
        "@(posedge vif.clk) in a method blocks"
    );
    assert_eq!(
        u(&sim, "data_after_plain"),
        0x22,
        "a plain drive still lands immediately"
    );
    assert_eq!(
        u(&sim, "t_cb"),
        15,
        "@(vif.cb) waits for the NEXT clocking event, not zero time"
    );
    assert_eq!(
        u(&sim, "data_after_cb"),
        0x11,
        "the clocking drive lands at the cb edge"
    );
}
