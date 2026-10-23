//! Followed-by decision timing and vacuity (§16.12.9, §16.14.8).
fn check(source: &str, expected: &[&str]) {
    let result = xezim::simulate(source, 100_000).unwrap();
    let mut lines: Vec<_> = result
        .output
        .iter()
        .map(|entry| entry.message.as_str())
        .collect();
    assert!(
        !lines.iter().any(|line| line.starts_with("FAIL")),
        "{lines:?}"
    );
    lines.retain(|line| *line == "PASS" || line.starts_with("T|"));
    lines.sort_unstable();
    assert_eq!(lines, expected);
}

#[test]
fn issue395_original() {
    check(
        include_str!("../issue_cases/issue395_original.sv"),
        &["PASS"],
    );
}

#[test]
fn followed_by_settles_on_the_first_successful_tick() {
    check(
        include_str!("../issue_cases/issue395_followed_neighbors.sv"),
        &[
            "T|follow|0 0 0|0 0 4",
            "T|follow|0 0 1|1 1 4",
            "T|follow|0 1 0|0 0 4",
            "T|follow|1 0 0|0 0 4",
            "T|follow|1 0 1|1 1 4",
            "T|follow|1 1 0|0 0 4",
            "T|mixed|0|1 1 4",
            "T|mixed|1|1 1 4",
        ],
    );
}
