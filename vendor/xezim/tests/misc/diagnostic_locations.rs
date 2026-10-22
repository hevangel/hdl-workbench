//! Parse and elaboration errors print `file:line:col` of the ORIGINAL source
//! (the `include`d file a line came from, not the file that included it),
//! the source line, and a caret under the offending text. Simulation-mode
//! stdout starts with the build identity even when parsing or elaboration
//! fails; diagnostics themselves remain isolated on stderr.

use std::path::{Path, PathBuf};
use std::process::Command;

fn case_dir(name: &str) -> PathBuf {
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
        .join("diagnostic_locations")
        .join(name);
    std::fs::create_dir_all(&dir).expect("mkdir");
    dir
}

/// Run `xezim top.sv` inside `dir` (so reported paths are relative); returns
/// (exit code, stdout, stderr).
fn run(dir: &Path) -> (i32, String, String) {
    let out = Command::new(env!("CARGO_BIN_EXE_xezim"))
        .current_dir(dir)
        .args(["--no-cache", "top.sv"])
        .output()
        .expect("run xezim");
    (
        out.status.code().unwrap_or(-1),
        String::from_utf8_lossy(&out.stdout).into_owned(),
        String::from_utf8_lossy(&out.stderr).into_owned(),
    )
}

fn banner() -> String {
    format!(
        "=== xezim {} ===\ngit {} ({})\n",
        env!("CARGO_PKG_VERSION"),
        env!("XEZIM_GIT_HASH"),
        env!("XEZIM_GIT_DATE")
    )
}

#[test]
fn syntax_error_in_top_file() {
    let dir = case_dir("top");
    std::fs::write(
        dir.join("top.sv"),
        "module top;\n  logic [7:0] a;\n  initial begin\n    a = 8'h1\n    $display(\"a=%0d\", a);\n  end\nendmodule\n",
    )
    .unwrap();
    let (code, stdout, stderr) = run(&dir);
    assert_eq!(code, 1);
    assert_eq!(stdout, banner());
    assert_eq!(
        stderr,
        concat!(
            "Parse errors in 'top.sv' (file 1 of 1):\n",
            "top.sv:5:5: error: expected Semicolon, found SystemIdentifier '$display'\n",
            "    5 |     $display(\"a=%0d\", a);\n",
            "      |     ^~~~~~~~\n",
        )
    );
}

#[test]
fn syntax_error_in_included_file() {
    let dir = case_dir("include");
    std::fs::write(dir.join("body.svh"), "wire w;\nassign w = 1 +;\n").unwrap();
    std::fs::write(
        dir.join("top.sv"),
        "module top;\n`include \"body.svh\"\nendmodule\n",
    )
    .unwrap();
    let (code, stdout, stderr) = run(&dir);
    assert_eq!(code, 1);
    assert_eq!(stdout, banner());
    // The recovery error after the include is back in top.sv, on its own line.
    assert_eq!(
        stderr,
        concat!(
            "Parse errors in 'top.sv' (file 1 of 1):\n",
            "In file included from top.sv:2:\n",
            "body.svh:2:15: error: expected expression, found Semicolon ';'\n",
            "    2 | assign w = 1 +;\n",
            "      |               ^\n",
            "top.sv:3:1: error: expected Semicolon, found KwEndmodule 'endmodule'\n",
            "    3 | endmodule\n",
            "      | ^~~~~~~~~\n",
        )
    );
}

#[test]
fn undeclared_identifier_at_elaboration() {
    let dir = case_dir("undeclared");
    std::fs::write(
        dir.join("top.sv"),
        "module top;\n  logic [7:0] a;\n  initial begin\n    a = bogus_sig + 1;\n    $display(\"a=%0d\", a);\n  end\nendmodule\n",
    )
    .unwrap();
    let (code, stdout, stderr) = run(&dir);
    assert_eq!(code, 1);
    assert_eq!(stdout, banner());
    assert_eq!(
        stderr,
        concat!(
            "top.sv:4:9: error: Undeclared identifier 'bogus_sig'\n",
            "    4 |     a = bogus_sig + 1;\n",
            "      |         ^~~~~~~~~\n",
        )
    );
}

#[test]
fn undeclared_identifier_in_included_file() {
    let dir = case_dir("undeclared_include");
    std::fs::write(
        dir.join("body.svh"),
        "  initial begin\n    missing_var = 1;\n  end\n",
    )
    .unwrap();
    std::fs::write(
        dir.join("top.sv"),
        "module top;\n  logic a;\n`include \"body.svh\"\nendmodule\n",
    )
    .unwrap();
    let (code, stdout, stderr) = run(&dir);
    assert_eq!(code, 1);
    assert_eq!(stdout, banner());
    assert_eq!(
        stderr,
        concat!(
            "In file included from top.sv:3:\n",
            "body.svh:2:5: error: Undeclared identifier 'missing_var'\n",
            "    2 |     missing_var = 1;\n",
            "      |     ^~~~~~~~~~~\n",
        )
    );
}

#[test]
fn clean_run_starts_with_identity_then_prints_design_and_result() {
    let dir = case_dir("clean");
    std::fs::write(
        dir.join("top.sv"),
        "module top;\n  initial begin\n    #5 $display(\"hello\");\n    $finish;\n  end\nendmodule\n",
    )
    .unwrap();
    let (code, stdout, stderr) = run(&dir);
    assert_eq!(code, 0);
    assert_eq!(
        stdout,
        format!(
            "{}hello\nSimulation finished at time 5 ($finish called)\n",
            banner()
        )
    );
    assert_eq!(stderr, "");
}
