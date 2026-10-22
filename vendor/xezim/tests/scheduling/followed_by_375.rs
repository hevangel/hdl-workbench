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

/// IEEE 1800 §16.12.9: issue #375, original reproducer.
#[test]
fn issue375_original() {
    check(include_str!("../issue_cases/rest_issue375_0.sv"), &["PASS"]);
}

/// IEEE 1800 §16.12.9: neighboring cases.
#[test]
fn follow_neighbors() {
    check(
        include_str!("../issue_cases/rest_follow_neighbors.sv"),
        &["PASS followed neighbors"],
    );
}

/// IEEE 1800 §16.12.9: neighboring cases.
#[test]
fn follow_range_neighbors() {
    check(
        include_str!("../issue_cases/rest_follow_range_neighbors.sv"),
        &["PASS followed range neighbors"],
    );
}
