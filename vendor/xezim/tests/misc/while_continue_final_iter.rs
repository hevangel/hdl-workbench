//! §12.7.2: a `continue` taken on the FINAL iteration of a `while` reaches
//! the condition-false exit with `continue_flag` still set (the `for` arm
//! clears the flag after the body; `while` re-tests the condition first).
//! Leaked, the flag skipped every statement after the loop — a function
//! ended without executing its `return` and the caller read a STALE return
//! slot from the previous call. This silently killed any queue-scan of the
//! shape `while (i < q.size()) begin if (dead) begin i++; continue; end ...`
//! when the last element took the continue branch (UVM's sequencer
//! arbitration scan). All expected values reference-verified.

use std::process::Command;

fn run(name: &str, src: &str) -> String {
    let dir = std::env::temp_dir().join(format!("xezim_wcfi_{}_{}", name, std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join(format!("{name}.sv"));
    std::fs::write(&path, src).unwrap();
    let out = Command::new(env!("CARGO_BIN_EXE_xezim"))
        .args([
            "--simulate",
            "-s",
            "test",
            path.to_str().unwrap(),
            "--no-cache",
        ])
        .output()
        .expect("run xezim");
    let mut text = String::from_utf8_lossy(&out.stdout).to_string();
    text.push_str(&String::from_utf8_lossy(&out.stderr));
    text
}

#[test]
fn continue_on_last_while_iteration_still_returns() {
    // Second call: the single element takes the continue branch, the re-test
    // is false, and the function must still reach its `return -1`.
    let text = run(
        "final_cont",
        r#"class scan_c;
  int marks[$];
  function int choose(int dead_mark);
    int i;
    i = 0;
    while (i < marks.size()) begin
      if (marks[i] == dead_mark) begin
        $display("T|entry %0d dead", i);
        i++;
        continue;
      end
      $display("T|entry %0d alive", i);
      return i;
    end
    $display("T|loop exit");
    return -1;
  endfunction
endclass
module test;
  scan_c s = new();
  initial begin
    s.marks.push_back(7);
    $display("T|call1 -> %0d", s.choose(3));
    $display("T|call2 -> %0d", s.choose(7));
    $finish;
  end
endmodule
"#,
    );
    assert!(
        text.contains("T|call1 -> 0"),
        "alive entry selected:\n{text}"
    );
    assert!(
        text.contains("T|entry 0 dead"),
        "dead branch taken:\n{text}"
    );
    assert!(
        text.contains("T|loop exit"),
        "statements after the loop must run when the final iteration continues:\n{text}"
    );
    assert!(
        text.contains("T|call2 -> -1"),
        "second call must return -1, not a stale 0:\n{text}"
    );
}

#[test]
fn dead_process_queue_scan_returns_none() {
    // The sequencer-arbitration shape: elements hold process handles; the
    // scan skips KILLED/FINISHED entries via `continue`. When every entry is
    // dead the scan must return -1 (the previous call returned 0, so a
    // leaked flag reproduced that stale 0). status values reference-checked:
    // WAITING (2) while parked at #20, FINISHED (0) after.
    let text = run(
        "proc_scan",
        r#"class req_c;
  process process_id;
endclass
class scan_c;
  req_c arb_q[$];
  function int choose();
    int i;
    i = 0;
    while (i < arb_q.size()) begin
      if ((arb_q[i].process_id.status == process::KILLED) ||
          (arb_q[i].process_id.status == process::FINISHED)) begin
        i++;
        continue;
      end
      return i;
    end
    return -1;
  endfunction
endclass
module test;
  scan_c s = new();
  initial begin : registrant
    req_c r = new();
    r.process_id = process::self();
    s.arb_q.push_back(r);
    #20;
  end
  initial begin
    #5;
    $display("T|early choose=%0d", s.choose());
    #30;
    $display("T|late choose=%0d", s.choose());
    $finish;
  end
endmodule
"#,
    );
    assert!(
        text.contains("T|early choose=0"),
        "live entry is selectable:\n{text}"
    );
    assert!(
        text.contains("T|late choose=-1"),
        "all-dead queue must yield -1 (stale-return leak):\n{text}"
    );
}
