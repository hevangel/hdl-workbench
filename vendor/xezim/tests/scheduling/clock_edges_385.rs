//! IEEE 1800-2017 §9.4.2 (Table 9-2), §16.5 and §16.14: declarative assertions
//! observe the first Active clock edge and sample the preceding Preponed values.
//! Issue #385; expected results checked with the reference simulator.

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
fn original_385() {
    let out = output(include_str!("../issue_cases/original_385.sv"));
    assert!(out.iter().any(|s| s == "PASS"), "{out:?}");
}

#[test]
fn neighbours_385() {
    let out = output(include_str!("../issue_cases/clock_edges_385.sv"));
    let actual: Vec<_> = out.iter().filter(|s| s.starts_with("T|")).collect();
    let expected = [
        "T|x_rise|4 4 4",
        "T|zero_rise|4 4 4",
        "T|x_fall|4 4 4",
        "T|one_fall|4 4 4",
        "T|late_rise|3 3 3",
        "T|declared_one|3 3 3",
        "T|declared_zero|3 3 3",
        "T|zero_to_x|4 4 4",
        "T|one_to_z|4 4 4",
        "T|x_to_z|3 3 3",
        "T|vector_lsb|2 2 2",
        "T|vector_x_msb|4 4 4",
        "T|iff_initial|3 3 3",
        "T|preponed|1 3",
    ];
    assert_eq!(actual.len(), expected.len(), "{out:?}");
    for want in expected {
        assert!(
            actual.iter().any(|s| s.as_str() == want),
            "missing {want:?}: {out:?}"
        );
    }
}
