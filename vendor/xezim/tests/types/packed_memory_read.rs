//! A packed memory's element read is a slice at a runtime offset. It used
//! to load the WHOLE signal into a register and select from that, so one
//! read copied the entire memory and a deeper memory cost more per read;
//! the slice is taken in place now. The values here are the reference
//! simulator's, including the 7.4.6 / 11.5.1 unknown and out-of-range
//! addresses and a non-zero-based element dimension.
use std::path::PathBuf;
use std::process::Command;

fn run(name: &str, src: &str) -> String {
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("packed_memory_read");
    std::fs::create_dir_all(&dir).unwrap();
    let sv = dir.join(format!("{name}.sv"));
    std::fs::write(&sv, src).unwrap();
    // `--verbose`: the engine instruction counters read below.
    let output = Command::new(env!("CARGO_BIN_EXE_xezim"))
        .args([
            "--simulate",
            "--verbose",
            "-s",
            "t",
            "--no-cache",
            sv.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    let mut text = String::from_utf8_lossy(&output.stdout).to_string();
    text.push_str(&String::from_utf8_lossy(&output.stderr));
    assert!(output.status.success(), "run failed:\n{text}");
    text
}

#[test]
fn packed_memory_reads_take_the_slice_in_place() {
    let text = run(
        "xidx",
        r#"
// Unknown and out-of-range addresses on a packed memory (7.4.6 / 11.5.1).
module t;
  logic clk = 0; always #5 clk = ~clk;
  logic [15:0][31:0] mem;
  logic [7:0] a_ok = 8'd3, a_oob = 8'd40;
  logic [7:0] a_x;
  logic [31:0] q_ok, q_oob, q_x;
  logic [8:1][7:0] nz;            // non-zero-based element dim
  logic [3:0] i_nz = 4'd5;
  logic [7:0] q_nz;
  always @(posedge clk) begin
    q_ok  <= mem[a_ok];
    q_oob <= mem[a_oob];
    q_x   <= mem[a_x];
    q_nz  <= nz[i_nz];
  end
  initial begin
    for (int k = 0; k < 16; k++) mem[k] = 32'(k * 32'h11111111 + 5);
    for (int k = 1; k <= 8; k++) nz[k] = 8'(k * 8'h21);
    a_x = 8'bxxxx_xxxx;
    repeat (3) @(posedge clk);
    #1 $display("XI ok=%h oob=%h x=%h nz=%h", q_ok, q_oob, q_x, q_nz);
    $finish;
  end
endmodule
"#,
    );
    assert!(
        text.contains("XI ok=33333338 oob=xxxxxxxx x=xxxxxxxx nz=a5"),
        "wrong values:\n{text}"
    );
}

/// One read must not cost more because the memory is bigger. Both designs
/// write the same 256 entries and run the same 400 read cycles; only the
/// declared depth differs, so any growth is the read. Measured in simulated
/// instructions, which do not drift the way wall time does.
#[test]
fn read_cost_does_not_grow_with_memory_depth() {
    let mk = |depth: usize| {
        let mut s = String::from(
            r#"
module t;
  logic clk = 0; always #5 clk = ~clk;
  logic [__HI__:0][31:0] mem;
  logic [15:0] a; logic [31:0] q;
  always @(posedge clk) begin q <= mem[a]; a <= a + 16'd7; end
  initial begin
    for (int k = 0; k < 256; k++) mem[k] = 32'(k + 7);
    a = 16'd3;
    repeat (400) @(posedge clk);
    #1 $display("R q=%h", q);
    $finish;
  end
endmodule
"#,
        );
        s = s.replace("__HI__", &(depth - 1).to_string());
        s
    };
    let small = run("depth_small", &mk(256));
    let big = run("depth_big", &mk(4096));
    let insns = |t: &str| -> u64 {
        t.lines()
            .find_map(|l| l.split("vm_insns_total=").nth(1))
            .and_then(|s| s.split_whitespace().next())
            .and_then(|s| s.parse().ok())
            .unwrap_or(0)
    };
    let (s, b) = (insns(&small), insns(&big));
    assert!(s > 0 && b > 0, "no instruction counts:\n{small}\n{big}");
    // Same writes, same reads: a 16x deeper memory must not cost more.
    assert!(
        b < s + s / 4,
        "read cost grew with memory depth: {s} -> {b} instructions"
    );
}
