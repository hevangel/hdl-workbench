//! §16.4 deferred immediate assertions (`assert #0`, `assert final`): the
//! condition is evaluated where the statement runs, but the action block
//! runs only when the report matures at the end of the time slot — and a
//! report is flushed when its process resumes from a wait before then
//! (§16.4.2). Reference-validated, including that reports still pending at
//! `$finish` never run their action blocks.

use xezim::simulate;

fn outs(sim: &xezim::compiler::Simulator) -> Vec<String> {
    sim.output
        .iter()
        .map(|o| o.message.clone())
        .filter(|m| !m.starts_with("Simulation"))
        .collect()
}

/// Reports flushed by a `#0` and an event-control resume, and one pending
/// at `$finish`.
#[test]
fn deferred_reports_flush_on_resume() {
    let src = r#"
module ta;
  logic s = 0;
  initial begin
    assert #0 (0) else $display("A fail at %0t", $time);
    $display("after A");
    assert #0 (0) else $display("B fail (suspends via #0 after)");
    #0;
    $display("after #0");
    assert final (0) else $display("C final fail");
    $display("after C");
    assert #0 (0) else $display("D fail then event wait");
    @(s);
    $display("after @s");
    #1;
    assert #0 (0) else $display("E fail then finish");
    $finish;
  end
  initial begin #0; #0; s = 1; end
endmodule
"#;
    let sim = simulate(src, 100).expect("simulate failed");
    assert_eq!(
        outs(&sim),
        ["after A", "after #0", "after C", "after @s"],
        "no action block may run"
    );
}

/// Reports mature at the end of their time slot; `#0` hops of another
/// process in the slot do not mature them early.
#[test]
fn deferred_reports_mature_at_slot_end() {
    let src = r#"
module tb;
  logic s = 0, q = 0;
  initial begin
    assert #0 (0) else $display("F1 fail at %0t", $time);
    #1;
    $display("F1 proc resumed at %0t", $time);
    assert #0 (0) else $display("F2 fail at %0t", $time);
    q <= 1;
    #1;
    $display("t=%0t", $time);
    assert #0 (0) else $display("F3 fail at %0t (proc ends, other finishes after #0s)", $time);
  end
  initial begin
    #2; #0; #0; #0;
    $display("finishing at %0t", $time);
    $finish;
  end
endmodule
"#;
    let sim = simulate(src, 100).expect("simulate failed");
    assert_eq!(
        outs(&sim),
        [
            "F1 fail at 0",
            "F1 proc resumed at 1",
            "F2 fail at 1",
            "t=2",
            "finishing at 2"
        ]
    );
}
