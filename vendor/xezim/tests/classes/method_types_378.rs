//! IEEE 1800 regression coverage for the remaining issue reports.
fn check(source: &str, expected: &[&str]) {
    let result = xezim::simulate(source, 100_000).unwrap();
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

/// IEEE 1800 §8.25: issue #378, original reproducer.
#[test]
fn issue378_original() {
    check(include_str!("../issue_cases/rest_issue378_0.sv"), &["PASS"]);
}

/// IEEE 1800 §8.25: neighboring cases.
#[test]
fn method_neighbors() {
    check(
        include_str!("../issue_cases/rest_method_neighbors.sv"),
        &["PASS method neighbors"],
    );
}
