//! §6.21, §13.3 and §13.4: subroutine statics initialize once before time zero.
//! Aggregates share declaration-owned storage across calls and recursion.
//! Expected transcripts were checked with the reference simulator.
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
fn issue349_static_initializer_timing() {
    check(
        include_str!("../issue_cases/issue349_static_lifetime.sv"),
        7,
    );
}
#[test]
fn issue355_static_aggregate_storage() {
    check(
        include_str!("../issue_cases/issue355_static_lifetime.sv"),
        9,
    );
}
#[test]
fn static_lifetime_neighbors() {
    check(
        include_str!("../issue_cases/p2_static_lifetime_neighbors.sv"),
        13,
    );
}

/// Chosen relaxed policy: an initialized block local without an explicit
/// lifetime initializes on each entry. The reference rejects this form.
#[test]
fn chosen_implicit_block_initializer_policy() {
    let result = simulate(
        r#"module top;
int answer;
initial begin
 for(int i=0;i<2;i++) begin int n=0; n++; answer=n; end
 $display("T|%0d",answer);
end
endmodule"#,
        1000,
    )
    .unwrap();
    let lines: Vec<_> = result.output.iter().map(|x| x.message.as_str()).collect();
    assert_eq!(lines, ["T|1"]);
}
