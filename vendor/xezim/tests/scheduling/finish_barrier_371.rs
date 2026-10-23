//! Regression coverage for expression, collection, and scheduling semantics.
use xezim::simulate;
fn check(source: &str, expected: &[&str]) {
    let result = simulate(source, 100_000).unwrap();
    let lines: Vec<_> = result
        .output
        .iter()
        .map(|entry| entry.message.as_str())
        .collect();
    assert!(
        !lines.iter().any(|line| line.starts_with("FAIL")),
        "{lines:?}"
    );
    for line in expected {
        assert!(lines.contains(line), "missing {line:?}: {lines:?}");
    }
}

/// IEEE 1800 §20.2 / §38.4: issue #371, original reproducer.
#[test]
fn issue371_finish_discards_slot_nba() {
    check(
        include_str!("../issue_cases/next_issue371_0.sv"),
        &["PASS q=2"],
    );
}

/// IEEE 1800 §20.2: issue #372, original reproducer.
#[test]
fn issue372_finish_stops_bytecode() {
    check(
        include_str!("../issue_cases/next_issue372_0.sv"),
        &["PASS n=2"],
    );
}

#[test]
fn finish_initial_stops_remaining_statements() {
    check(
        include_str!("../issue_cases/next_issue372_1.sv"),
        &["initial: n=0 (expect 0)"],
    );
}

#[test]
fn finish_neighbors() {
    let source = include_str!("../issue_cases/next_finish_neighbors.sv");
    check(source, &[]);
    let result = simulate(source, 100_000).unwrap();
    assert_eq!(
        result
            .output
            .iter()
            .filter(|entry| entry.message.starts_with("PASS"))
            .count(),
        1,
        "{:?}",
        result.output
    );
}

#[test]
fn finish_barrier_neighbor_0() {
    check(
        include_str!("../issue_cases/next_finish_case0.sv"),
        &["PASS finish barrier"],
    );
}

#[test]
fn finish_barrier_neighbor_1() {
    check(
        include_str!("../issue_cases/next_finish_case1.sv"),
        &["PASS finish barrier"],
    );
}

#[test]
fn finish_barrier_neighbor_2() {
    check(
        include_str!("../issue_cases/next_finish_case2.sv"),
        &["PASS finish barrier"],
    );
}

#[test]
fn finish_barrier_neighbor_3() {
    check(
        include_str!("../issue_cases/next_finish_case3.sv"),
        &["PASS finish barrier"],
    );
}

#[test]
fn finish_barrier_neighbor_4() {
    check(
        include_str!("../issue_cases/next_finish_case4.sv"),
        &["PASS finish barrier"],
    );
}

/// §20.2: native code must stop after a terminating interpreter bridge.
#[cfg(feature = "jit")]
#[test]
fn native_finish_stops_following_assignment() {
    use std::process::Command;
    let dir = std::env::temp_dir().join(format!("xezim_native_finish_{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let source = dir.join("top.sv");
    std::fs::write(&source, include_str!("../issue_cases/next_issue372_0.sv")).unwrap();
    let result = Command::new(env!("CARGO_BIN_EXE_xezim"))
        .args(["--no-cache", "-s", "top"])
        .arg(&source)
        .env("XEZIM_JIT", "1")
        .env("XEZIM_JIT_VERBOSE", "1")
        .env("XEZIM_TWO_STATE", "0")
        .env_remove("XEZIM_AOT")
        .output()
        .unwrap();
    let text = format!(
        "{}{}",
        String::from_utf8_lossy(&result.stdout),
        String::from_utf8_lossy(&result.stderr)
    );
    let _ = std::fs::remove_dir_all(dir);
    assert!(result.status.success(), "{text}");
    assert!(
        text.contains("compiled 1/1 edge blocks"),
        "native path was not exercised: {text}"
    );
    assert!(
        text.contains("PASS n=2") && !text.contains("FAIL"),
        "{text}"
    );
}
