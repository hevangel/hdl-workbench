//! IEEE 1800-2023 §9.3.3: `break`/`continue` inside a blocking loop body
//! (`while`/`do…while`/`for` whose body has `#delay`/`wait`/`@event`) must
//! be honoured even though the loop body suspends and resumes via the
//! process-statement continuation model. Previously a `break` inside a
//! blocking `while` was silently ignored — the loop re-ran its body
//! indefinitely (the unrolled continuation re-appended the `while` stmt
//! without checking the loop-control flags the body set).
//!
//! Reference (commercial reference simulator): a `while` with `#5` + `break`
//! at i==4 produces `log == {1, 3}` (iter 2 `continue`-skipped, iter 4 broke).
use std::process::Command;

fn xezim() -> String {
    // CARGO_BIN_EXE_<name> resolves to the binary built for the ACTIVE
    // profile (debug under plain `cargo test`, release under `--release`),
    // unlike a hardcoded target/release path which breaks CI (issue #51).
    env!("CARGO_BIN_EXE_xezim").to_string()
}

fn run(src: &str, tag: &str) -> String {
    let path = format!("/tmp/blkbrk_{tag}.sv");
    std::fs::write(&path, src).unwrap();
    let out = Command::new(xezim())
        .args(["--simulate", "-s", "top", &path])
        .output()
        .expect("run xezim");
    String::from_utf8_lossy(&out.stdout).into_owned()
}

#[test]
fn blocking_while_honours_break_and_continue() {
    let src = r#"module top;
  int log[$];
  int i;
  initial begin
    i = 0;
    while (i < 10) begin
      #5;                 // blocking — loop body suspends
      i++;
      if (i == 2) continue;   // skip logging i==2
      if (i == 4) break;      // stop at i==4
      log.push_back(i);
    end
    if (log.size()==2 && log[0]==1 && log[1]==3)
      $display("RESULT PASS"); else $display("RESULT FAIL log=%p", log);
    $finish;
  end
endmodule
"#;
    let out = run(src, "while");
    assert!(
        out.contains("RESULT PASS"),
        "expected break/continue honoured\n{out}"
    );
}

#[test]
fn blocking_for_honours_break() {
    // `for` is lowered to `while` — same gate applies.
    let src = r#"module top;
  int seen[$];
  initial begin
    for (int i = 0; i < 10; i++) begin
      #1;                 // blocking
      if (i == 5) break;
      seen.push_back(i);
    end
    // seen should be {0,1,2,3,4} — break at i==5
    if (seen.size()==5 && seen[0]==0 && seen[4]==4)
      $display("RESULT PASS"); else $display("RESULT FAIL seen=%p", seen);
    $finish;
  end
endmodule
"#;
    let out = run(src, "for");
    assert!(
        out.contains("RESULT PASS"),
        "expected for-break honoured\n{out}"
    );
}

#[test]
fn blocking_repeat_honours_break() {
    let src = r#"module top;
  int log[$];
  initial begin
    repeat (10) begin
      #1;                 // blocking
      if (log.size() == 3) break;
      log.push_back(log.size());
    end
    // log == {0,1,2} — break when size reaches 3
    if (log.size()==3 && log[0]==0 && log[1]==1 && log[2]==2)
      $display("RESULT PASS"); else $display("RESULT FAIL log=%p", log);
    $finish;
  end
endmodule
"#;
    let out = run(src, "repbrk");
    assert!(
        out.contains("RESULT PASS"),
        "expected repeat-break honoured\n{out}"
    );
}

#[test]
fn blocking_repeat_honours_continue() {
    // A trailing `continue` on the final iteration must NOT leak past the
    // loop and suppress the statements after it (the historical bug).
    let src = r#"module top;
  int i; int log[$];
  initial begin
    repeat (6) begin
      #1;                 // blocking
      i++;
      if (i % 2 == 0) continue;   // skip even i
      log.push_back(i);           // push 1,3,5
    end
    // log == {1,3,5}
    if (log.size()==3 && log[0]==1 && log[1]==3 && log[2]==5)
      $display("RESULT PASS"); else $display("RESULT FAIL log=%p", log);
    $finish;
  end
endmodule
"#;
    let out = run(src, "repcon");
    assert!(
        out.contains("RESULT PASS"),
        "expected repeat-continue honoured\n{out}"
    );
}

// The cases above all place the blocking control (`#delay`) BEFORE the
// `break`/`continue`, with only NON-blocking statements after it — so they
// exercise "flag set, then skip non-blocking statements", which the
// synchronous `exec_statement` guard already handled. They do NOT cover a
// `break`/`continue` FOLLOWED BY a call to a subroutine that itself blocks.
//
// That shape was the real gap (FlooNOC-verif "bug 28"): a blocking call is
// intercepted in `run_process_stmts` AHEAD of `exec_statement`, and inlining
// it SAVES AND CLEARS the caller's break/continue flags for the callee body —
// so the guarded iterations ran the call anyway. `continue` was dropped
// entirely; `break` fired one iteration late. An inline `#delay` in the same
// spot escaped the bug only because the flag persists across suspend/resume.
//
// Reference (Verilator 5.052 + commercial): `continue` runs `work()` for odd
// i only; `break` runs it for i<4 only.

#[test]
fn blocking_for_continue_skips_call_to_blocking_task() {
    let src = r#"module top;
  bit sent[8];
  task automatic work(int unsigned i);
    #10;                       // the called subroutine consumes time
    sent[i] = 1'b1;
  endtask
  task automatic run();
    for (int unsigned i = 0; i < 8; i++) begin
      if ((i % 2) == 0) continue;   // skip even i
      work(i);                      // <-- blocking CALL after the guard
    end
  endtask
  initial begin
    int unsigned fails = 0;
    run();
    // work() must have run for ODD i only.
    for (int unsigned i = 0; i < 8; i++)
      if (sent[i] !== ((i % 2) != 0)) fails++;
    if (fails == 0) $display("RESULT PASS");
    else            $display("RESULT FAIL sent=%p", sent);
    $finish;
  end
endmodule
"#;
    let out = run(src, "for_continue_call");
    assert!(
        out.contains("RESULT PASS"),
        "continue must skip a following call to a blocking task\n{out}"
    );
}

#[test]
fn blocking_for_break_skips_call_to_blocking_task() {
    let src = r#"module top;
  bit sent[8];
  task automatic work(int unsigned i);
    #10;
    sent[i] = 1'b1;
  endtask
  task automatic run();
    for (int unsigned i = 0; i < 8; i++) begin
      if (i >= 4) break;            // stop at i==4
      work(i);                      // <-- blocking CALL after the guard
    end
  endtask
  initial begin
    int unsigned fails = 0;
    run();
    // work() must have run for i < 4 only.
    for (int unsigned i = 0; i < 8; i++)
      if (sent[i] !== (i < 4)) fails++;
    if (fails == 0) $display("RESULT PASS");
    else            $display("RESULT FAIL sent=%p", sent);
    $finish;
  end
endmodule
"#;
    let out = run(src, "for_break_call");
    assert!(
        out.contains("RESULT PASS"),
        "break must skip a following call to a blocking task\n{out}"
    );
}

// PR #174 review (aionhw): the fix above keyed its "flag consumer" exemption
// on the STATEMENT KIND (While/For/Repeat/Foreach/DoWhile), but a `break`/
// `continue`'s own loop re-entry tail is the SAME kind as a completely fresh
// nested loop that simply happens to follow the jump in source. Both look
// identical to a kind-only check, so a fresh nested loop was let through as
// if it were the outer loop's own re-entry: it "consumed" the outer flag and
// ran anyway (`continue`), or ate the flag before the outer loop's real
// re-entry ever saw it (`break`, which then never stopped the outer loop).
//
// Fixed by tracking WHICH exact statement position in a process continuation
// is a genuine synthesized re-entry (`ProcCont::loop_tail_at`), rather than
// matching on kind alone. These three cases are the reviewer's own probe.
//
// Reference (Verilator 5.052 + commercial): inner_runs=8 (continue skips
// even i, so the nested `for` runs its 2 iterations for the 4 odd i only);
// outer_iters=3 (break at i==2 stops the outer loop after i=0,1,2); r_runs=1
// (continue skips every i but 1, so the nested `repeat` runs exactly once).

#[test]
fn blocking_for_continue_skips_nested_for_with_blocking_call() {
    let src = r#"module top;
  int inner_runs = 0;
  task automatic work(int unsigned i);
    #10;
  endtask
  task automatic run();
    for (int unsigned i = 0; i < 8; i++) begin
      if ((i % 2) == 0) continue;         // skip even i
      for (int unsigned j = 0; j < 2; j++) begin  // <-- fresh nested loop
        inner_runs++;
        work(i);
      end
    end
  endtask
  initial begin
    run();
    if (inner_runs == 8) $display("RESULT PASS");
    else                 $display("RESULT FAIL inner_runs=%0d", inner_runs);
    $finish;
  end
endmodule
"#;
    let out = run(src, "for_continue_nested_for");
    assert!(
        out.contains("RESULT PASS"),
        "continue must skip a following FRESH nested for-loop, not be consumed by it\n{out}"
    );
}

#[test]
fn blocking_for_break_not_consumed_by_nested_foreach_with_blocking_call() {
    let src = r#"module top;
  int hits[8] = '{2,2,2,2,2,2,2,2};
  int outer_iters = 0;
  task automatic work(int unsigned i);
    #10;
  endtask
  task automatic run();
    for (int unsigned i = 0; i < 8; i++) begin
      outer_iters++;
      if (i >= 2) break;
      foreach (hits[k]) if (k == 0) work(i);   // <-- fresh nested loop
    end
  endtask
  initial begin
    run();
    if (outer_iters == 3) $display("RESULT PASS");
    else                  $display("RESULT FAIL outer_iters=%0d", outer_iters);
    $finish;
  end
endmodule
"#;
    let out = run(src, "for_break_nested_foreach");
    assert!(
        out.contains("RESULT PASS"),
        "a following FRESH nested foreach must not swallow the outer break\n{out}"
    );
}

#[test]
fn blocking_for_continue_skips_nested_repeat_with_blocking_call() {
    let src = r#"module top;
  int r_runs = 0;
  task automatic work(int unsigned i);
    #10;
  endtask
  task automatic run();
    for (int unsigned i = 0; i < 4; i++) begin
      if (i != 1) continue;
      repeat (1) begin           // <-- fresh nested loop, same kind as a tail
        work(i);
        r_runs++;
      end
    end
  endtask
  initial begin
    run();
    if (r_runs == 1) $display("RESULT PASS");
    else             $display("RESULT FAIL r_runs=%0d", r_runs);
    $finish;
  end
endmodule
"#;
    let out = run(src, "for_continue_nested_repeat");
    assert!(
        out.contains("RESULT PASS"),
        "continue must skip a following FRESH nested repeat, not be consumed by it\n{out}"
    );
}

// The review also names `while` after `continue` as failing the same way,
// and `do…while` as already correct. Both lower to a `While` re-entry tail
// (a `for` does too), so pin both: the nested `while` is the fresh-vs-tail
// ambiguity in its rawest form, and the `do…while` case guards the
// body-first unroll (`body; while (cond) body`) against a regression.
// Reference: w_runs=2, dw_runs=2 (continue skips every i but 1; the nested
// loop then runs its 2 iterations exactly once).

#[test]
fn blocking_for_continue_skips_nested_while_with_blocking_call() {
    let src = r#"module top;
  int w_runs = 0;
  task automatic work(int unsigned i);
    #10;
  endtask
  task automatic run();
    for (int unsigned i = 0; i < 4; i++) begin
      int unsigned j = 0;
      if (i != 1) continue;
      while (j < 2) begin          // <-- fresh nested loop, same kind as a tail
        work(i);
        w_runs++;
        j++;
      end
    end
  endtask
  initial begin
    run();
    if (w_runs == 2) $display("RESULT PASS");
    else             $display("RESULT FAIL w_runs=%0d", w_runs);
    $finish;
  end
endmodule
"#;
    let out = run(src, "for_continue_nested_while");
    assert!(
        out.contains("RESULT PASS"),
        "continue must skip a following FRESH nested while, not be consumed by it\n{out}"
    );
}

#[test]
fn blocking_for_continue_skips_nested_do_while_with_blocking_call() {
    let src = r#"module top;
  int dw_runs = 0;
  task automatic work(int unsigned i);
    #10;
  endtask
  task automatic run();
    for (int unsigned i = 0; i < 4; i++) begin
      int unsigned j = 0;
      if (i != 1) continue;
      do begin                     // <-- fresh nested loop (body-first unroll)
        work(i);
        dw_runs++;
        j++;
      end while (j < 2);
    end
  endtask
  initial begin
    run();
    if (dw_runs == 2) $display("RESULT PASS");
    else              $display("RESULT FAIL dw_runs=%0d", dw_runs);
    $finish;
  end
endmodule
"#;
    let out = run(src, "for_continue_nested_do_while");
    assert!(
        out.contains("RESULT PASS"),
        "continue must skip a following FRESH nested do-while\n{out}"
    );
}
