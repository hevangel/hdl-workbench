use std::process::Command;

fn run_source(extra: &[&str]) -> std::process::Output {
    let dir = std::env::temp_dir().join(format!("xezim_banner_{}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("create temporary directory");
    let source = dir.join("banner.sv");
    std::fs::write(
        &source,
        "module top; initial begin $display(\"BODY\"); $finish; end endmodule\n",
    )
    .expect("write source");
    let mut command = Command::new(env!("CARGO_BIN_EXE_xezim"));
    command.args(["--simulate", "--no-cache"]);
    command.args(extra);
    command.arg(source);
    let output = command.output().expect("run simulation");
    let _ = std::fs::remove_dir_all(dir);
    output
}

fn assert_banner_starts_stdout(output: &std::process::Output) {
    assert!(
        output.status.success(),
        "simulation failed:\n{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut lines = stdout.lines();
    let expected_version = format!("=== xezim {} ===", env!("CARGO_PKG_VERSION"));
    assert_eq!(
        lines.next(),
        Some(expected_version.as_str()),
        "version must be the first simulation line:\n{stdout}"
    );
    let expected_commit = format!(
        "git {} ({})",
        env!("XEZIM_GIT_HASH"),
        env!("XEZIM_GIT_DATE")
    );
    assert_eq!(
        lines.next(),
        Some(expected_commit.as_str()),
        "commit must be the second simulation line:\n{stdout}"
    );
    assert!(stdout.contains("BODY"), "design did not run:\n{stdout}");
}

#[test]
fn ordinary_simulation_starts_with_build_identity() {
    assert_banner_starts_stdout(&run_source(&[]));
}

#[test]
fn verbose_simulation_does_not_duplicate_build_identity() {
    let output = run_source(&["--verbose"]);
    assert_banner_starts_stdout(&output);
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(stdout.matches("=== xezim ").count(), 1, "{stdout}");
    assert_eq!(stdout.matches("git ").count(), 1, "{stdout}");
}
