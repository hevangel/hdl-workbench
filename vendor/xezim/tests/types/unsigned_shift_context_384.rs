//! §11.8.2 / §11.4.10: unsigned expression type reaches shift operands
//! before widening. Casts, concatenations and shift counts stay self-determined.

use std::process::Command;

fn check(source: &str, mode: &[(&str, &str)], relax: bool) -> String {
    static SEQ: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);
    let dir = std::env::temp_dir().join(format!(
        "xezim_shift_384_{}_{}",
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

const SOURCE: &str = include_str!("../issue_cases/unsigned_shift_context_384.sv");
#[test]
fn unsigned_shift_context() {
    check(SOURCE, &[], false);
}
#[test]
fn unsigned_shift_context_eager() {
    check(SOURCE, &[("XEZIM_METHOD_TIER", "0")], false);
}
#[test]
fn unsigned_shift_context_interpreted() {
    check(
        SOURCE,
        &[("XEZIM_COMPILE_METHODS", "0"), ("XEZIM_PROC_FSM", "0")],
        false,
    );
}

#[test]
fn unsigned_shift_original_four_paths() {
    let result = xezim::simulate(
        include_str!("../issue_cases/unsigned_shift_original_384.sv"),
        100,
    )
    .expect("unsigned shift reproducer");
    assert!(
        result.output.iter().any(|line| line.message == "PASS"),
        "{:?}",
        result.output
    );
    assert!(
        !result
            .output
            .iter()
            .any(|line| line.message.contains("FAIL"))
    );
}

const SIDE_EFFECTS: &str = include_str!("../issue_cases/unsigned_shift_side_effects_384.sv");
#[test]
fn shift_type_probes_preserve_side_effects() {
    check(SIDE_EFFECTS, &[], false);
}
#[test]
fn shift_type_probes_preserve_side_effects_eager() {
    check(SIDE_EFFECTS, &[("XEZIM_METHOD_TIER", "0")], false);
}
#[test]
fn shift_type_probes_preserve_side_effects_interpreted() {
    check(
        SIDE_EFFECTS,
        &[("XEZIM_COMPILE_METHODS", "0"), ("XEZIM_PROC_FSM", "0")],
        false,
    );
}
