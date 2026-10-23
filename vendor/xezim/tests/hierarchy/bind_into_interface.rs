//! §23.11 — a `bind` whose target is an INTERFACE (by definition name or by
//! instance path). Only module targets were handled, so the bound instance
//! was silently dropped; an unbound checker interface was then picked up as
//! an extra top-level root with unconnected ports (an AVIP's assertion
//! interfaces bound into its monitor BFM interfaces never saw a signal).
//!
//! The expected lines were cross-checked against the reference simulator.

use xezim::simulate;

const SRC: &str = r#"
interface chk (input logic a);
  initial $display("CHK up %m");
  always @(posedge a) $display("%0t %m saw a", $time);
endinterface
interface mon (input logic a);
endinterface
interface pmon (input logic a);
endinterface
module agent;
  logic a = 0;
  mon u_mon(.a(a));
  pmon u_pmon(.a(a));
  initial #5 a = 1;
endmodule
module top;
  agent u_agent();
  agent u_agent2();
  initial #10 $finish;
endmodule
bind mon chk C (.a(a));
bind top.u_agent2.u_pmon chk C2 (.a(a));
"#;

#[test]
fn bind_attaches_to_interface_targets() {
    let sim = simulate(SRC, 1000).expect("simulate failed");
    let got: Vec<String> = sim.output.iter().map(|o| o.message.clone()).collect();
    assert_eq!(
        got,
        vec![
            "CHK up top.u_agent.u_mon.C",
            "CHK up top.u_agent2.u_mon.C",
            "CHK up top.u_agent2.u_pmon.C2",
            "5 top.u_agent.u_mon.C saw a",
            "5 top.u_agent2.u_mon.C saw a",
            "5 top.u_agent2.u_pmon.C2 saw a",
        ]
    );
}
