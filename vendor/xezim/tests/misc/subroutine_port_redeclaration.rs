//! §23.2.2.1 / §13.3: a non-ANSI subroutine port may be completed by one later
//! variable declaration of its name. A second variable declaration, or a port
//! declaration after a variable of that name, declares it again. The rejected
//! bodies are rejected by the reference simulator too, and it runs the legal
//! ones with the same output.

use xezim::simulate;

#[test]
fn subroutine_port_redeclared() {
    for body in [
        "task t; input x; reg x; reg x; $display(\"%0d\", x); endtask initial t(1);",
        "task t; output x; reg x; reg x; x = 1; endtask reg y; initial t(y);",
        "task t; integer x; input integer x; $display(\"%0d\", x); endtask initial t(1);",
        "task t; real x; output real x; x = 1.0; endtask real y; initial t(y);",
        "task t; real x; output integer x; x = 1; endtask integer y; initial t(y);",
        "task t; reg y; reg y; $display(\"%0d\", y); endtask initial t;",
        "function integer f; input x; reg x; reg x; f = x; endfunction \
         initial $display(\"%0d\", f(1));",
        "function integer f; integer x; input integer x; f = x; endfunction \
         initial $display(\"%0d\", f(1));",
    ] {
        let src = format!("module test;\n  {body}\nendmodule\n");
        assert!(simulate(&src, 10).is_err(), "accepted:\n{src}");
    }
}

#[test]
fn subroutine_port_completed_once() {
    let src = r#"
module test;
  task t;
    input [3:0] x;
    reg [3:0] x;
    output y;
    reg y;
    $display("T|%0d", x);
    y = 1;
  endtask
  task w;
    input a;
    reg a;
    begin : b
      reg a;
    end
  endtask
  task u;
    output real r;
    real r;
    r = 2.5;
  endtask
  reg q;
  real rr;
  initial begin
    t(5, q);
    w(1'b0);
    u(rr);
    $display("T|%0d %0.1f", q, rr);
  end
endmodule
"#;
    let sim = simulate(src, 10).expect("legal non-ANSI ports must run");
    let lines: Vec<&str> = sim
        .output
        .iter()
        .filter_map(|o| o.message.strip_prefix("T|"))
        .collect();
    assert_eq!(lines, ["5", "1 2.5"], "{:?}", sim.output);
}
