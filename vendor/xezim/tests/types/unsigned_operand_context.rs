//! §11.8.1 / §11.8.2: an unsigned operand makes the whole expression
//! unsigned, and that type reaches every context-determined operand before
//! it is evaluated. `z ? '0 : (a / b)` with signed `a`, `b` divides unsigned
//! because '0 is unsigned (§5.7.1); a comparison against an unsigned value,
//! or a sum with one, does the same to a division or sign extension on the
//! other side. Concatenation members and cast operands stay self-determined.

use std::process::Command;

fn check(mode: &[(&str, &str)]) {
    static SEQ: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);
    // A fresh directory of our own: never one that already exists.
    let dir = loop {
        let dir = std::env::temp_dir().join(format!(
            "xezim_unsigned_operand_{}_{}",
            std::process::id(),
            SEQ.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
        ));
        match std::fs::create_dir(&dir) {
            Ok(()) => break dir,
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(e) => panic!("create {}: {e}", dir.display()),
        }
    };
    let path = dir.join("top.sv");
    std::fs::write(
        &path,
        include_str!("../issue_cases/unsigned_operand_context.sv"),
    )
    .unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_xezim"))
        .args(["--no-cache", "-s", "top"])
        .arg(&path)
        .env_remove("XEZIM_METHOD_TIER")
        .env_remove("XEZIM_COMPILE_METHODS")
        .env_remove("XEZIM_PROC_FSM")
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
}

#[test]
fn unsigned_operand_context() {
    check(&[]);
}

#[test]
fn unsigned_operand_context_eager() {
    check(&[("XEZIM_METHOD_TIER", "0")]);
}

#[test]
fn unsigned_operand_context_interpreted() {
    check(&[("XEZIM_COMPILE_METHODS", "0"), ("XEZIM_PROC_FSM", "0")]);
}
