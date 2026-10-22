//! XEZIM_VALUE_TRACE=<substr>[,...] prints every committed change of any
//! signal whose hierarchical name contains a pattern: time, name, old->new
//! value, dispatch phase, and the writing process's origin (file:line).
//! Built for "where does the data stop flowing down the pipeline" debugging
//! on designs we cannot see. NBA commits run outside the scheduling process,
//! so they are labeled `nba` instead of carrying a stale process origin.

use std::process::Command;

const SRC: &str = r#"module test;
  reg clk = 0;
  reg [7:0] wdata = 0;
  reg [7:0] stage1, stage2;
  reg vld = 0;
  always #5 clk = ~clk;
  always @(posedge clk) if (vld) stage1 <= wdata;
  always @(posedge clk) stage2 <= stage1;
  initial begin
    @(posedge clk); vld = 1; wdata = 8'hA5;
    @(posedge clk); wdata = 8'h3C;
    @(posedge clk); vld = 0;
    #20 $finish;
  end
endmodule
"#;

fn run(trace: &str, src_name: &str, src: &str) -> String {
    let dir = std::env::temp_dir().join(format!("xezim_vt_{}_{}", src_name, std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join(src_name);
    std::fs::write(&path, src).unwrap();
    let out = Command::new(env!("CARGO_BIN_EXE_xezim"))
        .args([
            "--simulate",
            "-s",
            "test",
            path.to_str().unwrap(),
            "--no-cache",
        ])
        .env("XEZIM_VALUE_TRACE", trace)
        .output()
        .expect("run xezim");
    let mut text = String::from_utf8_lossy(&out.stdout).to_string();
    text.push_str(&String::from_utf8_lossy(&out.stderr));
    text
}

#[test]
fn traces_blocking_writes_with_process_origin() {
    let text = run("wdata,vld", "vt_blk.sv", SRC);
    // Pattern resolution is announced so a typo is visible immediately.
    assert!(
        text.contains("pattern 'wdata' matched 1 signal(s)"),
        "{text}"
    );
    // Blocking writes carry old -> new, the active phase, and the writer.
    assert!(
        text.contains("t=5 wdata 00000000 -> 10100101 (active; initial block at"),
        "{text}"
    );
    assert!(text.contains("t=5 vld 0 -> 1 (active;"), "{text}");
}

#[test]
fn traces_nba_commits_without_stale_origin() {
    let text = run("stage", "vt_nba.sv", SRC);
    // stage1 <= wdata commits in the NBA region; the writer's pid is stale
    // there, so the line must say `nba`, not name some unrelated process.
    assert!(
        text.contains("t=15 stage1 xxxxxxxx -> 10100101 (nba; nba commit)"),
        "{text}"
    );
    assert!(
        text.contains("t=25 stage2 xxxxxxxx -> 10100101 (nba; nba commit)"),
        "{text}"
    );
    assert!(
        !text.contains("stage1 xxxxxxxx -> 10100101 (nba; initial"),
        "{text}"
    );
}

#[test]
fn traces_in_place_bit_writes() {
    // Bit/part-select writes mutate the table in place (no pre-write value),
    // so they print the arrowless `name = value` form — but they must print.
    let src = r#"module test;
  reg [7:0] word;
  reg clk = 0;
  always #5 clk = ~clk;
  initial begin
    word = 0;
    @(posedge clk) word[3] = 1'b1;
    @(posedge clk) word[7:4] = 4'hC;
    #10 $finish;
  end
endmodule
"#;
    let text = run("word", "vt_bits.sv", src);
    assert!(text.contains("t=5 word = 00001000 (active;"), "{text}");
    assert!(text.contains("t=15 word = 11001000 (active;"), "{text}");
}

#[test]
fn unmatched_pattern_is_reported() {
    let text = run("no_such_signal", "vt_miss.sv", SRC);
    assert!(
        text.contains("pattern 'no_such_signal' matched no signal"),
        "{text}"
    );
    assert!(!text.contains("[value-trace] t="), "{text}");
}
