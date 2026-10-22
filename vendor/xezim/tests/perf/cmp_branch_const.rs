//! §11.4.5 and §12.4: compare-to-constant branch fusion preserves four-state
//! equality, sizing, signedness, loops and merged control-flow targets.
//! Expected T| lines were recorded from the reference simulator.
use std::process::Command;

const DESIGN: &str = r#"
module width_compare #(parameter W = 8)(input clk, input logic [W-1:0] value,
    output logic [3:0] known_flags, output logic [3:0] unknown_flags);
  localparam logic [W-1:0] ONE = 1;
  localparam logic [W-1:0] ZERO = 0;
  localparam logic [W-1:0] UNKNOWN = 'x;
  localparam logic [W-1:0] HIGHZ = 'z;
  always @(posedge clk) begin
    if (value == ONE) known_flags[0] <= 1; else known_flags[0] <= 0;
    if (value === ONE) known_flags[1] <= 1; else known_flags[1] <= 0;
    if (value == ZERO) known_flags[2] <= 1; else known_flags[2] <= 0;
    if (value === ZERO) known_flags[3] <= 1; else known_flags[3] <= 0;
  end
  always @(posedge clk) begin
    if (value == UNKNOWN) unknown_flags[0] <= 1; else unknown_flags[0] <= 0;
    if (value === UNKNOWN) unknown_flags[1] <= 1; else unknown_flags[1] <= 0;
    if (value == HIGHZ) unknown_flags[2] <= 1; else unknown_flags[2] <= 0;
    if (value === HIGHZ) unknown_flags[3] <= 1; else unknown_flags[3] <= 0;
  end
endmodule
module tb;
  logic clk = 0;
  always #5 clk = ~clk;
  logic [128:0] value = 0;
  logic signed [7:0] signed_value = 0;
  wire [3:0] known1, unknown1, known8, unknown8, known64, unknown64;
  wire [3:0] known65, unknown65, known129, unknown129;
  logic [7:0] signed_flags = 0;
  logic [7:0] loop_result = 0;
  logic eq_live = 0;
  logic [1:0] live_flags = 0;
  width_compare #(1) a(clk, value[0], known1, unknown1);
  width_compare #(8) b(clk, value[7:0], known8, unknown8);
  width_compare #(64) c(clk, value[63:0], known64, unknown64);
  width_compare #(65) d(clk, value[64:0], known65, unknown65);
  width_compare #(129) e(clk, value, known129, unknown129);
  always @(posedge clk) begin
    if (signed_value == -1) signed_flags[0] <= 1; else signed_flags[0] <= 0;
    if (signed_value === -1) signed_flags[1] <= 1; else signed_flags[1] <= 0;
    if (signed_value == 16'shffff) signed_flags[2] <= 1; else signed_flags[2] <= 0;
    if (signed_value === 16'shffff) signed_flags[3] <= 1; else signed_flags[3] <= 0;
    if (signed_value == 16'hffff) signed_flags[4] <= 1; else signed_flags[4] <= 0;
    if (signed_value === 16'hffff) signed_flags[5] <= 1; else signed_flags[5] <= 0;
    if (signed_value == 8'hff) signed_flags[6] <= 1; else signed_flags[6] <= 0;
    if (signed_value === 8'hff) signed_flags[7] <= 1; else signed_flags[7] <= 0;
  end
  always @(posedge clk) begin : loop_body
    int accum;
    accum = 0;
    for (int index = 0; index < 7; index++) begin
      case (value[2:0])
        0: accum = accum + index;
        1: accum = accum + 2 * index;
        2: accum = accum ^ index;
        3: accum = accum + 3;
        4: accum = accum + 4;
        default: accum = accum + 5;
      endcase
      if (value[7:0] === 8'hzz) accum = accum + 1;
    end
    loop_result <= accum;
  end
  always @(posedge clk) begin
    eq_live = value[7:0] == 8'h01;
    if (eq_live) live_flags[0] <= 1; else live_flags[0] <= 0;
    live_flags[1] <= eq_live;
  end
  initial begin
    for (int step = 0; step < 12; step++) begin
      @(negedge clk);
      case (step)
        0: value = 0;
        1: value = 1;
        2: value = 2;
        3: value = '1;
        4: value = 'x;
        5: value = 'z;
        6: value = {128'b0, 1'bx};
        7: value = {128'b0, 1'bz};
        8: value = 129'd1 << 64;
        9: value = 129'd1 << 128;
        10: value = (129'd1 << 128) | 129'd1;
        11: value = 129'hff;
      endcase
      signed_value = value[7:0];
      @(posedge clk);
      #1;
      $display("T|%0d %h/%h %h/%h %h/%h %h/%h %h/%h %h %h %b", step,
        known1, unknown1, known8, unknown8, known64, unknown64,
        known65, unknown65, known129, unknown129, signed_flags, loop_result, live_flags);
    end
    $finish;
  end
endmodule
"#;

const EXPECTED: &str = "T|0 c/0 c/0 c/0 c/0 c/0 00 15 00
T|1 3/0 3/0 3/0 3/0 3/0 00 2a 11
T|2 c/0 0/0 0/0 0/0 0/0 00 07 00
T|3 3/0 0/0 0/0 0/0 0/0 cf 23 00
T|4 0/2 0/2 0/2 0/2 0/2 00 23 x0
T|5 0/8 0/8 0/8 0/8 0/8 00 2a x0
T|6 0/2 0/0 0/0 0/0 0/0 00 23 x0
T|7 0/8 0/0 0/0 0/0 0/0 00 23 x0
T|8 c/0 c/0 c/0 0/0 0/0 00 15 00
T|9 c/0 c/0 c/0 c/0 0/0 00 15 00
T|10 3/0 3/0 3/0 3/0 0/0 00 2a 11
T|11 3/0 0/0 0/0 0/0 0/0 cf 23 00";

fn run(source: &str, expected: &str, enabled: bool, four_state: bool, merge: &str) -> String {
    run_backend(source, expected, enabled, four_state, merge, None)
}

fn run_backend(
    source: &str,
    expected: &str,
    enabled: bool,
    four_state: bool,
    merge: &str,
    backend: Option<&str>,
) -> String {
    static SEQ: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);
    let directory = std::env::temp_dir().join(format!(
        "xezim_cmp_branch_const_{}_{}",
        std::process::id(),
        SEQ.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
    ));
    std::fs::create_dir_all(&directory).unwrap();
    let path = directory.join("design.sv");
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
        .env("XEZIM_CASEJUMP_OFF", "1")
        .env("XEZIM_JIT", "0")
        .env_remove("XEZIM_TS_JIT")
        .env("XEZIM_NO_PARALLEL", "1")
        .env("XEZIM_VERBOSE", "1")
        .env("XEZIM_PROFILE_REPORT", "1")
        .env("XEZIM_EDGE_BLOCK_STATS", "1")
        .env("XEZIM_EDGE_MERGE", merge)
        .env_remove("XEZIM_AOT")
        .env_remove("XEZIM_CMP_BRANCH_CONST_OFF")
        .env_remove("XEZIM_TS_DENY");
    if !enabled {
        command.env("XEZIM_CMP_BRANCH_CONST_OFF", "1");
    }
    if four_state {
        command.env("XEZIM_TS_DENY", "Load,LoadS,LoadRng,LoadBit,LoadRngDyn");
    }
    if let Some(backend) = backend {
        command
            .env("XEZIM_JIT", "1")
            .env("XEZIM_JIT_VERBOSE", "1")
            .env("XEZIM_CACHE_DIR", directory.join("native-cache"))
            .env("XEZIM_AOT", if backend == "aot" { "1" } else { "0" });
    }
    let output = command.output().expect("run comparison fixture");
    let text = format!(
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let _ = std::fs::remove_dir_all(directory);
    assert!(
        output.status.success(),
        "comparison fixture failed:\n{text}"
    );
    let actual = text
        .lines()
        .filter(|line| line.starts_with("T|"))
        .collect::<Vec<_>>()
        .join("\n");
    assert_eq!(
        actual, expected,
        "enabled={enabled}, four_state={four_state}, merge={merge}\n{text}"
    );
    text
}

fn stat(text: &str, key: &str) -> u64 {
    text.lines()
        .find_map(|line| {
            line.find(key).map(|start| {
                line[start + key.len()..]
                    .split(|c: char| !c.is_ascii_digit())
                    .next()
                    .unwrap()
                    .parse()
                    .unwrap()
            })
        })
        .unwrap_or_else(|| panic!("missing counter {key}:\n{text}"))
}

fn contains_fused_opcode(text: &str) -> bool {
    text.lines()
        .any(|line| line.split_whitespace().nth(1) == Some("CmpBranchConst:"))
}

#[test]
fn equality_branch_constants_preserve_reference_values_in_both_executors() {
    for four_state in [true, false] {
        let enabled = run(DESIGN, EXPECTED, true, four_state, "0");
        let disabled = run(DESIGN, EXPECTED, false, four_state, "0");
        assert!(
            contains_fused_opcode(&enabled),
            "new comparison opcode was not exercised:\n{enabled}"
        );
        assert!(
            !contains_fused_opcode(&disabled),
            "disabled comparison opcode was emitted:\n{disabled}"
        );
        if four_state {
            assert!(
                stat(&enabled, "vm_insns_total=") < stat(&disabled, "vm_insns_total="),
                "comparison fusion failed to reduce four-state VM work"
            );
        }
        for enabled in [false, true] {
            run(SIGNED_DESIGN, SIGNED_EXPECTED, enabled, four_state, "0");
        }
    }
}

const KNOWN_DESIGN: &str = r#"
module decoder #(parameter BIAS = 0)(input clk, input [7:0] selector,
    input [7:0] seed, output logic [7:0] result);
  always @(posedge clk) begin
    if (selector == 8'd1) result <= seed + BIAS + 1;
    else if (selector === 8'd2) result <= seed + BIAS + 2;
    else begin
      case (selector)
        3: result <= seed + BIAS + 3;
        4: result <= seed + BIAS + 4;
        5: result <= seed + BIAS + 5;
        default: result <= seed + BIAS;
      endcase
    end
  end
endmodule
module tb;
  logic clk = 0;
  always #5 clk = ~clk;
  logic [7:0] selector = 0, seed = 0;
  wire [7:0] result [0:9];
  logic [7:0] proof;
  always_comb begin
    if (selector == 8'd1) proof = seed + 8'd1;
    else if (selector === 8'd2) proof = seed + 8'd2;
    else proof = seed;
  end
  for (genvar index = 0; index < 10; index++) begin : lanes
    decoder #(index) u(clk, selector, seed, result[index]);
  end
  initial begin
    for (int step = 0; step < 8; step++) begin
      @(negedge clk);
      selector = 8'(step);
      seed = 8'(32 + step * 17);
      @(posedge clk);
      #1;
      $display("T|known %0d %h %h %h %h", step, result[0], result[5], result[9], proof);
    end
    $finish;
  end
endmodule
"#;

const KNOWN_EXPECTED: &str = "T|known 0 20 25 29 20
T|known 1 32 37 3b 32
T|known 2 44 49 4d 44
T|known 3 56 5b 5f 53
T|known 4 68 6d 71 64
T|known 5 7a 7f 83 75
T|known 6 86 8b 8f 86
T|known 7 97 9c a0 97";

/// §10.4.2 and §12.4: fused targets relocate across merged NBA-only blocks.
#[test]
fn known_comparison_branches_keep_two_state_execution_and_block_merging() {
    let enabled = run(KNOWN_DESIGN, KNOWN_EXPECTED, true, false, "8");
    let disabled = run(KNOWN_DESIGN, KNOWN_EXPECTED, false, false, "8");
    let separate = run(KNOWN_DESIGN, KNOWN_EXPECTED, true, false, "0");
    assert!(
        contains_fused_opcode(&enabled),
        "merged comparison opcode missing:\n{enabled}"
    );
    assert!(!contains_fused_opcode(&disabled));
    for text in [&enabled, &disabled] {
        assert!(
            text.contains("[EDGE-MERGE] merged 10 blocks into 2 (cap 8)"),
            "comparison blocks stopped merging:\n{text}"
        );
        assert!(
            stat(text, "two_state_evals=") > 0,
            "known comparisons lost the two-state executor:\n{text}"
        );
    }
    assert!(!separate.contains("[EDGE-MERGE] merged"));
}

/// §11.4.5 and §12.4: both native backends execute fused branches;
/// a combinational runtime counter proves actual use.
#[cfg(feature = "jit")]
#[test]
fn constant_comparison_branches_execute_on_each_native_backend() {
    for backend in ["jit", "aot"] {
        for enabled in [false, true] {
            let text = run_backend(
                KNOWN_DESIGN,
                KNOWN_EXPECTED,
                enabled,
                true,
                "8",
                Some(backend),
            );
            assert_eq!(
                contains_fused_opcode(&text),
                enabled,
                "backend={backend}\n{text}"
            );
            assert!(
                stat(&text, "comb_jit native=") > 0,
                "backend={backend} did not execute native comparison code:\n{text}"
            );
            let prefix = if backend == "aot" {
                "[AOT] edge blocks compiled "
            } else {
                "[JIT] backend=cranelift compiled "
            };
            let compiled = text.lines().find_map(|line| {
                line.strip_prefix(prefix)
                    .and_then(|rest| rest.split('/').next())
                    .and_then(|value| value.parse::<usize>().ok())
            });
            assert!(
                compiled.is_some_and(|count| count > 0),
                "backend={backend} did not compile native comparison blocks:\n{text}"
            );
            // Merge admission requires successful two-state lowering. This
            // run denies that tier to isolate native execution, so separate
            // edge blocks are expected even with the merge cap enabled.
            assert!(
                !text.contains("[EDGE-MERGE] merged"),
                "backend={backend} unexpectedly merged TS-denied blocks:\n{text}"
            );
        }
        // Wide and signed-width shapes may decline native compilation, but
        // admitted X/Z constants and operands must retain exact semantics.
        // The OFF run establishes the same backend's unfused behavior.
        for enabled in [false, true] {
            run_backend(DESIGN, EXPECTED, enabled, true, "0", Some(backend));
            run_backend(
                SIGNED_DESIGN,
                SIGNED_EXPECTED,
                enabled,
                true,
                "0",
                Some(backend),
            );
        }
    }
}

// §11.4.5: sign extension preserves both value and X/Z planes for case equality.
const SIGNED_DESIGN: &str = r#"
module tb;
  logic clk = 0;
  always #5 clk = ~clk;
  logic signed [7:0] s8 = 0;
  logic signed [15:0] s16 = 0;
  logic signed [31:0] s32 = 0;
  logic signed [63:0] s64 = 0;
  logic [15:0] u16 = 0;
  logic [15:0] flags = 0;
  always @(posedge clk) begin
    if (s8 === s16) flags[0] <= 1; else flags[0] <= 0;
    if (s16 === s8) flags[1] <= 1; else flags[1] <= 0;
    if (s8 === u16) flags[2] <= 1; else flags[2] <= 0;
    if (u16 === s8) flags[3] <= 1; else flags[3] <= 0;
    if (s8 === 8'shff) flags[4] <= 1; else flags[4] <= 0;
    if (16'shffff === s8) flags[5] <= 1; else flags[5] <= 0;
    if (s8 === 16'shffff) flags[6] <= 1; else flags[6] <= 0;
    if (s8 === 16'sbxxxxxxxxx0000001) flags[7] <= 1; else flags[7] <= 0;
    if (s8 === 16'sbzzzzzzzzz0000001) flags[8] <= 1; else flags[8] <= 0;
    if (s32 === s64) flags[9] <= 1; else flags[9] <= 0;
    if (s64 === s32) flags[10] <= 1; else flags[10] <= 0;
    if (s32 === 64'shffffffff80000000) flags[11] <= 1; else flags[11] <= 0;
    if (s64 === 32'sh80000000) flags[12] <= 1; else flags[12] <= 0;
    if (s8 === s8) flags[13] <= 1; else flags[13] <= 0;
    if (s16 === s16) flags[14] <= 1; else flags[14] <= 0;
    if (s8 === 16'sbxxxxxxxxxxxxxxxx) flags[15] <= 1; else flags[15] <= 0;
  end
  initial begin
    for (int step = 0; step < 9; step++) begin
      @(negedge clk);
      case (step)
        0: begin s8 = 0; s16 = 0; s32 = 0; s64 = 0; end
        1: begin s8 = -1; s16 = -1; s32 = -1; s64 = -1; end
        2: begin s8 = -2; s16 = -2; s32 = 32'sh80000000; s64 = 64'shffffffff80000000; end
        3: begin s8 = 8'sbx0000001; s16 = 16'sbxxxxxxxxx0000001; s32 = 32'sh7fffffff; s64 = 64'sh000000007fffffff; end
        4: begin s8 = 8'sbz0000001; s16 = 16'sbzzzzzzzzz0000001; s32 = 32'sh80000000; s64 = 64'sh0000000080000000; end
        5: begin s8 = 'x; s16 = 'x; s32 = 'x; s64 = 'x; end
        6: begin s8 = 'z; s16 = 'z; s32 = 'z; s64 = 'z; end
        7: begin s8 = 8'h7f; s16 = 16'h007f; s32 = 32'sh7fffffff; s64 = 64'shffffffff7fffffff; end
        8: begin s8 = 8'h80; s16 = 16'hff80; s32 = 32'sh80000001; s64 = 64'shffffffff80000001; end
      endcase
      u16 = s16;
      @(posedge clk);
      #1;
      $display("T|signed %0d %h", step, flags);
    end
    $finish;
  end
endmodule
"#;

const SIGNED_EXPECTED: &str = "T|signed 0 660f
T|signed 1 6673
T|signed 2 7e03
T|signed 3 6683
T|signed 4 6903
T|signed 5 e603
T|signed 6 6603
T|signed 7 600f
T|signed 8 6603";
