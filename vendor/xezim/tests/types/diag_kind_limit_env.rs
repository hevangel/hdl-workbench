//! `XEZIM_DIAG_LIMIT` overrides the per-kind elaboration diagnostic cap.
//!
//! The cap keeps a noisy design readable, but it hides exactly the messages
//! you reach for when a whole CLASS of things is wrong: five port width
//! mismatches followed by "further messages suppressed" reads as "there were
//! five", and the count that would have told you it was systemic is gone.
//!
//! Runs the binary in a subprocess: the limit is read through a `OnceLock`, so
//! it is fixed for the life of a process and an in-process test could not vary
//! it (and would race the rest of the suite, which runs in parallel).

use std::process::Command;

/// Eight DISTINCT ports, each connected to a narrower actual — the cap groups
/// by message KIND, and a repeat of the same (module, port) pair is folded
/// separately, so eight ports are needed to produce eight diagnostics.
const SRC: &str = r#"
module sink(input logic [15:0] a, b, c, d, e, f, g, h);
  initial #1;
endmodule
module top;
  logic [3:0] w0, w1, w2, w3, w4, w5, w6, w7;
  sink u(.a(w0), .b(w1), .c(w2), .d(w3), .e(w4), .f(w5), .g(w6), .h(w7));
  initial #2 $finish;
endmodule
"#;

/// Returns (mismatch_count, suppression_note_count) from stderr.
fn run(limit: Option<&str>) -> (usize, usize) {
    // A UNIQUE path per invocation. The tests in this file run in parallel and
    // each spawns a subprocess, so a shared path lets one test truncate the
    // file while another's subprocess is reading it — which shows up as a
    // wrong diagnostic count on a slow machine and passes on a fast one.
    static SEQ: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
    let n = SEQ.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let dir = std::env::temp_dir().join(format!("xezim_diaglim_{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join(format!("diaglim_{}.sv", n));
    std::fs::write(&path, SRC).unwrap();
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_xezim"));
    cmd.arg("--simulate")
        .arg("-s")
        .arg("top")
        .arg(path.to_str().unwrap());
    match limit {
        Some(v) => {
            cmd.env("XEZIM_DIAG_LIMIT", v);
        }
        // Must be REMOVED, not set empty: an ambient value in the developer's
        // shell would otherwise silently retune the default case.
        None => {
            cmd.env_remove("XEZIM_DIAG_LIMIT");
        }
    }
    let out = cmd.output().expect("failed to run xezim");
    let err = String::from_utf8_lossy(&out.stderr);
    (
        err.matches("port width mismatch").count(),
        err.matches("further messages of this kind are suppressed")
            .count(),
    )
}

#[test]
fn default_caps_at_five_and_says_how_to_see_the_rest() {
    let (n, notes) = run(None);
    assert_eq!(n, 5, "default per-kind cap");
    assert_eq!(notes, 1, "exactly one suppression note");
}

#[test]
fn an_explicit_limit_raises_the_cap() {
    assert_eq!(run(Some("8")).0, 8, "all eight reported at limit 8");
    assert_eq!(run(Some("2")).0, 2, "limit 2 reports two");
}

#[test]
fn zero_means_unlimited_and_emits_no_suppression_note() {
    let (n, notes) = run(Some("0"));
    assert_eq!(n, 8, "every diagnostic reported");
    assert_eq!(notes, 0, "nothing was suppressed, so nothing to note");
}

#[test]
fn an_unparseable_value_keeps_the_default() {
    // A diagnostic setting must never fail a simulation run.
    assert_eq!(run(Some("banana")).0, 5);
    assert_eq!(run(Some("")).0, 5);
}
