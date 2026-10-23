//! §8.9 and §11.5.1: selected static property writes reach the shared cell.
use xezim::simulate;
#[test]
fn issue346_static_property_selected_writes() {
    let result = simulate(
        include_str!("../issue_cases/issue346_static_select.sv"),
        1000,
    )
    .unwrap();
    let lines: Vec<_> = result.output.iter().map(|x| x.message.as_str()).collect();
    assert_eq!(
        lines,
        [
            "PASS method bit-select",
            "PASS static method part-select",
            "PASS scoped bit-select"
        ]
    );
}

#[test]
fn static_property_select_neighbors() {
    let result = simulate(
        include_str!("../issue_cases/p2_static_select_neighbors.sv"),
        1000,
    )
    .unwrap();
    let lines: Vec<_> = result.output.iter().map(|x| x.message.as_str()).collect();
    assert!(!lines.iter().any(|x| x.starts_with("FAIL")), "{lines:?}");
    assert!(lines.contains(&"ALL PASS"), "{lines:?}");
    assert_eq!(
        lines.iter().filter(|x| x.starts_with("PASS")).count(),
        14,
        "{lines:?}"
    );
}
