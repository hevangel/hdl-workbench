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

/// IEEE 1800 §16.9.3: issue #363, original reproducer.
#[test]
fn issue363_sampled_explicit_clock() {
    check(include_str!("../issue_cases/next_issue363_0.sv"), &["PASS"]);
}

/// IEEE 1800 §16.9.3: issue #364, original reproducer.
#[test]
fn issue364_action_past_history() {
    check(
        include_str!("../issue_cases/next_issue364_0.sv"),
        &[
            "t=5 v=0 past(v)=0",
            "t=15 v=3 past(v)=0",
            "t=25 v=2 past(v)=3",
            "t=35 v=13 past(v)=2",
            "t=45 v=4 past(v)=13",
        ],
    );
}

/// IEEE 1800 §16.9.3: issue #366, original reproducer.
#[test]
fn issue366_past_gating() {
    check(
        include_str!("../issue_cases/next_issue366_0.sv"),
        &["RESULT h1=26 f1=0 h3=22 f3=0"],
    );
}

/// IEEE 1800 §16.5.1: issue #370, original reproducer.
#[test]
fn issue370_preponed_struct_member() {
    check(include_str!("../issue_cases/next_issue370_0.sv"), &["DONE"]);
}

#[test]
fn sample_neighbors() {
    let source = include_str!("../issue_cases/next_sample_neighbors.sv");
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
fn preponed_neighbors() {
    let source = include_str!("../issue_cases/next_preponed_neighbors.sv");
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
fn explicit_clock_neighbors() {
    check(
        include_str!("../issue_cases/next_clock_neighbors.sv"),
        &["PASS explicit clock neighbors"],
    );
}

#[test]
fn action_history_neighbors() {
    check(
        include_str!("../issue_cases/next_action_neighbors.sv"),
        &["PASS action neighbors"],
    );
}
