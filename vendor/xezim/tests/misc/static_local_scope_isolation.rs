//! §13.4.2 lifetime rules for `static` subroutine locals: a static local is
//! ONE cell shared across calls of ITS OWN subroutine, and nothing else. A
//! same-named plain (automatic) local in a DIFFERENT subroutine is a distinct
//! variable, even when the other subroutine is on the current call stack.
//!
//! The failure this pins is silent and bidirectional: bind the two together
//! and an inner routine both READS the ancestor's static instead of its own
//! initialiser and CORRUPTS that static on write, so results drift call over
//! call. Names like `cnt`/`i`/`idx` collide constantly, so nothing about the
//! source looks wrong. It is reachable from any implementation that resolves
//! a static local by NAME across open call frames rather than by the frame
//! that declared it.
//!
//! Every expected value here is reference-verified.

use std::process::Command;

fn run(name: &str, src: &str) -> String {
    // Unique per call: tests run as parallel threads of ONE process, so a
    // pid-only directory name lets siblings clobber each other's source.
    static SEQ: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);
    let dir = std::env::temp_dir().join(format!(
        "xezim_statloc_{}_{}_{}",
        name,
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
        .output()
        .expect("run xezim");
    let mut text = String::from_utf8_lossy(&out.stdout).to_string();
    text.push_str(&String::from_utf8_lossy(&out.stderr));
    let _ = std::fs::remove_dir_all(&dir);
    text
}

#[test]
fn static_local_is_isolated_from_same_named_locals_elsewhere() {
    let text = run(
        "scope",
        r#"module top;
  // A: an inner subroutine's AUTOMATIC local must not bind to an ancestor's
  //    same-named static, in either direction. `a_inner` must return 100
  //    (99+1, its own initialiser) and must not disturb `a_outer`'s cell.
  function automatic int a_inner();
    int cnt = 99;
    cnt = cnt + 1;
    return cnt;
  endfunction
  function automatic int a_outer();
    static int cnt = 5;
    return a_inner() * 1000 + cnt;
  endfunction

  // B: two sibling subroutines with same-named statics stay independent.
  function automatic int b_one(); static int s = 10; s = s + 1; return s; endfunction
  function automatic int b_two(); static int s = 20; s = s + 2; return s; endfunction

  // C: a static local does persist across calls of its own subroutine —
  //    the isolation above must not be achieved by giving up sharing.
  function automatic int c_acc(); static int n = 0; n = n + 3; return n; endfunction

  // D/E: a TASK's static versus a function's same-named automatic local.
  int d_seen;
  task automatic d_task(); static int v = 7; v = v + 1; d_seen = v; endtask
  function automatic int d_fn(); int v = 100; v = v + 1; return v; endfunction

  initial begin
    $display("A %0d %0d", a_outer(), a_outer());
    $display("B %0d %0d %0d %0d", b_one(), b_two(), b_one(), b_two());
    $display("C %0d %0d %0d", c_acc(), c_acc(), c_acc());
    d_task(); $display("D %0d %0d", d_fn(), d_seen);
    d_task(); $display("E %0d %0d", d_fn(), d_seen);
  end
endmodule
"#,
    );
    for expect in [
        // a_inner returns 100 both times; a_outer's static stays 5.
        "A 100005 100005",
        "B 11 22 12 24",
        "C 3 6 9",
        "D 101 8",
        "E 101 9",
    ] {
        assert!(text.contains(expect), "missing `{expect}` in:\n{text}");
    }
}

/// §13.4.2: recursion does NOT give each activation its own copy of a static
/// local — all activations share the one cell, so the counter below reaches
/// the recursion depth. xezim currently gives each activation a fresh cell
/// and returns 1. Reference-verified expectation: 4.
///
#[test]
fn recursive_activations_share_one_static_cell() {
    let text = run(
        "recur",
        r#"module top;
  function automatic int rec(int n);
    static int depth = 0;
    depth = depth + 1;
    if (n > 0) return rec(n - 1);
    return depth;
  endfunction
  initial $display("R %0d", rec(3));
endmodule
"#,
    );
    assert!(
        text.contains("R 4"),
        "recursive static did not share one cell:\n{text}"
    );
}
