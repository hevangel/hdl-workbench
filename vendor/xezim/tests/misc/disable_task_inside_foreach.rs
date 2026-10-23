//! §9.6.2: `disable <task>` inside a `foreach` in that task ends the task.
//! The `foreach` arms treated the pending disable as a plain `break`: they
//! cleared the flag, so the task's remaining statements ran, and the
//! disable target stayed set — a later, unrelated loop then kept its own
//! `break` flag raised past its end. With compiled blocks now running more
//! of their statements through carried-local fallbacks next to such calls,
//! the leak surfaced as a comb block reading a stale loop count. And the
//! task-call arm compared the target with the task's instance-qualified
//! name (`u0.early`), so a `disable early` issued inside an instance never
//! cleared at all: every later loop ran past its `break` and the rest of
//! the run printed nothing. Expected values are the reference simulator's.

use xezim::simulate;

#[test]
fn disable_of_the_task_inside_foreach_ends_the_task() {
    let src = r#"
module sub;
  int aa [int];
  initial begin aa[1] = 10; aa[3] = 30; end
  task automatic early(input int v, output int r);
    r = 1;
    foreach (aa[k]) if (k == v) disable early;
    r = 2;
  endtask
  task automatic early2(input int v, output int r);
    r = 1;
    if (v == 3) disable early2;
    r = 2;
  endtask
  initial begin
    int a, b, c, d;
    #1;
    early(3, a); early(4, b); early2(3, c); early2(4, d);
    $display("T| sub %0d %0d %0d %0d", a, b, c, d);
  end
endmodule
module tb;
  int aa [int];
  initial begin aa[1] = 10; aa[3] = 30; end
  task automatic early(input int v, output int r);
    r = 1;
    foreach (aa[k]) if (k == v) disable early;
    r = 2;
  endtask
  sub u ();
  initial begin
    int a, b;
    #2;
    early(3, a); early(4, b);
    $display("T| top %0d %0d", a, b);
    // a loop after the disabled call: its own `break` must still stop it
    for (a = 0; a < 5; a++) if (a == 2) break;
    b = 0;
    repeat (3) b++;
    $display("T| after %0d %0d", a, b);
  end
endmodule
"#;
    let sim = simulate(src, 100).expect("design must run");
    let got: Vec<&str> = sim
        .output
        .iter()
        .map(|o| o.message.as_str())
        .filter(|m| m.starts_with("T|"))
        .collect();
    assert_eq!(got, ["T| sub 1 2 1 2", "T| top 1 2", "T| after 2 3"]);
}

#[test]
fn disable_of_an_instance_task_clears_at_its_return() {
    let src = r#"
module sub #(parameter ID = 0) (input logic clk);
  int aa [int];
  initial begin aa[1] = 10; aa[3] = 30; end
  task automatic early(input int v, output int r);
    r = 1;
    foreach (aa[k]) if (k == v) disable early;
    r = 2;
  endtask
  always @(posedge clk) begin
    int a, b, n;
    early(3, a); early(4, b);
    n = 0;
    for (int i = 0; i < 5; i++) begin if (i == 2) break; n++; end
    $display("T| %0d early %0d %0d n=%0d", ID, a, b, n);
  end
endmodule
module tb;
  logic clk = 0;
  sub #(0) u0 (.clk(clk));
  sub #(1) u1 (.clk(clk));
  int m;
  initial begin
    #5 clk = 1; #5 clk = 0;
    m = 0;
    for (int i = 0; i < 5; i++) begin if (i == 3) break; m++; end
    $display("T| tb m=%0d", m);
    #1 $finish;
  end
endmodule
"#;
    let sim = simulate(src, 100).expect("design must run");
    let got: Vec<&str> = sim
        .output
        .iter()
        .map(|o| o.message.as_str())
        .filter(|m| m.starts_with("T|"))
        .collect();
    assert_eq!(
        got,
        ["T| 0 early 1 2 n=2", "T| 1 early 1 2 n=2", "T| tb m=3"]
    );
}
