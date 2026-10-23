//! Packed memory arena (`PackedMem`, on by default; `XEZIM_PACKED_MEM=0`
//! turns it off). Every array in these tests has more than 100,000 cells, so
//! it skips per-element names and the arena admits it. Results are copied
//! into scalar signals because arena cells have no names of their own.
//!
//! * A blocking range store (`mem[i][7:0] = v`) must advance the program
//!   counter: the executor arm once ended in `continue` without it, so the
//!   C906 testbench's memory-image loop never finished.
//! * The non-blocking range store (`mem[i][3:0] <= v`) had the same missing
//!   step and hung the simulation.
use xezim::simulate;

fn u(sim: &xezim::compiler::Simulator, n: &str) -> u64 {
    sim.get_signal(n)
        .or_else(|| sim.get_signal(&format!("top.{}", n)))
        .unwrap_or_else(|| panic!("signal not found: {}", n))
        .to_u64()
        .unwrap_or_else(|| panic!("{} not u64-able", n))
}

fn bits(sim: &xezim::compiler::Simulator, n: &str) -> String {
    sim.get_signal(n)
        .or_else(|| sim.get_signal(&format!("top.{}", n)))
        .unwrap_or_else(|| panic!("signal not found: {}", n))
        .to_bin_string()
}

#[test]
fn packed_mem_range_store_terminates_and_keeps_upper_bits() {
    const SRC: &str = r#"
module top;
  logic [15:0] mem [0:131071];
  int i, errs = 0;
  initial begin
    for (i = 0; i < 256; i = i + 1) mem[i] = 16'hFFFF;
    for (i = 0; i < 256; i = i + 1) mem[i][7:0] = i[7:0];
    for (i = 0; i < 256; i = i + 1) if (mem[i] !== {8'hFF, i[7:0]}) errs++;
  end
endmodule
"#;
    let sim = simulate(SRC, 100).expect("simulate failed");
    assert_eq!(u(&sim, "errs"), 0, "partial range stores into packed cells");
}

#[test]
fn packed_mem_nba_range_store_matures_in_nba_region() {
    const SRC: &str = r#"
module top;
  logic clk = 0;
  logic [7:0] mem [0:131071];
  logic [7:0] pre, post;
  int i = 5;
  always @(posedge clk) begin
    mem[i][3:0] <= 4'h3;
    pre = mem[i];
  end
  initial begin
    mem[5] = 8'hff;
    #1 clk = 1;
    #1 post = mem[5];
    $finish;
  end
endmodule
"#;
    let sim = simulate(SRC, 100).expect("simulate failed");
    assert_eq!(
        u(&sim, "pre"),
        0xff,
        "NBA must not land before the NBA region"
    );
    assert_eq!(u(&sim, "post"), 0xf3, "NBA range store into a packed cell");
}

#[test]
fn packed_mem_element_types_widths_and_x() {
    const SRC: &str = r#"
module top;
  logic [7:0]  m8  [0:100000];
  logic [15:0] m16 [0:100000];
  bit   [31:0] b32 [0:100000];
  byte         sb  [0:100000];
  logic [63:0] m64 [0:100000];
  logic [7:0] init8, xs8, oob8, xidx8;
  logic [15:0] oob16, part16;
  logic [31:0] init32, x32;
  logic [63:0] v64;
  int neg, is_neg;
  int j;
  initial begin
    init8 = m8[0];
    init32 = b32[0];
    sb[7] = -3;
    neg = sb[7];
    is_neg = sb[7] < 0;
    m64[9] = 64'hfedc_ba98_7654_3210;
    v64 = m64[9];
    m8[6] = 8'hzx;
    xs8 = m8[6];
    b32[6] = 32'hx;
    x32 = b32[6];
    oob8 = m8[100001];
    oob16 = m16[-1];
    j = 'x;
    xidx8 = m8[j];
    m16[5] = 16'h1234;
    m16[5][15:8] = 8'h00;
    part16 = m16[5];
  end
endmodule
"#;
    let sim = simulate(SRC, 100).expect("simulate failed");
    assert_eq!(bits(&sim, "init8"), "xxxxxxxx", "4-state cells start x");
    assert_eq!(u(&sim, "init32"), 0, "2-state cells start 0");
    assert_eq!(u(&sim, "neg") as u32 as i32, -3, "byte cells are signed");
    assert_eq!(u(&sim, "is_neg"), 1);
    assert_eq!(u(&sim, "v64"), 0xfedc_ba98_7654_3210, "64-bit cells");
    assert_eq!(
        bits(&sim, "xs8"),
        "zzzzxxxx",
        "x/z survive in 4-state cells"
    );
    assert_eq!(u(&sim, "x32"), 0, "2-state cells drop x");
    assert_eq!(
        bits(&sim, "oob8"),
        "xxxxxxxx",
        "out-of-range read is x at element width"
    );
    assert_eq!(bits(&sim, "oob16"), "x".repeat(16));
    assert_eq!(bits(&sim, "xidx8"), "xxxxxxxx", "x index reads x");
    assert_eq!(u(&sim, "part16"), 0x0034, "range store into a 16-bit cell");
}

#[test]
fn packed_mem_force_release_cell() {
    const SRC: &str = r#"
module top;
  logic [7:0] mem [0:131071];
  logic [7:0] a = 8'h10;
  logic [7:0] f1, f2, r1, r2;
  initial begin
    force mem[20] = a + 8'h1;
    mem[20] = 8'h00;
    #1 f1 = mem[20];
    a = 8'h40;
    #1 f2 = mem[20];
    release mem[20];
    #1 r1 = mem[20];
    mem[20] = 8'h77;
    #1 r2 = mem[20];
  end
endmodule
"#;
    let sim = simulate(SRC, 100).expect("simulate failed");
    assert_eq!(u(&sim, "f1"), 0x11, "a forced cell ignores ordinary stores");
    assert_eq!(
        u(&sim, "f2"),
        0x41,
        "a forced expression tracks its operands"
    );
    assert_eq!(u(&sim, "r1"), 0x41, "a released variable keeps its value");
    assert_eq!(u(&sim, "r2"), 0x77, "a released cell takes stores again");
}

#[test]
fn packed_mem_readmem_writemem_round_trip() {
    let dir = std::env::temp_dir().join(format!("xezim_packed_mem_{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let hex = dir.join("m.hex");
    let src = format!(
        r#"
module top;
  logic [11:0] src_m [0:131071];
  logic [11:0] dst_m [0:131071];
  logic [11:0] d0, d3, d7;
  int j;
  initial begin
    for (j = 0; j < 8; j++) src_m[j] = 12'h100 + j * 3;
    $writememh("{0}", src_m, 0, 7);
    $readmemh("{0}", dst_m);
    d0 = dst_m[0];
    d3 = dst_m[3];
    d7 = dst_m[7];
  end
endmodule
"#,
        hex.display()
    );
    let sim = simulate(&src, 100).expect("simulate failed");
    assert_eq!(u(&sim, "d0"), 0x100);
    assert_eq!(u(&sim, "d3"), 0x109);
    assert_eq!(u(&sim, "d7"), 0x115);
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn packed_mem_whole_array_ops() {
    const SRC: &str = r#"
module top;
  logic [7:0] m8 [0:100000];
  logic [7:0] cp [0:100000];
  int total, sum, eq, neq, cnt;
  logic [7:0] c5, clast;
  initial begin
    foreach (m8[k]) m8[k] = k[7:0];
    total = 0;
    foreach (m8[k]) if (k < 10) total += m8[k];
    sum = m8.sum() with (int'(item));
    cp = m8;
    c5 = cp[5];
    clast = cp[100000];
    eq = cp == m8;
    cp[3] = 8'h00;
    neq = cp != m8;
    cnt = 0;
    foreach (cp[k]) if (cp[k] == 8'h07) cnt++;
  end
endmodule
"#;
    let sim = simulate(SRC, 100).expect("simulate failed");
    assert_eq!(u(&sim, "total"), 45, "foreach over a packed array");
    // 390 full cycles of 0..255 plus 0..160.
    assert_eq!(u(&sim, "sum"), 390 * (255 * 256 / 2) + (160 * 161 / 2));
    assert_eq!(u(&sim, "c5"), 5, "whole-array copy");
    assert_eq!(u(&sim, "clast"), (100000 % 256) as u64);
    assert_eq!(u(&sim, "eq"), 1);
    assert_eq!(u(&sim, "neq"), 1);
    assert_eq!(u(&sim, "cnt"), 391, "element compare in foreach");
}

#[test]
fn packed_mem_comb_reader_follows_nba_writes() {
    // A continuous assign reading a packed memory through a dynamic index
    // re-evaluates on every settle (its per-element reads never resolve to
    // table ids): an NBA into the memory must reach it.
    const SRC: &str = r#"
module top;
  logic clk = 0;
  logic [7:0] big [0:131071];
  logic [16:0] a = 1;
  logic [7:0] e;
  logic [7:0] e2;
  assign e = big[a];
  always @(posedge clk) big[1] <= big[1] + 1;
  initial begin
    big[1] = 5;
    #1 clk = 1;
    #1 e2 = e;
  end
endmodule
"#;
    let sim = simulate(SRC, 100).expect("simulate failed");
    assert_eq!(u(&sim, "e2"), 6, "comb reader sees the NBA'd cell");
}

#[test]
fn packed_mem_off_switch_matches_arena() {
    use std::process::Command;
    const SRC: &str = r#"
module arena_off;
  logic clk = 0;
  logic [7:0]  m8  [0:100000];
  logic [15:0] m16 [0:100000];
  bit   [31:0] b32 [0:100000];
  int i = 5, j;
  always @(posedge clk) begin
    m8[i][3:0] <= 4'h3;
    m16[i] <= m16[i] + 16'h101;
    b32[i] <= 32'hdead_beef;
  end
  initial begin
    $display("init %b %h", m8[0], b32[0]);
    m8[5] = 8'hff; m16[5] = 16'h1234;
    m8[5][7:4] = 4'ha;
    $display("oob %b %b", m8[100001], m16[-1]);
    #1 clk = 1;
    #1 $display("nba %h %h %h", m8[5], m16[5], b32[5]);
    force m8[20] = m16[5][7:0];
    m8[20] = 0;
    #1 $display("forced %h", m8[20]);
    release m8[20];
    for (j = 0; j < 4; j++) m8[j] = j * 7;
    $display("cells %h %h %h", m8[0], m8[3], m8[20]);
  end
endmodule
"#;
    let dir = std::env::temp_dir().join(format!("xezim_packed_off_{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let source = dir.join("arena_off.sv");
    std::fs::write(&source, SRC).unwrap();
    let run = |arena: &str| {
        let out = Command::new(env!("CARGO_BIN_EXE_xezim"))
            .args(["--no-cache", "-s", "arena_off", "--max-time", "100"])
            .arg(&source)
            .current_dir(&dir)
            .env("XEZIM_PACKED_MEM", arena)
            .output()
            .expect("run xezim");
        assert!(
            out.status.success(),
            "XEZIM_PACKED_MEM={} failed: {:?}",
            arena,
            out
        );
        String::from_utf8_lossy(&out.stdout).into_owned()
    };
    let on = run("1");
    let off = run("0");
    let _ = std::fs::remove_dir_all(&dir);
    assert!(
        on.contains("nba a3 1335 deadbeef"),
        "arena run output:\n{}",
        on
    );
    assert_eq!(on, off, "arena and off-switch transcripts must match");
}
