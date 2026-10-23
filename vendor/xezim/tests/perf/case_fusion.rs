//! §12.5 case selection: dense tables and recycled comparison temporaries
//! preserve first matches, hold behavior, sizing and X/Z semantics. Expected
//! T| lines were recorded from the reference simulator for this fixture.
use std::process::Command;

const DESIGN: &str = r#"
module tb;
  logic clk = 0;
  always #5 clk = ~clk;
  logic [3:0] selector = 0;
  logic [7:0] seed = 8'h31;
  logic signed [7:0] signed_selector = 0;
  logic [64:0] wide_selector = 0;
  logic [7:0] dense4 = 0, dense7 = 0, duplicate = 0, held = 8'h5a;
  logic [7:0] signed_result = 0, wide_result = 0, unknown_result = 0;
  always @(posedge clk) begin
    case (selector)
      0: dense4 <= seed + 8'd1;
      1: dense4 <= seed + 8'd2;
      2: dense4 <= seed + 8'd3;
      3: dense4 <= seed + 8'd4;
      default: dense4 <= seed ^ 8'ha5;
    endcase
    case (selector)
      4'd0: dense7 <= seed + 8'd1;
      8'd1: dense7 <= seed ^ 8'd2;
      32'd2: dense7 <= seed + 8'd3;
      3'd3: dense7 <= seed ^ 8'd4;
      4: dense7 <= seed + 8'd5;
      5: dense7 <= seed ^ 8'd6;
      6: dense7 <= seed + 8'd7;
      default: dense7 <= seed ^ 8'h5a;
    endcase
    case (selector)
      0: duplicate <= seed + 8'd1;
      0: duplicate <= seed + 8'd99;
      1: duplicate <= seed + 8'd2;
      2: duplicate <= seed + 8'd3;
      3: duplicate <= seed + 8'd4;
      default: duplicate <= seed ^ 8'hff;
    endcase
    case (selector)
      0: held <= seed + 8'd1;
      1: held <= seed + 8'd2;
      2: held <= seed + 8'd3;
      3: held <= seed + 8'd4;
    endcase
    case (signed_selector)
      -1: signed_result <= seed + 8'd1;
      0: signed_result <= seed + 8'd2;
      1: signed_result <= seed + 8'd3;
      2: signed_result <= seed + 8'd4;
      default: signed_result <= seed;
    endcase
    case (wide_selector)
      0: wide_result <= seed + 8'd1;
      1: wide_result <= seed + 8'd2;
      2: wide_result <= seed + 8'd3;
      3: wide_result <= seed + 8'd4;
      default: wide_result <= seed;
    endcase
    case (selector)
      4'bxxxx: unknown_result <= seed + 8'd1;
      4'bzzzz: unknown_result <= seed + 8'd2;
      0: unknown_result <= seed + 8'd3;
      1: unknown_result <= seed + 8'd4;
      default: unknown_result <= seed;
    endcase
  end
  initial begin
    for (int step = 0; step < 18; step++) begin
      @(negedge clk);
      selector = step < 16 ? 4'(step) : step == 16 ? 4'bxxxx : 4'bzzzz;
      seed = 8'(step * 13 + 49);
      signed_selector = 8'(step - 1);
      wide_selector = {1'(step & 1), 60'b0, 4'(step)};
      @(posedge clk);
      #1;
      $display("T|%0d %h %h %h %h %h %h %h", step, dense4, dense7, duplicate, held, signed_result, wide_result, unknown_result);
    end
    $finish;
  end
endmodule
"#;

const EXPECTED: &str = "T|0 32 32 32 32 32 32 34
T|1 40 3c 40 40 40 3e 42
T|2 4e 4e 4e 4e 4e 4e 4b
T|3 5c 5c 5c 5c 5c 58 58
T|4 c0 6a 9a 5c 65 65 65
T|5 d7 74 8d 5c 72 72 72
T|6 da 86 80 5c 7f 7f 7f
T|7 29 d6 73 5c 8c 8c 8c
T|8 3c c3 66 5c 99 99 99
T|9 03 fc 59 5c a6 a6 a6
T|10 16 e9 4c 5c b3 b3 b3
T|11 65 9a 3f 5c c0 c0 c0
T|12 68 97 32 5c cd cd cd
T|13 7f 80 25 5c da da da
T|14 42 bd 18 5c e7 e7 e7
T|15 51 ae 0b 5c f4 f4 f4
T|16 a4 5b fe 5c 01 02 02
T|17 ab 54 f1 5c 0e 0e 10";

fn run(source: &str, expected: &str, jump_tables: bool, merge_mode: &str) -> String {
    static SEQ: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);
    let dir = std::env::temp_dir().join(format!(
        "xezim_case_fusion_{}_{}",
        std::process::id(),
        SEQ.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
    ));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("design.sv");
    std::fs::write(&path, source).unwrap();
    let mut command = Command::new(env!("CARGO_BIN_EXE_xezim"));
    command
        .args([
            "--simulate",
            "--no-cache",
            "-s",
            "tb",
            path.to_str().unwrap(),
        ])
        .env("XEZIM_JIT", "0")
        .env("XEZIM_VERBOSE", "1")
        .env("XEZIM_EDGE_MERGE", merge_mode)
        .env("XEZIM_PROFILE_REPORT", "1")
        .env("XEZIM_EDGE_BLOCK_STATS", "1")
        .env_remove("XEZIM_CASEJUMP_OFF");
    if !jump_tables {
        command.env("XEZIM_CASEJUMP_OFF", "1");
    }
    let output = command.output().expect("run case fixture");
    let text = format!(
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let _ = std::fs::remove_dir_all(dir);
    assert!(output.status.success(), "case fixture failed:\n{text}");
    let actual = text
        .lines()
        .filter(|line| line.starts_with("T|"))
        .collect::<Vec<_>>()
        .join("\n");
    assert_eq!(actual, expected, "jump_tables={jump_tables}");
    text
}

fn vm_instructions(text: &str) -> u64 {
    text.lines()
        .find_map(|line| line.strip_prefix("[PROF] vm_insns_total="))
        .expect("VM instruction counter")
        .parse()
        .unwrap()
}

#[test]
fn dense_cases_preserve_reference_values_and_execute_less_bytecode() {
    let enabled = run(DESIGN, EXPECTED, true, "8");
    let disabled = run(DESIGN, EXPECTED, false, "8");
    assert!(
        enabled
            .lines()
            .any(|line| line.split_whitespace().nth(1) == Some("CaseJump:")),
        "dense cases did not use jump tables:\n{enabled}"
    );
    assert!(
        !disabled
            .lines()
            .any(|line| line.split_whitespace().nth(1) == Some("CaseJump:"))
    );
    assert!(
        vm_instructions(&enabled) < vm_instructions(&disabled),
        "jump tables failed to reduce executed work"
    );
}

const MERGED_DESIGN: &str = r#"
module decoder #(parameter BIAS = 0)(input clk, input [1:0] selector, input [7:0] seed, output logic [7:0] q);
  always @(posedge clk) begin
    case (selector)
      0: q <= seed + BIAS + 1;
      1: q <= seed + BIAS + 2;
      2: q <= seed + BIAS + 3;
      3: q <= seed + BIAS + 4;
      default: q <= seed + BIAS + 5;
    endcase
  end
endmodule
module tb;
  logic clk = 0;
  always #5 clk = ~clk;
  logic [1:0] selector = 0;
  logic [7:0] seed = 8'h20;
  wire [7:0] q [0:9];
  for (genvar index = 0; index < 10; index++) begin : lanes
    decoder #(index) u(clk, selector, seed, q[index]);
  end
  initial begin
    for (int step = 0; step < 6; step++) begin
      @(negedge clk);
      selector = step < 4 ? 2'(step) : step == 4 ? 2'bxx : 2'bzz;
      seed = 8'(step * 17 + 32);
      @(posedge clk);
      #1;
      $display("T|merge %0d %h %h %h", step, q[0], q[5], q[9]);
    end
    $finish;
  end
endmodule
"#;

const MERGED_EXPECTED: &str = "T|merge 0 21 26 2a
T|merge 1 33 38 3c
T|merge 2 45 4a 4e
T|merge 3 57 5c 60
T|merge 4 69 6e 72
T|merge 5 7a 7f 83";

/// §12.5 and §10.4.2: jump targets and registers relocate across merged NBA bodies.
#[test]
fn dense_case_blocks_keep_merging_and_reference_values() {
    let merged = run(MERGED_DESIGN, MERGED_EXPECTED, true, "8");
    let separate = run(MERGED_DESIGN, MERGED_EXPECTED, true, "0");
    assert!(
        merged.contains("[EDGE-MERGE] merged 10 blocks into 2 (cap 8)"),
        "dense case merging was lost:\n{merged}"
    );
    assert!(!separate.contains("[EDGE-MERGE] merged"));
}
