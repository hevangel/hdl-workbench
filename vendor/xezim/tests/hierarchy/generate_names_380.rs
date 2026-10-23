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

/// IEEE 1800 §27.6: issue #380, original reproducer.
#[test]
fn issue380_original() {
    check(include_str!("../issue_cases/rest_issue380_0.sv"), &["PASS"]);
}

/// IEEE 1800 §27.6: waveform and toggle coverage use the same public scopes.
#[test]
fn unnamed_generate_waveform_and_coverage_names() {
    let dir = std::env::temp_dir().join(format!("xezim_gen_scope_380_{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(
        dir.join("top.sv"),
        include_str!("../issue_cases/rest_issue380_0.sv"),
    )
    .unwrap();
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_xezim"))
        .current_dir(&dir)
        .env("XEZIM_COV_DB", dir.join("cov.json"))
        .args(["--no-cache", "--wave", "--code-coverage=toggle", "top.sv"])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let text = String::from_utf8_lossy(&output.stdout);
    assert!(text.lines().any(|line| line == "PASS"), "{text}");
    let waveform = std::fs::read_to_string(dir.join("unnamed_genblk.vcd")).unwrap();
    let coverage = std::fs::read_to_string(dir.join("cov.json")).unwrap();
    for scope in ["genblk1[0]", "genblk1[1]"] {
        assert!(waveform.contains(scope), "{waveform}");
        assert!(coverage.contains(scope), "{coverage}");
    }
    assert!(!waveform.contains("__gf_"), "{waveform}");
    assert!(!coverage.contains("__gf_"), "{coverage}");
    assert!(waveform.contains("unl"), "{waveform}");
    let _ = std::fs::remove_dir_all(dir);
}

/// IEEE 1800 §27.4 / §27.6: neighboring cases.
#[test]
fn generate_neighbors() {
    check(
        include_str!("../issue_cases/rest_generate_neighbors.sv"),
        &["PASS generate neighbors"],
    );
}
