//! §23.3.3 / §25.3 — a module that is NOT the root (a child instance, or any
//! top under the multi-top wrapper) refers to a member of its own interface
//! array as `INT[0].irq`. Array expansion names the elements `INT[0]`, and
//! only those element names were registered as the module's local names, so
//! the inlining rewrite never prefixed a reference whose head segment is the
//! ARRAY name with a select: the continuous assign drove a phantom unscoped
//! `INT[0].irq` and the interface's `wait (irq == 1)` never woke. With the
//! same module as the sole top the references worked. The expected lines are
//! the reference simulator's output.

use xezim::simulate;

const SRC: &str = r#"
interface irq_if;
  logic irq;
  always begin
    wait (irq == 1);
    $display("%0t %m irq up", $time);
    wait (irq == 0);
    $display("%0t %m irq down", $time);
  end
endinterface
module hdl_top;
  irq_if INT [2]();
  logic [7:0] gp;
  assign INT[0].irq = gp[0];
  assign INT[1].irq = gp[1];
  initial begin
    gp = 0;
    #5 gp = 8'h01;
    #5 gp = 8'h02;
    #5 gp = 8'h00;
    #1 $display("%0t irq0=%b irq1=%b", $time, INT[0].irq, INT[1].irq);
  end
endmodule
module hvl_top;
  initial #20 $finish;
endmodule
"#;

#[test]
fn interface_array_members_resolve_in_a_non_root_module() {
    let out: Vec<String> = simulate(SRC, 1000)
        .expect("simulate failed")
        .output
        .iter()
        .map(|o| o.message.clone())
        .collect();
    assert_eq!(
        out,
        vec![
            "5 hdl_top.INT[0] irq up".to_string(),
            "10 hdl_top.INT[0] irq down".to_string(),
            "10 hdl_top.INT[1] irq up".to_string(),
            "15 hdl_top.INT[1] irq down".to_string(),
            "16 irq0=0 irq1=0".to_string(),
        ],
        "{out:?}"
    );
}
