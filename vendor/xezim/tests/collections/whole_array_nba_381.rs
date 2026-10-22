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

/// IEEE 1800 §7.6 / §10.4.2: issue #381, original reproducer.
#[test]
fn issue381_original() {
    check(
        include_str!("../issue_cases/rest_issue381_0.sv"),
        &[
            "PASS a = 1 2",
            "PASS b = 1 2",
            "PASS c = 1 2",
            "PASS d = 3 4",
        ],
    );
}

/// IEEE 1800 §7.6 / §10.4.2: neighboring cases.
#[test]
fn nba_neighbors() {
    check(
        include_str!("../issue_cases/rest_nba_neighbors.sv"),
        &["PASS NBA neighbors"],
    );
}
