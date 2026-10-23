//! Three LRM-audit fixes (ivtest clkgen_logic / clkgen_reg / implicit-port7):
//!
//! * §23.2.2.4 — `output logic clk = 0`: the default on an output VARIABLE
//!   port is its initializer (was silently dropped; clk started x and stayed
//!   x through `clk = ~clk`).
//! * §9.6.2 — `disable <task_name>` terminates invocations of that task
//!   executing in OTHER processes (was: set a global break flag and hang).
//! * §6.6.1 — a net driven by the outputs of TWO instances resolves all
//!   drivers. The sub-module assigns sat in the LAZY pending list (invisible
//!   to resolve_multi_driver_nets), and the folded `$__wres` assign's bare
//!   LHS was scope-hinted INTO one instance — the net read z forever.

use xezim::simulate;

fn messages(sim: &xezim::compiler::Simulator) -> Vec<String> {
    sim.output.iter().map(|o| o.message.clone()).collect()
}

#[test]
fn output_port_default_is_initializer() {
    let sim = simulate(
        r#"
module top(output logic clk = 0);
  initial begin
    #1 $display("CLK0=%b", clk);
    $finish;
  end
  initial #10 forever #10 clk = ~clk;
endmodule
"#,
        1000,
    )
    .expect("sim");
    let msgs = messages(&sim);
    assert!(
        msgs.iter().any(|m| m == "CLK0=0"),
        "output-port default must initialize the variable; output: {:?}",
        msgs
    );
}

/// §23.2.2.4: the same initializer must survive hierarchy flattening.
#[test]
fn nested_output_port_default_is_initializer() {
    let sim = simulate(
        r#"
module source_unit(output logic ready = 0);
endmodule
module shell;
  wire observed;
  source_unit u_source(.ready(observed));
  initial begin #1; $display("READY=%b", observed); $finish; end
endmodule
"#,
        1000,
    )
    .expect("sim");
    let msgs = messages(&sim);
    assert!(
        msgs.iter().any(|m| m == "READY=0"),
        "nested output-port initializer was lost: {:?}",
        msgs
    );
}

#[test]
fn disable_task_terminates_other_process() {
    let sim = simulate(
        r#"
module top;
  reg alive = 0;
  initial begin
    #30;
    disable ticker;
    #25;
    $display("AFTER=%b", alive);
    $finish;
  end
  initial fork ticker; join
  task ticker;
    forever begin
      #10 alive = ~alive;
    end
  endtask
endmodule
"#,
        1000,
    )
    .expect("sim");
    let msgs = messages(&sim);
    // ticker toggled at 10,20 (alive=0) then was disabled at 30 — the #40/#50
    // toggles never happen, so alive keeps its t<=20 value.
    assert!(
        msgs.iter().any(|m| m == "AFTER=0"),
        "disable <task> must stop the other process's invocation; output: {:?}",
        msgs
    );
}

#[test]
fn two_instance_outputs_resolve_on_one_net() {
    let sim = simulate(
        r#"
module m(input a, output b);
  assign b = a;
endmodule
module top;
  reg x; wire agree; wire clash;
  m u1(.a(x), .b(agree));
  m u2(.a(x), .b(agree));
  m u3(.a(x), .b(clash));
  m u4(.a(~x), .b(clash));
  initial begin
    x = 1;
    #1 $display("AGREE=%b CLASH=%b", agree, clash);
    $finish;
  end
endmodule
"#,
        1000,
    )
    .expect("sim");
    let msgs = messages(&sim);
    assert!(
        msgs.iter().any(|m| m == "AGREE=1 CLASH=x"),
        "§6.6.1 wired resolution across instance outputs; output: {:?}",
        msgs
    );
}
