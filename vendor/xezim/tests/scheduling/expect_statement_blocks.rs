//! §16.17: `expect` BLOCKS the process — one attempt of the property starts
//! at the next clocking event, and the statement (and its pass/fail action)
//! completes when that attempt succeeds or fails. It was parsed as an
//! immediate assertion of the clocked property, which evaluated false at
//! once: the else action ran at time 0 and the process never waited
//! (sv-tests `expect_test_uvm`). A fixed-delay chain of boolean terms is now
//! lowered to the equivalent wait-and-check sequence. Times cross-checked
//! against the reference simulator.

fn lines(src: &str) -> Vec<String> {
    xezim::simulate(src, 1000)
        .expect("simulate")
        .output
        .iter()
        .map(|o| o.message.clone())
        .collect()
}

const TB: &str = r#"
module top;
  bit clk = 0;
  reg read = 1, write = 0;
  always #50 clk = ~clk;
  always @(posedge clk) begin read <= 0; write <= ~write; end
  initial begin
    PROP
    $display("X|after %0t", $time);
    #400 $finish;
  end
endmodule
"#;

#[test]
fn passing_expect_waits_for_the_whole_sequence() {
    let o = lines(&TB.replace(
        "PROP",
        "expect (@(posedge clk) read ##1 write) $display(\"X|pass %0t\", $time); \
         else $display(\"X|fail %0t\", $time);",
    ));
    let x: Vec<&String> = o.iter().filter(|l| l.starts_with("X|")).collect();
    assert_eq!(x, vec!["X|pass 150", "X|after 150"], "{o:?}");
}

#[test]
fn failing_expect_runs_the_else_action_at_the_failing_clock() {
    let o = lines(&TB.replace(
        "PROP",
        "expect (@(posedge clk) read ##2 read) $display(\"X|pass %0t\", $time); \
         else $display(\"X|fail %0t\", $time);",
    ));
    let x: Vec<&String> = o.iter().filter(|l| l.starts_with("X|")).collect();
    assert_eq!(x, vec!["X|fail 250", "X|after 250"], "{o:?}");
}
