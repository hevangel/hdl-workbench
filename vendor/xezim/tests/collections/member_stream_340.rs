//! §11.4.14: class collection streams contain their elements in stream order.
//! Expected values were checked with the reference simulator.
use xezim::simulate;

#[test]
fn issue340_class_queue_stream() {
    let result = simulate(include_str!("../issue_cases/issue340_stream.sv"), 1000).unwrap();
    let lines: Vec<_> = result.output.iter().map(|x| x.message.as_str()).collect();
    assert!(lines.contains(&"PASS"), "{lines:?}");
    assert!(!lines.iter().any(|x| x.starts_with("FAIL")), "{lines:?}");
}

#[test]
fn member_stream_neighbors() {
    let result = simulate(
        include_str!("../issue_cases/p2_member_stream_neighbors.sv"),
        1000,
    )
    .unwrap();
    let lines: Vec<_> = result
        .output
        .iter()
        .map(|x| x.message.as_str())
        .filter(|x| x.starts_with("T|"))
        .collect();
    assert_eq!(
        lines,
        [
            "T|queue 123456",
            "T|reverse 563412",
            "T|slice 3456",
            "T|dynamic 789a",
            "T|fixed 123456",
            "T|joined aa123456bb",
            "T|empty 0",
            "T|method 123456"
        ]
    );
}
