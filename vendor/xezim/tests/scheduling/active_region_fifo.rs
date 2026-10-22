#[test]
fn many_same_time_delay_resumptions_complete() {
    let src = r#"
`timescale 1ns/1ns
module tb;
  localparam int PROCESSES = 256;
  localparam int WAKES = 8;
  int done = 0;

  genvar i;
  generate
    for (i = 0; i < PROCESSES; i++) begin : workers
      initial begin
        repeat (WAKES) #1;
        done++;
      end
    end
  endgenerate

  initial begin
    #(WAKES + 1);
    $display("ACTIVE_FIFO done=%0d", done);
    $finish;
  end
endmodule
"#;

    let sim = xezim::simulate(src, 20).expect("simulate");
    assert!(
        sim.output
            .iter()
            .any(|line| line.message == "ACTIVE_FIFO done=256")
    );
}
