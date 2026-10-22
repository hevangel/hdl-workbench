//! §10.6.2 / §9.4.2: edge aliases on a vector one `assign v[A +: N] =
//! {N{b}}` replicates (see `replicated_clock_edges`) are shared only while
//! every bit follows that one driver. A procedural `force` of one bit makes
//! it differ from the others, so a vector some `force` targets keeps one
//! alias per bit.
//!
//! Next to that rule: a forced bit holds against part-select drivers
//! (§10.6.2), and a gate or pull on one bit resolves with the vector's
//! driver by strength (§28.12). Expected values are the reference
//! simulator's.
use std::path::PathBuf;
use std::process::Command;

fn run(tag: &str, src: &str, top: &str, env: &[(&str, &str)]) -> String {
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("replicated_edges_forced");
    std::fs::create_dir_all(&dir).unwrap();
    let sv = dir.join(format!("{tag}.sv"));
    std::fs::write(&sv, src).unwrap();
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_xezim"));
    for (k, v) in env {
        cmd.env(k, v);
    }
    let output = cmd
        .args(["--simulate", "-s", top, "--no-cache", sv.to_str().unwrap()])
        .output()
        .unwrap();
    let mut text = String::from_utf8_lossy(&output.stdout).to_string();
    text.push_str(&String::from_utf8_lossy(&output.stderr));
    assert!(output.status.success(), "run failed:\n{text}");
    text
}

fn t_lines(text: &str) -> Vec<&str> {
    text.lines()
        .filter_map(|l| l.find("T|").map(|i| &l[i..]))
        .collect()
}

const FORCED: &str = r#"
`timescale 1ns/1ns
module tb;
  logic clk = 0;
  always #5 clk = ~clk;
  wire [3:0] fv;
  assign fv[3:0] = {4{clk}};
  wire [3:0] sv;
  assign sv[3:0] = {4{clk}};
  wire [3:0] fw;
  assign fw = {4{clk}};
  int f [4];
  always @(posedge fv[2]) f[0]++;
  always @(posedge fw[2]) f[1]++;
  always @(fv[2]) f[2]++;
  always @(posedge sv[2]) f[3]++;
  initial begin
    #12 force fv[2] = 1'b0; force fw[2] = 1'b0;
    #23 release fv[2]; release fw[2];
    #28;
    $display("T|force %0d %0d %0d %b", f[0], f[1], f[2], fv);
    $finish;
  end
  initial begin #20 $display("T|mid %b %b", fv, fw); end
endmodule
"#;

#[test]
fn forced_vector_keeps_an_alias_per_bit() {
    let o = run("forced", FORCED, "tb", &[("XEZIM_DUMP_EDGE_SENS", "1")]);
    assert!(
        !o.contains("fv[2] shares"),
        "a forced vector must keep its own aliases:\n{o}"
    );
    assert!(
        o.contains("sv[2] shares the alias of bit 0"),
        "an unforced replicated vector still shares:\n{o}"
    );
    // The whole-vector form already follows the force.
    assert!(
        t_lines(&o)
            .iter()
            .any(|l| l.starts_with("T|force ") && l.split(' ').nth(2) == Some("4")),
        "{o}"
    );
}

#[test]
fn force_on_a_bit_of_a_part_select_driven_vector() {
    let o = run("forced_values", FORCED, "tb", &[]);
    assert_eq!(
        t_lines(&o),
        ["T|mid 1011 1011", "T|force 4 4 9 0000"],
        "{o}"
    );
}

#[test]
fn gate_and_pull_on_one_bit_of_a_driven_vector() {
    let src = r#"
`timescale 1ns/1ns
module tb;
  logic clk = 0, c2 = 0;
  always #5 clk = ~clk;
  always #7 c2 = ~c2;
  wire [3:0] gv;
  assign gv[3:0] = {clk, clk, clk, clk};
  buf (gv[2], c2);
  wire [3:0] wv;
  assign (weak0, weak1) wv[3:0] = {clk, clk, clk, clk};
  pullup (wv[1]);
  int g [2], w [3];
  always @(posedge gv[2]) g[0]++;
  always @(gv) g[1]++;
  always @(posedge wv[1]) w[0]++;
  always @(posedge wv[0]) w[1]++;
  always @(wv[1]) w[2]++;
  initial begin
    #63;
    $display("T|gate %0d %0d %b", g[0], g[1], gv);
    $display("T|pull %0d %0d %0d %b", w[0], w[1], w[2], wv);
    $finish;
  end
endmodule
"#;
    let o = run("drivers", src, "tb", &[]);
    assert_eq!(
        t_lines(&o),
        ["T|gate 9 20 0000", "T|pull 1 6 1 0010"],
        "{o}"
    );
}

/// §10.6.2: `force` of one bit holds against a part-select driver of the
/// whole vector whatever its right-hand side — a concatenation or a
/// conditional, not only a replication.
#[test]
fn force_on_a_bit_holds_against_select_drivers() {
    for (tag, drv) in [
        ("cat", "assign fv[3:0] = {clk, clk, clk, clk};"),
        ("cond", "assign fv[3:0] = clk ? 4'hf : 4'h0;"),
    ] {
        let src = format!(
            r#"
`timescale 1ns/1ns
module tb;
  logic clk = 0;
  always #5 clk = ~clk;
  wire [3:0] fv;
  {drv}
  int f [3];
  always @(posedge fv[2]) f[0]++;
  always @(fv[2]) f[1]++;
  always @(fv) f[2]++;
  initial begin
    #12 force fv[2] = 1'b0;
    #23 release fv[2];
    #28;
    $display("T|force %0d %0d %0d %b", f[0], f[1], f[2], fv);
    $finish;
  end
  initial begin #20 $display("T|mid %b", fv); end
endmodule
"#
        );
        let o = run(tag, &src, "tb", &[]);
        assert_eq!(
            t_lines(&o),
            ["T|mid 1011", "T|force 4 9 13 0000"],
            "{tag}: {o}"
        );
    }
}
