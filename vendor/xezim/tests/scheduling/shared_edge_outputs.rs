//! §9.2: an input-stable process still writes after another driver changes Q.

use std::process::Command;

#[test]
fn opposite_edge_writers_are_not_suppressed() {
    let dir = std::env::temp_dir().join(format!("shared_edge_outputs_{}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("temporary directory");
    let source = dir.join("drivers.sv");
    std::fs::write(
        &source,
        r#"
module top;
  reg clock=0;
  reg whole=0, immediate=0;
  reg [3:0] partial=0;
  int errors=0;
  always #2 clock=~clock;
  always @(posedge clock) whole<=1;
  always @(negedge clock) whole<=0;
  always @(posedge clock) immediate=1;
  always @(negedge clock) immediate=0;
  always @(posedge clock) partial[2:1]<=2'b11;
  always @(negedge clock) partial[1:0]<=2'b00;
  initial begin
    repeat(40) begin
      @(posedge clock); #1;
      if(whole!==1 || immediate!==1 || partial[2:1]!==2'b11) errors++;
      @(negedge clock); #1;
      if(whole!==0 || immediate!==0 || partial[1:0]!==2'b00) errors++;
    end
    $display("SHARED_ERRORS %0d",errors);
    $finish;
  end
endmodule
"#,
    )
    .expect("write source");
    for armed in ["0", "1"] {
        for merge in ["0", "1"] {
            let result = Command::new(env!("CARGO_BIN_EXE_xezim"))
                .env("XEZIM_EVENT_EDGE", "1")
                .env("XEZIM_ARMED_EDGE", armed)
                .env("XEZIM_EDGE_MERGE", merge)
                .env("XEZIM_EVENT_EDGE_HEAL", "0")
                .args(["-s", "top"])
                .arg(&source)
                .output()
                .expect("run simulator");
            let text = format!(
                "{}{}",
                String::from_utf8_lossy(&result.stdout),
                String::from_utf8_lossy(&result.stderr)
            );
            assert!(result.status.success(), "{text}");
            assert!(
                text.contains("SHARED_ERRORS 0"),
                "armed={armed} merge={merge}: {text}"
            );
        }
    }
}
