//! IEEE 1800 §6.21, §12.7.1, §13.3 and §23.9: declarations keep their own
//! storage and declared type across nested scopes and process suspension.
//! Expected results were checked with the reference simulator.
use xezim::simulate;

fn check(src: &str, expected_passes: usize) {
    let sim = simulate(src, 1000).expect("simulate scope regression");
    let lines: Vec<_> = sim.output.iter().map(|o| o.message.as_str()).collect();
    assert!(
        !lines
            .iter()
            .any(|s| s.contains("FAIL") || s.contains("RUNAWAY")),
        "{lines:?}"
    );
    assert_eq!(
        lines.iter().filter(|s| s.starts_with("PASS")).count(),
        expected_passes,
        "{lines:?}"
    );
}

#[test]
fn issue341_declaration_scope() {
    check(include_str!("../issue_cases/issue341_scope.sv"), 1);
}

#[test]
fn issue344_declaration_scope() {
    check(include_str!("../issue_cases/issue344_scope.sv"), 10);
}

#[test]
fn issue345_declaration_scope() {
    check(include_str!("../issue_cases/issue345_scope.sv"), 1);
}

#[test]
fn issue348_declaration_scope() {
    check(include_str!("../issue_cases/issue348_scope.sv"), 5);
}

#[test]
fn issue350_declaration_scope() {
    check(include_str!("../issue_cases/issue350_scope.sv"), 2);
}

#[test]
fn scope_declaration_neighbors() {
    check_lines(
        include_str!("../issue_cases/p1_scope_neighbors.sv"),
        &[
            "T|PASS loop signed 4",
            "T|PASS loop outer 5",
            "T|PASS short outer -2",
            "T|PASS byte outer -8",
            "T|PASS concurrent int 1001",
            "T|PASS concurrent bits 2",
            "T|PASS output formal -1000",
            "T|PASS nested signed -2",
            "T|PASS string collision 79",
            "T|errors=0",
        ],
    );
}

#[test]
fn storage_declaration_neighbors() {
    check_lines(
        include_str!("../issue_cases/p1_storage_neighbors.sv"),
        &[
            "T|PASS always nested collision 2",
            "T|PASS static nested collision 22",
            "T|PASS named nesting 50",
            "T|PASS named static 101",
            "T|PASS instance nesting 10",
            "T|errors=0",
        ],
    );
}

#[test]
fn context_declaration_neighbors() {
    check_lines(
        include_str!("../issue_cases/p1_context_neighbors.sv"),
        &["T|counts=3 3 13 1 3 errors=0"],
    );
}

#[test]
fn select_declaration_neighbors() {
    check_lines(
        include_str!("../issue_cases/p1_select_neighbors.sv"),
        &[
            "T|PASS input select 3e",
            "T|PASS local selects 85",
            "T|PASS property selects 91",
            "T|PASS indexed up d0",
            "T|PASS indexed down 30",
            "T|PASS module unchanged a5",
            "T|errors=0",
        ],
    );
}

fn check_lines(src: &str, expected: &[&str]) {
    let sim = simulate(src, 1000).expect("simulate declaration neighbors");
    let lines: Vec<_> = sim.output.iter().map(|o| o.message.as_str()).collect();
    assert!(!lines.iter().any(|s| s.contains("FAIL")), "{lines:?}");
    let trace: Vec<_> = lines.into_iter().filter(|s| s.starts_with("T|")).collect();
    assert_eq!(trace, expected);
}

#[test]
fn delayed_loop_declaration_neighbors() {
    check_lines(
        include_str!("../issue_cases/p1_loop_neighbors.sv"),
        &[
            "T|PASS byte 4",
            "T|PASS outer byte 5",
            "T|PASS shortint 4",
            "T|PASS outer shortint 5",
            "T|PASS integer 4",
            "T|PASS outer integer 5",
            "T|PASS packed signed 4",
            "T|PASS outer packed 5",
            "T|PASS typedef 4",
            "T|PASS outer typedef 5",
            "T|errors=0",
        ],
    );
}

#[test]
fn surviving_capture_and_nested_string_neighbors() {
    check_lines(
        include_str!("../issue_cases/p1_capture_neighbors.sv"),
        &["T|capture 2 1", "T|string Zbcdefgh", "T|errors=0"],
    );
}

#[test]
fn string_formal_aliases_and_parameters_keep_frame_kind() {
    check_lines(
        include_str!("../issue_cases/p1_string_formal_neighbors.sv"),
        &[
            "T|direct Abc!",
            "T|alias Bbc!",
            "T|chain Cbc!",
            "T|scoped Dbc!",
            "T|parameter Ebc!",
            "T|packed 12345679",
        ],
    );
}

#[test]
fn scoped_string_formal_chains_resist_module_alias_collisions() {
    check_lines(
        include_str!("../issue_cases/p1_scoped_string_formal_neighbors.sv"),
        &["T|scoped-chain Sbc!", "T|scoped-packed 12345679"],
    );
}

#[test]
fn blocking_string_formals_follow_receiver_specialization() {
    check_lines(
        include_str!("../issue_cases/p1_blocking_string_formal_neighbors.sv"),
        &[
            "T|direct Abc!",
            "T|alias Bbc!",
            "T|parameter-string Ebc!",
            "T|parameter-int 12345679",
            "T|static-string Sbc!",
            "T|static-int 87654320",
            "T|parameter-string-again Eyz!",
        ],
    );
}
