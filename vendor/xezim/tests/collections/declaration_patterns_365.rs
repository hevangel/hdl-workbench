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

/// IEEE 1800 §10.9.1: issue #365, original reproducer.
#[test]
fn issue365_default_declaration_initializer() {
    check(
        include_str!("../issue_cases/next_issue365_0.sv"),
        &["a=7 7 7 7", "b=7 7 7 7", "c=7 7 7 7", "d=7 7 7 7"],
    );
}

#[test]
fn init_neighbors() {
    let source = include_str!("../issue_cases/next_init_neighbors.sv");
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
