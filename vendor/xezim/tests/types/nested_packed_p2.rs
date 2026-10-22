//! §7.4.1, §7.2.1 and §11.5.1: selected reads and writes follow declared geometry.
//! Expected values were checked with the reference simulator.
use xezim::simulate;

fn check(source: &str, passes: usize) {
    let result = simulate(source, 1000).unwrap();
    let lines: Vec<_> = result.output.iter().map(|x| x.message.as_str()).collect();
    assert!(!lines.iter().any(|x| x.starts_with("FAIL")), "{lines:?}");
    assert!(lines.contains(&"ALL PASS"), "{lines:?}");
    assert_eq!(
        lines.iter().filter(|x| x.starts_with("PASS")).count(),
        passes,
        "{lines:?}"
    );
}
#[test]
fn issue352_nested_packed_locals() {
    check(include_str!("../issue_cases/issue352_packed.sv"), 10);
}
#[test]
fn issue353_nested_packed_signals() {
    check(include_str!("../issue_cases/issue353_packed.sv"), 8);
}
#[test]
fn issue354_unpacked_packed_boundary() {
    check(include_str!("../issue_cases/issue354_packed.sv"), 7);
}
#[test]
fn nested_packed_geometry_neighbors() {
    check(
        include_str!("../issue_cases/p2_nested_packed_neighbors.sv"),
        18,
    );
}
