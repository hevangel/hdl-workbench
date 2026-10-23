#![cfg(target_os = "linux")]

use std::path::Path;
use std::process::Command;

fn check_case(file: &str) {
    let source = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/issue_cases")
        .join(file);
    // A regressed size solve must fail within a bounded time and memory budget.
    let result = Command::new("bash")
        .args([
            "-c",
            "ulimit -v 2097152 || exit 1; exec timeout --kill-after=2s 30s \"$1\" \"$2\"",
            "constraint-check",
        ])
        .arg(env!("CARGO_BIN_EXE_xezim"))
        .arg(source)
        .env("XEZIM_STACK_MB", "64")
        .output()
        .expect("run constraint regression");
    let output = format!(
        "{}{}",
        String::from_utf8_lossy(&result.stdout),
        String::from_utf8_lossy(&result.stderr)
    );
    assert!(
        result.status.success(),
        "{file}: {}\n{output}",
        result.status
    );
    assert!(!output.contains("TEST_FAIL"), "{file}: {output}");
    assert!(output.contains("TEST_PASS"), "{file}: {output}");
}

#[test]
fn issue_252_integral_elements_with_real_bounds() {
    check_case("constraint_252.sv");
}

#[test]
fn issue_253_four_state_wide_implication() {
    check_case("constraint_253.sv");
}

#[test]
fn issue_254_joint_sizes_sum_and_integer_size() {
    check_case("constraint_254.sv");
}

#[test]
fn issue_302_sum_window_rand_bounds_budget() {
    check_case("constraint_302.sv");
}
