//! §11.4.5, §12.4 and §12.5: two-state constant comparison branches
//! preserve equality, fallback, live results and relocated control flow.
//! Expected lines were recorded from the reference simulator.
use std::process::Command;

const DESIGN: &str = r#"
module tb;
  logic clk = 0;
  always #5 clk = ~clk;
  logic [7:0] value = 0;
  logic signed [7:0] signed_value = 0;
  logic [63:0] word_value = 0;
  logic [64:0] wide_value = 0;
  logic [2:0] selector = 0;
  logic [8:0] flags = 0;
  logic [5:0] signed_flags = 0;
  logic [1:0] width_flags = 0, live_flags = 0;
  logic live_compare = 0;
  logic [7:0] loop_result = 0, table_result = 0;
  always @(posedge clk) begin
    if (value == 8'h2a) flags[0] <= 1; else flags[0] <= 0;
    if (value === 8'h2a) flags[1] <= 1; else flags[1] <= 0;
    if (value == 8'h00) flags[2] <= 1; else flags[2] <= 0;
    if (value === 8'hff) flags[3] <= 1; else flags[3] <= 0;
    if (value == '1) flags[4] <= 1; else flags[4] <= 0;
    if (value == '0) flags[5] <= 1; else flags[5] <= 0;
    if (value == 16'h002a) flags[6] <= 1; else flags[6] <= 0;
    if (value === 8'hxx) flags[7] <= 1; else flags[7] <= 0;
    if (value === 8'hzz) flags[8] <= 1; else flags[8] <= 0;
  end
  always @(posedge clk) begin
    if (signed_value == 16'shfffe) signed_flags[0] <= 1; else signed_flags[0] <= 0;
    if (signed_value === 16'shfffe) signed_flags[1] <= 1; else signed_flags[1] <= 0;
    if (signed_value == 16'hfffe) signed_flags[2] <= 1; else signed_flags[2] <= 0;
    if (signed_value === 8'shfe) signed_flags[3] <= 1; else signed_flags[3] <= 0;
    if (signed_value == '1) signed_flags[4] <= 1; else signed_flags[4] <= 0;
    if (signed_value === 16'shxxxx) signed_flags[5] <= 1; else signed_flags[5] <= 0;
  end
  always @(posedge clk) begin
    if (word_value == 64'h8000000000000000) width_flags[0] <= 1; else width_flags[0] <= 0;
    if (wide_value === 65'h10000000000000000) width_flags[1] <= 1; else width_flags[1] <= 0;
  end
  always @(posedge clk) begin : repeated_branches
    int accum;
    accum = 0;
    for (int index = 0; index < 6; index++) begin
      if (index == 3) accum = accum + 17; else accum = accum + index;
      if (value == 8'h2a) accum = accum + 3;
    end
    loop_result <= accum;
  end
  always @(posedge clk) begin : relocated_table
    case (selector)
      0: begin
        if (value == 8'h2a) table_result <= 8'h91; else table_result <= 8'h11;
      end
      1: table_result <= 8'h22;
      2: table_result <= 8'h33;
      3: table_result <= 8'h44;
      default: table_result <= 8'h55;
    endcase
    if (value === 8'hff) table_result <= 8'hee;
  end
  always @(posedge clk) begin
    live_compare = value == 8'h2a;
    if (live_compare) live_flags[0] <= 1; else live_flags[0] <= 0;
    live_flags[1] <= live_compare;
  end
  initial begin
    for (int step = 0; step < 8; step++) begin
      @(negedge clk);
      case (step)
        0: value = 0;
        1: value = 8'h2a;
        2: value = 8'hff;
        3: value = 8'hfe;
        4: value = 'x;
        5: value = 'z;
        6: value = 8'b0010x010;
        7: value = 8'b0010z010;
      endcase
      signed_value = value;
      selector = step;
      word_value = step == 4 ? 'x : step == 5 ? 'z : step[0] ? 64'h8000000000000000 : 0;
      wide_value = step == 4 ? 'x : step == 5 ? 'z : step[0] ? 65'h10000000000000000 : 0;
      @(posedge clk);
      #1;
      $display("T|%0d %h %h %b %h %h %b", step, flags, signed_flags,
        width_flags, loop_result, table_result, live_flags);
    end
    $finish;
  end
endmodule
"#;

const EXPECTED: &str = "T|0 024 00 00 1d 11 00
T|1 043 00 11 2f 22 11
T|2 018 10 00 1d ee 00
T|3 000 0b 11 1d 44 00
T|4 080 20 00 1d 55 x0
T|5 100 00 00 1d 55 x0
T|6 000 00 00 1d 55 x0
T|7 000 00 11 1d 55 x0";

fn run(four_state: bool, x_plane: bool) -> String {
    static SEQ: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);
    let directory = std::env::temp_dir().join(format!(
        "xezim_eqc_branch_{}_{}",
        std::process::id(),
        SEQ.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
    ));
    std::fs::create_dir_all(&directory).unwrap();
    let path = directory.join("design.sv");
    std::fs::write(&path, DESIGN).unwrap();
    let mut command = Command::new(env!("CARGO_BIN_EXE_xezim"));
    for (key, _) in std::env::vars() {
        if key.starts_with("XEZIM_") {
            command.env_remove(key);
        }
    }
    command
        .args([
            "--simulate",
            "--no-cache",
            "-s",
            "tb",
            path.to_str().unwrap(),
        ])
        .env("XEZIM_JIT", "0")
        .env("XEZIM_TS_JIT", "0")
        .env("XEZIM_AOT", "0")
        .env("XEZIM_NO_PARALLEL", "1")
        .env("XEZIM_VERBOSE", "1")
        .env("XEZIM_PROFILE_REPORT", "1")
        .env("XEZIM_EDGE_BLOCK_STATS", "1")
        .env("XEZIM_EDGE_MERGE", "0")
        .env("XEZIM_TS_X", if x_plane { "1" } else { "0" });
    if four_state {
        command.env("XEZIM_TS_DENY", "Load,LoadS,LoadRng,LoadBit,LoadRngDyn");
    }
    let output = command.output().expect("run constant comparison branches");
    let text = format!(
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let _ = std::fs::remove_dir_all(directory);
    assert!(output.status.success(), "branch fixture failed:\n{text}");
    let actual = text
        .lines()
        .filter(|line| line.starts_with("T|"))
        .collect::<Vec<_>>()
        .join("\n");
    assert_eq!(
        actual, EXPECTED,
        "four_state={four_state}, x_plane={x_plane}\n{text}"
    );
    text
}

fn stat(text: &str, key: &str) -> u64 {
    text.lines()
        .find_map(|line| {
            let start = line.find(key)? + key.len();
            line[start..]
                .split(|c: char| !c.is_ascii_digit())
                .next()?
                .parse()
                .ok()
        })
        .unwrap_or_else(|| panic!("missing counter {key}:\n{text}"))
}

#[test]
fn constant_branches_preserve_reference_control_flow_and_fallback() {
    for x_plane in [false, true] {
        let text = run(false, x_plane);
        assert!(
            stat(&text, "two_state_evals=") > 0,
            "two-state execution was bypassed:\n{text}"
        );
        assert!(
            text.lines()
                .any(|line| line.split_whitespace().nth(1) == Some("CaseJump:")),
            "relocation neighbor did not emit its jump table:\n{text}"
        );
    }
    let text = run(true, false);
    assert!(
        stat(&text, "vm_insns_total=") > 0,
        "four-state fallback was bypassed:\n{text}"
    );
}
