//! §27.4: generated instances see signed genvars in both directions.
//! Expected scope names were checked with the reference simulator.
use xezim::simulate;

fn check(source: &str, expected: &[&str]) {
    let sim = simulate(source, 5).expect("simulate signed generate instances");
    let mut lines: Vec<_> = sim
        .output
        .iter()
        .map(|o| o.message.trim_end())
        .filter(|line| line.starts_with("T|"))
        .collect();
    lines.sort_unstable();
    let mut expected = expected.to_vec();
    expected.sort_unstable();
    assert_eq!(lines, expected);
}

#[test]
fn signed_up() {
    check(
        r####"module marker; initial $display("T|%m"); endmodule module top; for(genvar g=-2;g<1;g++) begin: bank marker m(); end endmodule
"####,
        &["T|top.bank[-2].m", "T|top.bank[-1].m", "T|top.bank[0].m"],
    );
}

#[test]
fn signed_down() {
    check(
        r####"module marker; initial $display("T|%m"); endmodule module top; for(genvar g=1;g>=-1;g--) begin: bank marker m(); end endmodule
"####,
        &["T|top.bank[1].m", "T|top.bank[0].m", "T|top.bank[-1].m"],
    );
}
