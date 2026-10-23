//! IEEE 1800-2017 §13.4.1 and §7.6: a function-name return is a read
//! of its return variable. Unpacked results survive frame teardown in declaration order.
//! Issue #387; expected results checked with the reference simulator.

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
fn original_387() {
    let out = output(include_str!("../issue_cases/original_387.sv"));
    assert!(out.iter().any(|s| s == "got '{}"), "{out:?}");
}

#[test]
fn neighbours_387() {
    let out = output(include_str!("../issue_cases/collection_returns_387.sv"));
    let actual: Vec<_> = out.iter().filter(|s| s.starts_with("T|")).collect();
    let expected = [
        "T|empty|0",
        "T|filled|2 7 0",
        "T|queue|2 7 2",
        "T|local_dyn|2 7 0",
        "T|fixed_local|7 0",
        "T|fixed_self|7 0",
        "T|fixed_fall|8 0",
        "T|ranged_self|7 8",
        "T|controls|13 ok 9",
        "T|calls|11",
    ];
    assert_eq!(actual.len(), expected.len(), "{out:?}");
    for want in expected {
        assert!(
            actual.iter().any(|s| s.as_str() == want),
            "missing {want:?}: {out:?}"
        );
    }
}

#[test]
fn scalar_locals_shadow_module_collections_387() {
    let out = output(include_str!("../issue_cases/return_shadow_387.sv"));
    assert!(out.iter().any(|s| s == "T|shadow|17 2 3"), "{out:?}");
    assert!(
        out.iter().any(|s| s == "T|dynamic_shadow|18 19 4 5 6 7"),
        "{out:?}"
    );
}
