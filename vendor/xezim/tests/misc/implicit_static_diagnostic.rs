//! §6.21: an initializer on an implicitly-static subroutine variable is an
//! error — but the message has to say WHERE. A user hitting the rule on a
//! large multi-file design was told only the variable name ("Variable 'MIN'
//! is implicitly static"), with no file, line, or enclosing subroutine, and
//! had no way to find the declaration.
//!
//! Also pins the escape hatch: real designs carry the pattern and other
//! simulators let it be suppressed, so `--relax-implicit-static` (env
//! `XEZIM_ALLOW_IMPLICIT_STATIC=1`) downgrades it to a warning.

use std::process::Command;

const PKG: &str = r#"package math_pkg;
  function int clamp_lo(int a, int b);
    int MIN = (a < b) ? a : b;
    return MIN;
  endfunction
endpackage
"#;

const TOP: &str = r#"module top;
  import math_pkg::*;
  initial $display("T|%0d", clamp_lo(3, 5));
endmodule
"#;

fn write_case(dir: &std::path::Path, pkg: &str) -> (String, String) {
    let p = dir.join("math_pkg.sv");
    let t = dir.join("top.sv");
    std::fs::write(&p, pkg).unwrap();
    std::fs::write(&t, TOP).unwrap();
    (p.to_string_lossy().into(), t.to_string_lossy().into())
}

fn run(args: &[&str], relax_env: bool) -> (String, bool) {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_xezim"));
    cmd.args(args);
    if relax_env {
        cmd.env("XEZIM_ALLOW_IMPLICIT_STATIC", "1");
    }
    let out = cmd.output().expect("run xezim");
    let mut text = String::from_utf8_lossy(&out.stdout).to_string();
    text.push_str(&String::from_utf8_lossy(&out.stderr));
    (text, out.status.success())
}

#[test]
fn implicit_static_error_names_file_line_and_subroutine() {
    let dir = tempdir("istatic_err");
    let (pkg, top) = write_case(&dir, PKG);
    let (text, ok) = run(
        &["--simulate", "-s", "top", &pkg, &top, "--no-cache"],
        false,
    );
    assert!(!ok, "must fail:\n{}", text);
    assert!(
        text.contains("Variable 'MIN' is implicitly static"),
        "keeps the §6.21 wording:\n{}",
        text
    );
    // The whole point: the message locates the declaration.
    assert!(
        text.contains("in function 'clamp_lo' of package 'math_pkg'"),
        "must name the enclosing subroutine and its scope:\n{}",
        text
    );
    assert!(
        text.contains("math_pkg.sv:3"),
        "must name the file and line of the declaration:\n{}",
        text
    );
    assert!(
        text.contains("--relax-implicit-static"),
        "must point at the escape hatch:\n{}",
        text
    );
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn relax_flag_and_env_downgrade_to_warning() {
    let dir = tempdir("istatic_relax");
    let (pkg, top) = write_case(&dir, PKG);

    let (text, ok) = run(
        &[
            "--simulate",
            "-s",
            "top",
            &pkg,
            &top,
            "--no-cache",
            "--relax-implicit-static",
        ],
        false,
    );
    assert!(
        ok,
        "--relax-implicit-static must let the run proceed:\n{}",
        text
    );
    assert!(
        text.contains("T|0"),
        "static initialization precedes the call:\n{}",
        text
    );
    assert!(
        text.contains("[xezim][warning]") && text.contains("implicitly static"),
        "downgraded to a warning that still names the variable:\n{}",
        text
    );

    assert!(
        text.contains("in function 'clamp_lo' of package 'math_pkg'")
            && text.contains("math_pkg.sv:3"),
        "the warning must locate the declaration:\n{}",
        text
    );

    let (text, ok) = run(&["--simulate", "-s", "top", &pkg, &top, "--no-cache"], true);
    assert!(
        ok,
        "XEZIM_ALLOW_IMPLICIT_STATIC=1 must do the same:\n{}",
        text
    );
    assert!(
        text.contains("T|0"),
        "static initialization precedes the call under the env knob:\n{}",
        text
    );
    assert!(
        text.contains("in function 'clamp_lo' of package 'math_pkg'")
            && text.contains("math_pkg.sv:3"),
        "the warning must locate the declaration:\n{}",
        text
    );
    let _ = std::fs::remove_dir_all(&dir);
}

/// An explicitly `automatic` subroutine is legal and must stay silent.
#[test]
fn automatic_subroutine_is_not_flagged() {
    let dir = tempdir("istatic_auto");
    let (pkg, top) = write_case(
        &dir,
        &PKG.replace("function int clamp_lo", "function automatic int clamp_lo"),
    );
    let (text, ok) = run(
        &["--simulate", "-s", "top", &pkg, &top, "--no-cache"],
        false,
    );
    assert!(ok, "automatic lifetime is legal:\n{}", text);
    assert!(text.contains("T|3"), "runs:\n{}", text);
    assert!(
        !text.contains("implicitly static"),
        "no §6.21 diagnostic for an automatic subroutine:\n{}",
        text
    );
    let _ = std::fs::remove_dir_all(&dir);
}

fn tempdir(tag: &str) -> std::path::PathBuf {
    let d = std::env::temp_dir().join(format!("xezim_{}_{}", tag, std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).unwrap();
    d
}

/// §6.20.4: a block-scope `localparam`/`parameter` inside a static task is a
/// CONSTANT, not a variable — §6.21 does not apply and it must not be
/// rejected (a customer testbench's task-body `localparam int MIN = 24;`
/// was; the reference runs it). Reference output: min=24 max=42.
#[test]
fn task_body_localparam_is_not_implicitly_static() {
    let dir = tempdir("istatic_lp");
    let src = dir.join("lp.sv");
    std::fs::write(
        &src,
        r#"module testbench;
  task drive_regs;
    localparam int MIN = 24;
    parameter  int MAX = 42;
    $display("T|min=%0d max=%0d", MIN, MAX);
  endtask
  initial drive_regs;
endmodule
"#,
    )
    .unwrap();
    let (text, ok) = run(
        &[
            "--simulate",
            "-s",
            "testbench",
            src.to_str().unwrap(),
            "--no-cache",
        ],
        false,
    );
    assert!(ok, "task-body localparam is legal:\n{}", text);
    assert!(
        text.contains("T|min=24 max=42"),
        "constants read back:\n{}",
        text
    );
    assert!(
        !text.contains("implicitly static"),
        "no §6.21 diagnostic for a constant:\n{}",
        text
    );
    let _ = std::fs::remove_dir_all(&dir);
}
