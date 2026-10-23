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

/// IEEE 1800 §6.24.1: issue #361, original reproducer.
#[test]
fn issue361_two_state_continuous_cast() {
    check(
        include_str!("../issue_cases/next_issue361_0.sv"),
        &["PASS assign eq0", "PASS assign eq7", "PASS initial"],
    );
}

#[test]
fn cast_neighbors() {
    let source = include_str!("../issue_cases/next_cast_neighbors.sv");
    check(source, &[]);
    let result = simulate(source, 100_000).unwrap();
    assert_eq!(
        result
            .output
            .iter()
            .filter(|entry| entry.message.starts_with("PASS"))
            .count(),
        6,
        "{:?}",
        result.output
    );
}
