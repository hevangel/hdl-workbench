//! IEEE 1800-2023 §7.2.1, §11.4.1: actual struct copies materialize
//! indexed sources once, preserving scalar and variable-size members.

use xezim::simulate;

fn lines(source: &str) -> Vec<String> {
    simulate(source, 100)
        .expect("simulate failed")
        .output
        .iter()
        .map(|line| line.message.clone())
        .filter(|line| line.starts_with("T|"))
        .collect()
}

#[test]
fn impure_index_copies_distinct_struct_payload_once() {
    let source = r#"module top;
  typedef struct { int tag; int q[$]; } record_t;
  record_t rows[2], dst;
  int calls = 0;
  function automatic int pick(); calls++; return 1; endfunction
  initial begin
    rows[0].tag = 7;
    rows[0].q.push_back(31);
    rows[1].tag = 9;
    rows[1].q.push_back(42);
    rows[1].q.push_back(43);
    dst = rows[0];
    $display("T|control %0d %0d %0d %0d", dst.tag, dst.q.size(), dst.q[0], calls);
    dst = rows[pick()];
    $display("T|selected %0d %0d %0d %0d %0d", dst.tag, dst.q.size(), dst.q[0], dst.q[1], calls);
  end
endmodule
"#;
    assert_eq!(
        lines(source),
        ["T|control 7 1 31 0", "T|selected 9 2 42 43 1"]
    );
}

#[test]
fn indexed_struct_copy_neighbors() {
    // Each complete payload and selector count matches the reference simulator.
    assert_eq!(lines(include_str!("struct_copy_selector.sv")), EXPECTED);
}

const EXPECTED: &[&str] = &[
    "T|fixed 9 2 42 43 1",
    "T|increment 9 2 42 43 2",
    "T|dynamic 9 2 42 43 1",
    "T|queue 9 2 42 43 1",
    "T|associative 9 2 42 43 1",
    "T|both 9 2 42 43 1 1",
    "T|member 9 2 42 43 1",
    "T|pop 9 2 42 43 0",
];
