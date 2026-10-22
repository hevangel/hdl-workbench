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

/// IEEE 1800 §16.7 / §16.9.2: issue #367, original reproducer.
#[test]
fn issue367_large_sequence_counts() {
    check(
        include_str!("../issue_cases/next_issue367_0.sv"),
        &["RESULT r299=1 r300=1 r301=0 d300=1 d5000=1"],
    );
}

#[test]
fn sequence_counter_neighbors() {
    check(
        include_str!("../issue_cases/next_count_neighbors.sv"),
        &[
            "PASS repetition boundaries",
            "PASS delay boundaries",
            "PASS goto and nonconsecutive",
            "PASS live attempt preservation",
        ],
    );
}
