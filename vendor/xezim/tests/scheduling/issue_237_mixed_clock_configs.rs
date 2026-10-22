//! Issue #237 discloses a configuration-dependent stall in a private design.
//! This bounded surrogate covers the reported public shape: unrelated clock
//! periods, state updated on both edges of the slower clock, and two parameter
//! configurations. It is coverage, not a claim that the private failure was
//! reproduced.

use std::io::Read;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

const SOURCE: &str = r#"`timescale 1ps/1ps
module mixed_clock_case #(parameter bit TRACK_FALLING = 0) (output logic done);
  logic quick_clk = 0;
  logic slow_clk = 0;
  int quick_edges = 0;
  int slow_rises = 0;
  int slow_falls = 0;
  always #5000 quick_clk = ~quick_clk;
  always #40690 slow_clk = ~slow_clk;
  always @(posedge quick_clk) quick_edges++;
  always @(posedge slow_clk) slow_rises++;
  generate if (TRACK_FALLING) begin : falling_path
    always @(negedge slow_clk) slow_falls++;
  end endgenerate
  initial begin
    #1000000;
    if (quick_edges != 100 || slow_rises != 12) begin
      $display("TEST_FAIL cfg=%0d quick=%0d rise=%0d fall=%0d",
               TRACK_FALLING, quick_edges, slow_rises, slow_falls);
    end else if (TRACK_FALLING && slow_falls != 12) begin
      $display("TEST_FAIL cfg=%0d quick=%0d rise=%0d fall=%0d",
               TRACK_FALLING, quick_edges, slow_rises, slow_falls);
    end else begin
      $display("TEST_PASS cfg=%0d quick=%0d rise=%0d fall=%0d",
               TRACK_FALLING, quick_edges, slow_rises, slow_falls);
    end
    done = 1;
  end
endmodule
module top;
  logic done_a, done_b;
  mixed_clock_case #(.TRACK_FALLING(0)) a(.done(done_a));
  mixed_clock_case #(.TRACK_FALLING(1)) b(.done(done_b));
  initial begin wait(done_a && done_b); #1; $finish; end
endmodule
"#;

#[test]
fn both_mixed_clock_configurations_complete() {
    let dir = std::env::temp_dir().join(format!("xezim_mixed_clock_{}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("create temporary directory");
    let path = dir.join("mixed_clock.sv");
    std::fs::write(&path, SOURCE).expect("write mixed-clock source");
    let mut child = Command::new(env!("CARGO_BIN_EXE_xezim"))
        .args(["--simulate", "--no-cache", path.to_str().unwrap()])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("run mixed-clock source");
    let start = Instant::now();
    let status = loop {
        if let Some(status) = child.try_wait().expect("wait") {
            break status;
        }
        if start.elapsed() > Duration::from_secs(20) {
            let _ = child.kill();
            panic!("mixed-clock configurations did not complete");
        }
        std::thread::sleep(Duration::from_millis(10));
    };
    let mut text = String::new();
    child.stdout.take().unwrap().read_to_string(&mut text).ok();
    child.stderr.take().unwrap().read_to_string(&mut text).ok();
    let _ = std::fs::remove_dir_all(&dir);
    assert!(status.success(), "run failed:\n{text}");
    assert!(!text.contains("TEST_FAIL"), "{text}");
    assert!(
        text.contains("TEST_PASS cfg=0 quick=100 rise=12 fall=0"),
        "{text}"
    );
    assert!(
        text.contains("TEST_PASS cfg=1 quick=100 rise=12 fall=12"),
        "{text}"
    );
}
