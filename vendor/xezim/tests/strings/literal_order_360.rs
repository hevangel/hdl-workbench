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

/// IEEE 1800 §5.9 / §11.10: issue #360, original reproducer.
#[test]
fn issue360_integral_string_literals() {
    check(include_str!("../issue_cases/next_issue360_0.sv"), &["PASS"]);
}

#[test]
fn literal_neighbors() {
    let source = include_str!("../issue_cases/next_literal_neighbors.sv");
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
