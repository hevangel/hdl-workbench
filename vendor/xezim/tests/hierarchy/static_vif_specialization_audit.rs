//! Static virtual-interface storage and lexical bindings (§8.26, §25.9).
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
fn static_vif_audit_nearest_declaration() {
    check(
        include_str!("../issue_cases/static_vif_declaration_shadow_audit.sv"),
        &[
            "T|parent-after-child 17",
            "T|copy 29",
            "T|child 29 1 0",
            "T|scalars 9 1 23 1",
            "T|methods 1 1 1",
            "T|setter 17 29",
            "T|child-null 1 17",
            "T|static-child 17 29 1",
        ],
    );
}

#[test]
fn static_vif_audit_alias_specializations() {
    check(
        include_str!("../issue_cases/static_vif_alias_specializations_audit.sv"),
        &["T|aliases 17 29", "T|copy 17", "T|clear 1 29"],
    );
}

#[test]
fn static_vif_audit_package_owners() {
    check(
        include_str!("../issue_cases/static_vif_package_owners_audit.sv"),
        &[
            "T|owners 17 29",
            "T|identities 0 1",
            "T|copy 17",
            "T|clear 1 29",
        ],
    );
}

#[test]
fn static_vif_audit_instance_inherited() {
    check(
        include_str!("../issue_cases/static_vif_instance_inherited_audit.sv"),
        &[
            "T|methods 17 17 1",
            "T|members 17 17 1",
            "T|copy 17",
            "T|clear 1 1",
        ],
    );
}

#[test]
fn static_vif_audit_null_specializations() {
    check(
        include_str!("../issue_cases/static_vif_null_specializations_audit.sv"),
        &[
            "T|initial 1 1",
            "T|bound 17 29 0 0",
            "T|copy 29",
            "T|clear 1 1 29 0",
            "T|clearcopy 1",
        ],
    );
}

#[test]
fn static_vif_audit_static_writeback() {
    check(
        include_str!("../issue_cases/static_vif_static_writeback_audit.sv"),
        &[
            "T|object-isolation 17 1",
            "T|objects 17 29",
            "T|copyclear 1 29",
            "T|output 17 29",
            "T|outputclear 1 29",
            "T|specialized-clear 1 29",
            "T|method-output 17 29",
            "T|method-clear 1 29",
            "T|task-output 17 29",
            "T|task-clear 1 29",
            "T|formal-store 17 29",
            "T|formal-clear 1 29",
        ],
    );
}

#[test]
fn static_vif_audit_call_values_and_output_defaults() {
    check(
        include_str!("../issue_cases/static_vif_call_arguments_audit.sv"),
        &[
            "T|call-null 0 1",
            "T|method-call 17",
            "T|free-call 17 1",
            "T|conditional 17 29",
            "T|calls 2",
            "T|null-call 1",
            "T|calls-after-null 3",
            "T|free-output-null 1",
            "T|method-output-null 1",
            "T|task-output-null 1",
        ],
    );
}
