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

/// IEEE 1800 §27.4: issue #374, original reproducer.
#[test]
fn issue374_original() {
    check(include_str!("../issue_cases/rest_issue374_0.sv"), &["PASS"]);
}
