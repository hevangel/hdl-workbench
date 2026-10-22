//! Lexical virtual-interface bindings survive nested scopes (§6.21, §25.9).
fn check(source: &str, expected: &[&str]) {
    let sim = xezim::simulate(source, 100_000).expect("simulation failed");
    let lines: Vec<_> = sim
        .output
        .iter()
        .map(|o| o.message.as_str())
        .filter(|s| s.starts_with("T|"))
        .collect();
    assert_eq!(lines, expected);
}

#[test]
fn static_vif_audit_local_frames() {
    check(
        include_str!("../issue_cases/static_vif_local_frames_audit.sv"),
        &["T|formal 1 0", "T|nested 47", "T|preserved 17"],
    );
}

#[test]
fn static_vif_audit_nested_scope_variants() {
    check(
        include_str!("../issue_cases/static_vif_scope_variants_audit.sv"),
        &[
            "T|local-null 1",
            "T|local-inner 29",
            "T|local-outer 17",
            "T|formal-null 1",
            "T|formal-inner 29",
            "T|formal-outer 17",
            "T|delay-null 1",
            "T|delay-inner 29",
            "T|delay-outer 17",
            "T|loop 94",
            "T|instance 47",
            "T|return 1 17",
        ],
    );
}
