//! #301 (reported by Ganesh-TNL): `randomize()` returned 0, with no message,
//! for `foreach (sel[i]) { b.W == a.W; }` when `W` is a multi-dimensional
//! packed member (`bit [8:0][7:0]`, `bit [31:0][47:0]`) wider than 64 bits
//! of two rand sub-objects.
//!
//! - §7.4.1: the joint solver did not split a multi-dimensional packed
//!   variable into segments, so a wide one could not be read at all; its
//!   element and part selects now name the elements of each dimension.
//! - §18.5.8.1/§12.7.3: a `foreach` over such a member, or over an array
//!   member of a rand sub-object (`foreach (a.W[i])`), is unrolled per
//!   index, and the acceptance check now checks it (it accepted violated
//!   loops before).
//! - §11.5.1: part-selects of multi-dimensional packed class members, and
//!   selects reached through a chain of handles (`y.a.W[i][j]`) or into an
//!   element of an unpacked array of packed arrays, read the right bits.
//! - §18.6.3: a `randomize()` that fails says so on stderr.
//!
//! Expected values come from the reference simulator. Random values differ
//! between simulators, so the probes print whether the constraints hold.

use std::process::Command;
use xezim::simulate;

fn lines(src: &str) -> Vec<String> {
    simulate(src, 100)
        .expect("simulate failed")
        .output
        .iter()
        .map(|o| o.message.clone())
        .collect()
}

fn t_lines(src: &str) -> Vec<String> {
    lines(src)
        .into_iter()
        .filter(|m| m.starts_with("T|"))
        .collect()
}

/// `T|<name> r=1 ok=1` for every probe the source displays, in order.
fn all_ok(src: &str) -> Vec<String> {
    src.lines()
        .filter_map(|l| {
            let i = l.find("$display(\"T|")? + "$display(\"".len();
            let name = l[i..].split_whitespace().next()?;
            Some(format!("{name} r=1 ok=1"))
        })
        .collect()
}

#[test]
fn issue_301_reproducer() {
    let out: Vec<String> = lines(include_str!("randomize_issue_301_wide2d.sv"))
        .into_iter()
        .filter(|m| m.starts_with("t_") || m.starts_with("TEST_"))
        .collect();
    let expected = [
        "t_64   2D 8x8=64b    OK",
        "t_72   2D 9x8=72b    OK",
        "t_wide 2D 32x48=1536b OK",
        "t_64   2D 8x8=64b    OK",
        "t_72   2D 9x8=72b    OK",
        "t_wide 2D 32x48=1536b OK",
        "TEST_PASS",
    ];
    assert_eq!(out, expected);
}

#[test]
fn issue_301_shapes() {
    let src = include_str!("randomize_issue_301_shapes.sv");
    let expected = all_ok(src);
    assert_eq!(expected.len(), 123);
    assert_eq!(t_lines(src), expected);
}

#[test]
fn issue_301_selects() {
    let src = include_str!("randomize_issue_301_selects.sv");
    let once = all_ok(src);
    assert_eq!(once.len(), 11);
    let expected: Vec<String> = once.iter().chain(once.iter()).cloned().collect();
    assert_eq!(t_lines(src), expected);
}

#[test]
fn issue_301_member_access() {
    let expected = [
        "T|w 01000000a500000080",
        "T|y a 00a5 0 1 0",
        "T|o 9 de91 2 1 0",
        "T|in 9 de91 2 1 0",
        "T|t2 a 00a5 0 1 1",
        "T|arr 11223344 22 1122 2",
        "T|darr 11223344 22 1122 2",
    ];
    assert_eq!(
        t_lines(include_str!("randomize_issue_301_access.sv")),
        expected
    );
}

/// A failing `randomize()` is reported on stderr, three times per class
/// and constraint set at most; `XEZIM_RAND_WARN=0` silences it.
#[test]
fn issue_301_failure_warning() {
    let file = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/classes/randomize_issue_301_warn.sv");
    let run = |quiet: bool| {
        let mut cmd = Command::new(env!("CARGO_BIN_EXE_xezim"));
        cmd.arg(&file).env_remove("XEZIM_RAND_DIAG");
        if quiet {
            cmd.env("XEZIM_RAND_WARN", "0");
        } else {
            cmd.env_remove("XEZIM_RAND_WARN");
        }
        let o = cmd.output().expect("failed to run xezim");
        (
            String::from_utf8_lossy(&o.stdout).into_owned(),
            String::from_utf8_lossy(&o.stderr).into_owned(),
        )
    };
    let (stdout, stderr) = run(false);
    let t: Vec<&str> = stdout.lines().filter(|l| l.starts_with("T|")).collect();
    assert_eq!(
        t,
        [
            "T|r=0", "T|r=0", "T|r=0", "T|r=0", "T|r=0", "T|p=1", "T|pw=0"
        ]
    );
    let c1 = "[xezim][warning] randomize() failed for class 'c1': its constraints have no \
              solution (lo (randomize_issue_301_warn.sv:5), hi (randomize_issue_301_warn.sv:5)); \
              returned 0";
    let warned: Vec<&str> = stderr.lines().filter(|l| l.starts_with(c1)).collect();
    assert_eq!(warned.len(), 3, "{stderr}");
    assert!(warned[2].ends_with("further failures of this class are not reported"));
    assert!(
        stderr.contains(
            "randomize() failed for class 'c2': its constraints have no solution \
             (ok_c (randomize_issue_301_warn.sv:6), with {...} (randomize_issue_301_warn.sv:11))"
        ),
        "{stderr}"
    );
    let (_, quiet) = run(true);
    assert!(!quiet.contains("randomize() failed"), "{quiet}");
}

/// Found while probing #301: repairing `x != q[i]` against a state queue
/// re-drew the queue's elements instead of `x`, so `randomize()` returned 1
/// with the state changed (§18.3: state variables are never assigned).
#[test]
fn state_queue_element_is_never_redrawn() {
    let src = r#"
module top;
  class p5; rand bit [1:0] x; int q[$] = '{0,1,2};
    constraint c { x != q[0]; x != q[1]; x != q[2]; } endclass
  class p6; rand bit [1:0] x; int q[$] = '{0,1,2};
    constraint c { foreach (q[i]) x != q[i]; } endclass
  initial begin
    p5 a = new(); p6 b = new();
    repeat (3) begin
      int r;
      r = a.randomize(); $display("T|a r=%0d x=%0d q=%p", r, a.x, a.q);
      r = b.randomize(); $display("T|b r=%0d x=%0d q=%p", r, b.x, b.q);
    end
  end
endmodule
"#;
    let mut expected = Vec::new();
    for _ in 0..3 {
        expected.push("T|a r=1 x=3 q='{0, 1, 2}".to_string());
        expected.push("T|b r=1 x=3 q='{0, 1, 2}".to_string());
    }
    assert_eq!(t_lines(src), expected);
}

/// §18.6.3: an unsatisfiable wide relation leaves rand sub-objects unchanged.
#[test]
fn failed_wide_subobject_constraint_preserves_both_objects() {
    let src = r#"
module rollback_probe;
  class fragment;
    rand bit [8:0][7:0] payload;
  endclass
  class pair_model;
    rand fragment left, right;
    bit [0:0] controls;
    constraint conflict {
      foreach (controls[index]) {
        left.payload == right.payload;
        left.payload[8] == 8'h12;
        right.payload[8] == 8'h34;
      }
    }
    function new(); left = new(); right = new(); endfunction
  endclass
  initial begin
    pair_model item = new();
    bit [71:0] saved_left, saved_right;
    int result;
    item.left.payload = 72'h112233445566778899;
    item.right.payload = 72'haabbccddeeff001122;
    saved_left = item.left.payload;
    saved_right = item.right.payload;
    result = item.randomize();
    $display("T|result=%0d preserved=%0d", result,
      item.left.payload === saved_left && item.right.payload === saved_right);
  end
endmodule
"#;
    assert_eq!(t_lines(src), ["T|result=0 preserved=1"]);
}
