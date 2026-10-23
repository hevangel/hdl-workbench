//! §6.12.2: a real converted to an integral type wraps to the target width in
//! every conversion path, not only an assignment: the generic resize (a
//! negated power of two into a 128-bit variable kept a saturated 64-bit
//! value) and a parameter of integral type (a 64-bit `longint` parameter
//! stayed real; wider ones saturated). The expected lines are the reference
//! simulator's.

use std::process::Command;

/// Run `src` (top module `top`) and return its display lines: everything on
/// stdout but the banner and the end-of-run note.
fn run_lines(tag: &str, src: &str, env: &[(&str, &str)]) -> Vec<String> {
    let dir = std::env::temp_dir().join(format!(
        "xezim_{}_{}_{}",
        "realwide",
        tag,
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let sv = dir.join("top.sv");
    std::fs::write(&sv, src).unwrap();
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_xezim"));
    for (k, v) in env {
        cmd.env(k, v);
    }
    let out = cmd
        .args(["--no-cache", "-s", "top"])
        .arg(&sv)
        .current_dir(&dir)
        .output()
        .expect("failed to run xezim");
    let _ = std::fs::remove_dir_all(&dir);
    let text = String::from_utf8_lossy(&out.stdout).into_owned();
    assert!(
        out.status.success(),
        "{text}{}",
        String::from_utf8_lossy(&out.stderr)
    );
    text.lines()
        .filter(|l| {
            !(l.starts_with('[')
                || l.starts_with("=== xezim")
                || l.starts_with("git ")
                || l.starts_with("Simulation finished")
                || l.starts_with("$finish"))
        })
        .map(str::to_string)
        .collect()
}

/// Both method tiers: interpreted, and compiled on the first call.
fn check(tag: &str, src: &str, want: &[&str]) {
    assert_eq!(run_lines(tag, src, &[]), want, "{tag}");
    assert_eq!(
        run_lines(&format!("{tag}_t0"), src, &[("XEZIM_METHOD_TIER", "0")]),
        want,
        "{tag} (XEZIM_METHOD_TIER=0)"
    );
}

#[test]
fn wide_targets_and_parameters() {
    let src = r#"module sub #(parameter bit [127:0] Q = 0, parameter longint L = 0) ();
  initial $display("T|sub Q=%h L=%0d", Q, L);
endmodule
module top;
  localparam bit [127:0] P1 = 2.0**100;
  localparam bit [127:0] P2 = -(2.0**100);
  localparam longint P3 = 2.0**64;
  localparam int P4 = 1.0e30;
  localparam bit [79:0] P5 = 2.0**70 + 2.0**10;
  localparam longint P6 = -(2.0**63) - 4096.0;
  bit [127:0] w; logic [95:0] w96; real r;
  sub #(.Q(-(2.0**100)), .L(1.0e30)) u ();
  initial begin
    $display("T|P1=%h", P1);
    $display("T|P2=%h", P2);
    $display("T|P3=%0d P4=%0d", P3, P4);
    $display("T|P5=%h P6=%0d", P5, P6);
    w = -(2.0**100);  $display("T|w -2^100 = %h", w);
    w = -1.0e30;      $display("T|w -1e30 = %h", w);
    r = -(2.0**70) - 2.0**20; w96 = r; $display("T|w96 = %h", w96);
  end
endmodule
"#;
    check(
        "wide_targets_and_parameters",
        src,
        &[
            "T|sub Q=fffffff0000000000000000000000000 L=5076964154930102272",
            "T|P1=00000010000000000000000000000000",
            "T|P2=fffffff0000000000000000000000000",
            "T|P3=0 P4=0",
            "T|P5=00400000000000000000 P6=9223372036854771712",
            "T|w -2^100 = fffffff0000000000000000000000000",
            "T|w -1e30 = fffffff360d3632fb98b000000000000",
            "T|w96 = ffffffbffffffffffff00000",
        ],
    );
}
