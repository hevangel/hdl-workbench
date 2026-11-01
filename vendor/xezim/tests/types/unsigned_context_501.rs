//! §11.8.1/§11.8.2: propagate unsigned type before evaluating nested operands.

use std::process::Command;

fn check(source: &str, mode: &[(&str, &str)], relax: bool) -> String {
    static SEQ: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);
    let dir = std::env::temp_dir().join(format!(
        "xezim_unsigned_501_{}_{}",
        std::process::id(),
        SEQ.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
    ));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("top.sv");
    std::fs::write(&path, source).unwrap();
    let mut command = Command::new(env!("CARGO_BIN_EXE_xezim"));
    command
        .args(["--no-cache", "-s", "top"])
        .arg(&path)
        .env_remove("XEZIM_ALLOW_IMPLICIT_STATIC")
        .env_remove("XEZIM_METHOD_TIER")
        .env_remove("XEZIM_COMPILE_METHODS");
    if relax {
        command.arg("--relax-implicit-static");
    }
    let output = command
        .envs(mode.iter().copied())
        .output()
        .expect("run simulator");
    std::fs::remove_dir_all(&dir).unwrap();
    let text = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(output.status.success(), "{text}");
    assert!(
        text.contains("ALL PASS") && !text.contains("FAIL"),
        "{text}"
    );
    text
}

fn run_cases(mode: &[(&str, &str)]) {
    check(include_str!("../issue_cases/issue501.sv"), mode, false);
    check(
        include_str!("../issue_cases/unsigned_context_neighbors_501.sv"),
        mode,
        false,
    );
}

#[test]
fn unsigned_context_original_and_neighbors() {
    run_cases(&[]);
}

#[test]
fn unsigned_context_original_and_neighbors_eager() {
    run_cases(&[("XEZIM_METHOD_TIER", "0")]);
}

#[test]
fn unsigned_context_original_and_neighbors_interpreted() {
    run_cases(&[("XEZIM_COMPILE_METHODS", "0"), ("XEZIM_PROC_FSM", "0")]);
}
