//! `XEZIM_TRACE_SCHED` emits an ORDERED log of what runs inside each time
//! slot. Every other trace answers "what changed and when"; a race is a
//! question about who went first INSIDE one timestamp, which none of them
//! can answer. Two same-timestamp scheduling bugs were found by bisecting
//! against a reference simulator because this did not exist.
//!
//! Pins the shape and the time window, not the scheduler's internal order:
//! the ordering itself is pinned by the tests that own each behaviour, and
//! at least one same-slot ordering issue is still open.

use std::process::Command;

fn run(window: &str, src: &str) -> String {
    static SEQ: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);
    let dir = std::env::temp_dir().join(format!(
        "xezim_schedtrace_{}_{}",
        std::process::id(),
        SEQ.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
    ));
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
        .env("XEZIM_TRACE_SCHED", window)
        .output()
        .expect("run xezim");
    let mut text = String::from_utf8_lossy(&out.stdout).to_string();
    text.push_str(&String::from_utf8_lossy(&out.stderr));
    let _ = std::fs::remove_dir_all(&dir);
    text
}

const SRC: &str = r#"module top;
  reg clk = 0;
  always #5 clk = ~clk;
  int hits = 0;
  always @(posedge clk) hits = hits + 1;
  initial begin
    #12;
    #20;
    $display("D hits=%0d", hits);
    $finish;
  end
endmodule
"#;

#[test]
fn sched_trace_emits_ordered_slot_entries() {
    let text = run("1", SRC);
    let lines: Vec<&str> = text.lines().filter(|l| l.starts_with("[sched]")).collect();
    assert!(!lines.is_empty(), "no [sched] output:\n{text}");
    assert!(
        lines.iter().any(|l| l.contains("+--- tick ---")),
        "no tick header:\n{text}"
    );
    // Both kinds of slot entry must be attributed: clock generators by
    // signal name, processes by pid AND source origin.
    assert!(
        lines.iter().any(|l| l.contains("clockgen clk ->")),
        "clock generator toggles not traced:\n{text}"
    );
    assert!(
        lines
            .iter()
            .any(|l| l.contains("proc pid=") && l.contains("block at")),
        "process entries not attributed to a source origin:\n{text}"
    );
    // The simulation must still behave identically with tracing on.
    assert!(text.contains("D hits="), "run did not complete:\n{text}");
}

#[test]
fn sched_trace_honours_its_time_window() {
    let text = run("20:25", SRC);
    let times: Vec<u64> = text
        .lines()
        .filter(|l| l.starts_with("[sched] t="))
        .filter_map(|l| l["[sched] t=".len()..].split_whitespace().next())
        .filter_map(|t| t.parse().ok())
        .collect();
    assert!(!times.is_empty(), "window produced no output:\n{text}");
    assert!(
        times.iter().all(|&t| (20..=25).contains(&t)),
        "emitted outside the requested window: {times:?}"
    );
}

#[test]
fn sched_trace_is_off_by_default() {
    let text = run("0", SRC);
    assert!(
        !text.contains("[sched]"),
        "tracing emitted while disabled:\n{text}"
    );
}
