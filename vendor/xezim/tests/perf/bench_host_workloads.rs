//! CI smoke for the host-CPU benchmark workloads (`src/benchw.rs`,
//! `xezim-bench` binary): each workload runs a SMALL cycle count and its
//! self-check (the Rust mirror of the design arithmetic) must pass. No
//! wall-clock assertions — timing lives in the bench binary, correctness
//! lives here, so a semantics regression can't hide behind a fast number.

#[test]
fn bench_workloads_self_check() {
    for w in xezim::benchw::workloads() {
        let src = (w.source)(500);
        let sim = xezim::simulate(&src, (w.sim_time)(500))
            .unwrap_or_else(|e| panic!("{}: compile/run failed: {}", w.name, e));
        (w.check)(&sim);
    }
}

#[test]
fn wide_four_state_word_operations() {
    let src = r#"
module tb;
  reg [291:0] source_bits, peer_bits, actual_bits, expected_bits;
  reg unknown_select;
  integer failures = 0;
  integer completed = 0;
  initial begin
    source_bits = {73{4'bzx10}};
    peer_bits = {73{4'b0010}};
    unknown_select = 1'bx;
    for (int shift = 0; shift <= 310; shift += 31) begin
      actual_bits = source_bits << shift;
      for (int bit_index = 0; bit_index < 292; bit_index++) begin
        if (bit_index >= shift) expected_bits[bit_index] = source_bits[bit_index - shift];
        else expected_bits[bit_index] = 0;
      end
      if (actual_bits !== expected_bits) failures++;
      actual_bits = source_bits >> shift;
      for (int bit_index = 0; bit_index < 292; bit_index++) begin
        if (bit_index + shift < 292) expected_bits[bit_index] = source_bits[bit_index + shift];
        else expected_bits[bit_index] = 0;
      end
      if (actual_bits !== expected_bits) failures++;
    end
    actual_bits = unknown_select ? source_bits : peer_bits;
    expected_bits = {73{4'bxx10}};
    if (actual_bits !== expected_bits) failures++;
    completed = 1;
    $finish;
  end
endmodule
"#;
    let sim = xezim::simulate(src, 10).expect("wide word operation simulation");
    let read = |name: &str| {
        sim.get_signal(name)
            .or_else(|| sim.get_signal(&format!("tb.{name}")))
            .unwrap()
            .to_u64()
    };
    assert_eq!(read("completed"), Some(1));
    assert_eq!(read("failures"), Some(0));
}
