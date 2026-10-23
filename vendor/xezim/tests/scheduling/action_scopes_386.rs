//! IEEE 1800-2017 §16.14.1, §23.9 and §27.4: assertion actions bind
//! to their declaring instance/generate scope; action locals shadow module variables.
//! Issue #386; expected results checked with the reference simulator.

use xezim::simulate;

fn output(src: &str) -> Vec<String> {
    simulate(src, 100)
        .expect("simulation failed")
        .output
        .iter()
        .map(|o| o.message.trim().to_string())
        .collect()
}

#[test]
fn original_386() {
    let out = output(include_str!("../issue_cases/original_386.sv"));
    assert!(out.iter().any(|s| s == "PASS"), "{out:?}");
}

#[test]
fn neighbours_386() {
    let out = output(include_str!("../issue_cases/action_scopes_386.sv"));
    let actual: Vec<_> = out.iter().filter(|s| s.starts_with("T|")).collect();
    let expected = [
        "T|generate|3 3 6",
        "T|siblings|3 3 3",
        "T|nested|3 3 3",
        "T|block_shadow|0 0",
        "T|child_idle|3 0",
        "T|fail_inc|3 3",
        "T|cover_inc|3 3",
        "T|assert_compound|3 3",
        "T|assert_assign|3 3",
        "T|assert_inc|3 3",
    ];
    assert_eq!(actual.len(), expected.len(), "{out:?}");
    for want in expected {
        assert!(
            actual.iter().any(|s| s.as_str() == want),
            "missing {want:?}: {out:?}"
        );
    }
}
