// Self-test: simulator output emitted via `$display` right up to the end of
// the simulation loop must be flushed before end-of-run profiling (`[PROF]`)
// is printed to stderr.
//
// When stdout and stderr are combined into one stream or file (e.g. `2>&1`
// in automated regression test harnesses), an unflushed stdout buffer on the
// writer thread could allow `eprintln!("[PROF]...")` (emitted synchronously
// on the main thread) to overtake the final `$display` output, corrupting
// the tail of the log before gold-file comparison markers.

use xezim::simulate;

const SRC: &str = r#"
module top;
  initial begin
    #10;
    $display("FINAL_SIM_OUTPUT_LINE");
  end
endmodule
"#;

#[test]
fn display_emitted_before_simulation_finish() {
    let sim = simulate(SRC, 1000).expect("simulate failed");
    let msgs: Vec<String> = sim.output.iter().map(|o| o.message.clone()).collect();
    assert!(
        msgs.iter().any(|m| m.contains("FINAL_SIM_OUTPUT_LINE")),
        "expected final simulation output to be captured, got: {:?}",
        msgs
    );
}
