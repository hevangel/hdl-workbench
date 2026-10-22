//! §18.6.2 — an infeasible constraint set over a wide domain must fail
//! promptly. Here propagation cannot refute the set (`%` is judged by the
//! evaluator once the variable is fixed), so the joint solver searches
//! 32-bit values one at a time; every refuted value splits the domain once
//! more, and without charging that split to the work budget the search ran
//! its whole node budget at a quadratic cost. The joint solver's debug line
//! (XEZIM_RAND_DBG) reports the nodes it spent: well under the node budget
//! (40000) means the work budget stopped it. The reference simulator also
//! reports no solution.

use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

const SRC: &str = r#"
class w_c;
  rand bit [31:0] x;
  rand bit [31:0] y;
  constraint c { x % 7 == 3; y == x + 1; y % 7 == 5; }
endclass
module top;
  initial begin
    automatic w_c w = new;
    automatic int r;
    w.x = 11; w.y = 22;
    r = w.randomize();
    $display("R r=%0d x=%0d y=%0d", r, w.x, w.y);
  end
endmodule
"#;

const NESTED_SRC: &str = r#"
class leaf_c;
  rand bit [31:0] x;
  rand bit [31:0] y;
  constraint impossible_c { x % 7 == 3; y == x + 1; y % 7 == 5; }
endclass
class root_c;
  rand leaf_c child;
  rand bit [31:0] marker;
  function new(); child = new; endfunction
endclass
module top;
  initial begin
    root_c cfg = new;
    int result;
    cfg.child.x = 17;
    cfg.child.y = 29;
    cfg.marker = 32'h12345678;
    result = cfg.randomize();
    $display("N result=%0d x=%0d y=%0d marker=%08x",
             result, cfg.child.x, cfg.child.y, cfg.marker);
  end
endmodule
"#;

const INLINE_SRC: &str = r#"
class item_c;
  rand bit [31:0] x;
  rand bit [31:0] y;
endclass
module top;
  initial begin
    item_c item = new;
    int result;
    item.x = 31;
    item.y = 47;
    result = item.randomize() with {
      x % 7 == 3;
      y == x + 1;
      y % 7 == 5;
    };
    $display("I result=%0d x=%0d y=%0d", result, item.x, item.y);
  end
endmodule
"#;

const RESET_SRC: &str = r#"
class blocked_c;
  rand bit [31:0] x;
  rand bit [31:0] y;
  constraint blocked_rule { x % 7 == 3; y == x + 1; y % 7 == 5; }
endclass
class ready_c;
  rand bit value;
endclass
module top;
  initial begin
    blocked_c blocked = new;
    ready_c ready = new;
    int blocked_result;
    int ready_result;
    blocked.x = 9;
    blocked.y = 12;
    blocked_result = blocked.randomize();
    ready_result = ready.randomize();
    $display("B result=%0d x=%0d y=%0d", blocked_result, blocked.x, blocked.y);
    $display("S result=%0d", ready_result);
  end
endmodule
"#;

fn run_with_deadline(
    source: &str,
    stem: &str,
    plusargs: &[&str],
) -> (std::process::ExitStatus, String, Duration) {
    // One directory per case: the cases run as parallel threads of one test
    // process, and each removes its directory when done, so a directory
    // shared by process id could vanish before another case's simulator
    // read its design.
    let dir = std::env::temp_dir().join(format!("xezim_rand_budget_{}_{stem}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("create temporary directory");
    let path = dir.join(format!("{stem}.sv"));
    std::fs::write(&path, source).expect("write design");
    let mut command = Command::new(env!("CARGO_BIN_EXE_xezim"));
    command.args(["--simulate", "--no-cache", path.to_str().unwrap()]);
    command.args(plusargs);
    let mut child = command
        .env("XEZIM_RAND_DBG", "1")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("run randomization case");
    let start = Instant::now();
    let status = loop {
        if let Some(status) = child.try_wait().expect("wait") {
            break status;
        }
        if start.elapsed() > Duration::from_secs(120) {
            let _ = child.kill();
            panic!("{stem} did not finish in 120 s");
        }
        std::thread::sleep(Duration::from_millis(20));
    };
    let elapsed = start.elapsed();
    let mut text = String::new();
    use std::io::Read;
    child.stdout.take().unwrap().read_to_string(&mut text).ok();
    child.stderr.take().unwrap().read_to_string(&mut text).ok();
    let _ = std::fs::remove_dir_all(&dir);
    (status, text, elapsed)
}

#[test]
fn infeasible_wide_set_fails_promptly() {
    let (status, text, _) = run_with_deadline(SRC, "wide", &[]);
    assert!(status.success(), "run failed:\n{text}");
    // failed, and left the variables as they were (§18.6.2)
    assert!(text.contains("R r=0 x=11 y=22"), "{text}");
    let nodes: Vec<u64> = text
        .lines()
        .filter_map(|l| l.strip_prefix("[rand-dbg] joint solve gave up: nodes="))
        .filter_map(|r| r.split_whitespace().next()?.parse().ok())
        .collect();
    assert!(!nodes.is_empty(), "no joint-solve report:\n{text}");
    assert!(
        nodes.iter().all(|&n| n < 20_000),
        "the search ran on its node budget: {nodes:?}"
    );
}

#[test]
fn nested_infeasible_object_obeys_shared_trial_budget() {
    let (status, text, elapsed) = run_with_deadline(
        NESTED_SRC,
        "nested_trials",
        &["+xezim_rand_trials=2", "+xezim_rand_timeout=30"],
    );
    assert!(status.success(), "run failed:\n{text}");
    assert!(
        elapsed < Duration::from_secs(10),
        "nested randomization exceeded its shared trial budget: {elapsed:?}\n{text}"
    );
    assert!(
        text.contains("N result=0 x=17 y=29 marker=12345678"),
        "{text}"
    );
    assert!(
        text.contains("randomize budget exhausted"),
        "missing budget diagnostic:\n{text}"
    );
    assert!(
        text.contains("last failed constraints: impossible_c"),
        "missing failing constraint name:\n{text}"
    );
    assert_eq!(
        text.matches("randomize budget exhausted").count(),
        1,
        "the nested solve must report exhaustion once:\n{text}"
    );
}

#[test]
fn nested_infeasible_object_obeys_wall_clock_budget() {
    let (status, text, elapsed) = run_with_deadline(
        NESTED_SRC,
        "nested_timeout",
        &["+xezim_rand_trials=100000000", "+xezim_rand_timeout=1"],
    );
    assert!(status.success(), "run failed:\n{text}");
    assert!(
        elapsed < Duration::from_secs(10),
        "nested randomization exceeded its wall-clock budget: {elapsed:?}\n{text}"
    );
    assert!(
        text.contains("N result=0 x=17 y=29 marker=12345678"),
        "{text}"
    );
    assert!(text.contains("randomize budget exhausted"), "{text}");
    assert!(
        text.contains("last failed constraints: impossible_c"),
        "missing failing constraint name:\n{text}"
    );
}

#[test]
fn inline_constraints_obey_budget_and_roll_back() {
    let (status, text, elapsed) = run_with_deadline(
        INLINE_SRC,
        "inline_trials",
        &["+xezim_rand_trials=2", "+xezim_rand_timeout=30"],
    );
    assert!(status.success(), "run failed:\n{text}");
    assert!(elapsed < Duration::from_secs(10), "{elapsed:?}\n{text}");
    assert!(text.contains("I result=0 x=31 y=47"), "{text}");
    assert!(
        text.contains("randomize budget exhausted in class item_c"),
        "missing inline-constraint diagnostic:\n{text}"
    );
}

#[test]
fn exhausted_budget_is_reset_for_the_next_call() {
    let (status, text, elapsed) = run_with_deadline(
        RESET_SRC,
        "budget_reset",
        &["+xezim_rand_trials=2", "+xezim_rand_timeout=30"],
    );
    assert!(status.success(), "run failed:\n{text}");
    assert!(elapsed < Duration::from_secs(10), "{elapsed:?}\n{text}");
    assert!(text.contains("B result=0 x=9 y=12"), "{text}");
    assert!(
        text.contains("S result=1"),
        "the next outer call inherited an exhausted budget:\n{text}"
    );
    assert_eq!(
        text.matches("randomize budget exhausted").count(),
        1,
        "only the infeasible call should exhaust its budget:\n{text}"
    );
}
