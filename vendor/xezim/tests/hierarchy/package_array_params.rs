//! Unpacked parameter shapes and scoped constant references (§6.20.2 / §26.3).
use std::process::Command;

fn check(name: &str, source: &str, top: &str, expected: &[&str]) {
    let dir = std::path::Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("package_array_params")
        .join(name);
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("top.sv");
    std::fs::write(&path, source).unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_xezim"))
        .args(["--no-cache", "--error-exit", "-s", top])
        .arg(path)
        .output()
        .unwrap();
    let text = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(output.status.success(), "{name}: {text}");
    let actual: Vec<_> = text.lines().filter(|s| s.starts_with("T|")).collect();
    assert_eq!(actual, expected, "{name}: {text}");
}

#[test]
fn submitted() {
    check(
        "submitted",
        include_str!("../issue_cases/package_array_params/submitted.sv"),
        "c26pad",
        &[
            "T|26.3g|size(ctl)=8",
            "T|26.3g|size(mem)=8",
            "T|26.3g|mem[7]=5 elem=8",
        ],
    );
}

#[test]
fn package_ascending() {
    check(
        "package_ascending",
        include_str!("../issue_cases/package_array_params/package_ascending.sv"),
        "top",
        &["T|bounds 4 5 2", "T|values 17 29"],
    );
}

#[test]
fn package_descending() {
    check(
        "package_descending",
        include_str!("../issue_cases/package_array_params/package_descending.sv"),
        "top",
        &["T|bounds 5 4 2", "T|values 17 29"],
    );
}

#[test]
fn package_negative() {
    check(
        "package_negative",
        include_str!("../issue_cases/package_array_params/package_negative.sv"),
        "top",
        &["T|negative -2 -1 29 29 29"],
    );
}

#[test]
fn package_dependencies() {
    check(
        "package_dependencies",
        include_str!("../issue_cases/package_array_params/package_dependencies.sv"),
        "top",
        &["T|dependencies 5 7 7"],
    );
}

#[test]
fn package_imports() {
    check(
        "package_imports",
        include_str!("../issue_cases/package_array_params/package_imports.sv"),
        "top",
        &["T|imports 5 5 5"],
    );
}

#[test]
fn package_oob_shadow() {
    check(
        "package_oob_shadow",
        include_str!("../issue_cases/package_array_params/package_oob_shadow.sv"),
        "top",
        &["T|oob 0 0 17"],
    );
}

#[test]
fn body_override() {
    check(
        "body_override",
        include_str!("../issue_cases/package_array_params/body_override.sv"),
        "top",
        &["T|override 2 2 2 9"],
    );
}

#[test]
fn body_bounds() {
    check(
        "body_bounds",
        include_str!("../issue_cases/package_array_params/body_bounds.sv"),
        "top",
        &["T|bodybounds 4 5 17 29"],
    );
}

#[test]
fn body_generate() {
    check(
        "body_generate",
        include_str!("../issue_cases/package_array_params/body_generate.sv"),
        "top",
        &["T|generate 9 10"],
    );
}

#[test]
fn body_dependencies() {
    check(
        "body_dependencies",
        include_str!("../issue_cases/package_array_params/body_dependencies.sv"),
        "top",
        &["T|bodydep 5 7 7"],
    );
}

#[test]
fn package_multidimensional() {
    check(
        "package_multidimensional",
        include_str!("../issue_cases/package_array_params/package_multidimensional.sv"),
        "top",
        &["T|multi 5 5 5"],
    );
}

#[test]
fn interface_body() {
    check(
        "interface_body",
        include_str!("../issue_cases/package_array_params/interface_body.sv"),
        "top",
        &["T|interface 3 8"],
    );
}

#[test]
fn package_oob_four_state() {
    check(
        "package_oob_four_state",
        include_str!("../issue_cases/package_array_params/package_oob_four_state.sv"),
        "top",
        &["T|oob 1 1 17"],
    );
}

#[test]
fn package_import_dependencies() {
    check(
        "package_import_dependencies",
        include_str!("../issue_cases/package_array_params/package_import_dependencies.sv"),
        "top",
        &["T|importenv 4 5 17 29 29"],
    );
}

#[test]
fn aggregate_elements_preserve_member_reads() {
    check(
        "aggregate_elements",
        include_str!("../issue_cases/package_array_params/aggregate_elements.sv"),
        "top",
        &["T|aggregate 9 10 7 8 13 4 5 101 011 0 1"],
    );
}
