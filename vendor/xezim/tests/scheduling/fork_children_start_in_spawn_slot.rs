//! §9.3.2: `fork` children begin executing in the time slot that spawned
//! them, ahead of whatever is still pending for that timestamp.
//!
//! Children used to be appended to the BACK of the current timestamp's
//! queue, so any event already pending there — a clock toggle, another
//! process resuming from the same `#delay` — ran first. A child whose first
//! act is an event control then armed AFTER the edge it was meant to catch
//! and slept until the next one. In a UVM bench that is a full clock period
//! of skew in every driver and monitor, since those bodies all run forked,
//! and it is invisible in the source.
//!
//! Both expectations are reference-verified.

use std::process::Command;

fn run(src: &str) -> String {
    let dir = std::env::temp_dir().join(format!("xezim_forkslot_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("tb.sv");
    std::fs::write(&path, src).unwrap();
    let out = Command::new(env!("CARGO_BIN_EXE_xezim"))
        .args([
            "--simulate",
            "-s",
            "top",
            path.to_str().unwrap(),
            "--no-cache",
        ])
        .output()
        .expect("run xezim");
    let mut text = String::from_utf8_lossy(&out.stdout).to_string();
    text.push_str(&String::from_utf8_lossy(&out.stderr));
    let _ = std::fs::remove_dir_all(&dir);
    text
}

#[test]
fn fork_child_arms_before_later_queued_same_time_events() {
    // No clock here on purpose: the racing edge comes from a second process
    // resuming from the same `#delay`, so this pins the queue position of a
    // spawned child and nothing else.
    let text = run(r#"module top;
  reg s = 0;
  int t_child = -1;
  string order;
  // Declared FIRST, so this process is queued for t=10 ahead of the one
  // below. Its child must run before that later-queued process, or the
  // edge it is waiting for happens before it ever arms.
  initial begin
    #10;
    fork begin @(posedge s); t_child = $time; end join
  end
  initial begin
    #10;
    s = 1'b1;
  end
  // Siblings must start in source order.
  initial begin
    #20;
    order = "";
    fork
      order = {order, "a"};
      order = {order, "b"};
      order = {order, "c"};
    join
  end
  initial begin
    #40;
    $display("F child=%0d order=%s", t_child, order);
    $finish;
  end
endmodule
"#);
    assert!(
        text.contains("F child=10 order=abc"),
        "fork child missed the same-slot edge, or siblings lost source order:\n{text}"
    );
}
