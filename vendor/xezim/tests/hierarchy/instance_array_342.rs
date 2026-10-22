//! §23.3.3.5: instance-array connections follow declared dimension order.
//! Expected transcripts were checked with the reference simulator.
use xezim::simulate;

fn check(source: &str, count: usize) {
    let result = simulate(source, 1000).unwrap();
    let lines: Vec<_> = result.output.iter().map(|x| x.message.as_str()).collect();
    assert!(!lines.iter().any(|x| x.starts_with("FAIL")), "{lines:?}");
    assert_eq!(
        lines.iter().filter(|x| x.starts_with("PASS")).count(),
        count,
        "{lines:?}"
    );
}

#[test]
fn issue342_instance_array_connections() {
    check(include_str!("../issue_cases/issue342_instance_array.sv"), 5);
}

#[test]
fn issue343_multidimensional_instance_arrays() {
    let source = include_str!("../issue_cases/issue343_instance_array.sv");
    check(source, 2);
    let result = simulate(source, 1000).unwrap();
    let names: Vec<_> = result
        .output
        .iter()
        .map(|x| x.message.as_str())
        .filter(|x| x.starts_with("instance"))
        .collect();
    assert_eq!(
        names,
        [
            "instance top.um[0][0]",
            "instance top.um[0][1]",
            "instance top.um[1][0]",
            "instance top.um[1][1]"
        ]
    );
}

#[test]
fn instance_array_connection_neighbors() {
    check(
        include_str!("../issue_cases/p2_instance_array_neighbors.sv"),
        8,
    );
}

/// §7.4.2 / §23.3.3.5: parameter names denote sized dimensions, including
/// imported and scoped names; real associative index types remain dynamic.
#[test]
fn next_rest_parameter_sized_instances() {
    check(
        include_str!("../issue_cases/rest_parameter_instances.sv"),
        8,
    );
}
