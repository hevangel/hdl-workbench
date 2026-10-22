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

/// IEEE 1800 §7.6 / §23.3.3: issue #358, original reproducer.
#[test]
fn issue358_array_port_positions() {
    check(
        include_str!("../issue_cases/next_issue358_0.sv"),
        &["arr0=11 arr1=12 brr1=20 brr2=21"],
    );
}

#[test]
fn port_neighbors() {
    let source = include_str!("../issue_cases/next_port_neighbors.sv");
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
