//! Reduction loops over an unpacked array, `for (k = ..) acc = acc | e(k);`
//! (and `&`), compile straight-line: `acc` carried in a register, `k` a
//! per-trip constant, one store of `acc` and stores that leave `k` where the
//! loop leaves it (marking every bit it changed). Values below were taken
//! from a reference simulator: OR and AND, x/z elements, the accumulator on
//! either side, stepped and down-counting loops, `for (int ...)`, a two-state
//! accumulator, an empty loop, observers of the accumulator and of the loop
//! variable's final value, and a clocked reduction over a memory.

use std::process::Command;

fn run(name: &str, src: &str) -> Vec<String> {
    let dir = std::env::temp_dir().join(format!("xezim_reduce_loop_{}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("mkdir");
    let sv = dir.join(format!("{name}.sv"));
    std::fs::write(&sv, src).expect("write sv");
    let out = Command::new(env!("CARGO_BIN_EXE_xezim"))
        .args(["--no-cache", "-s", "tb"])
        .arg(&sv)
        .output()
        .expect("run xezim");
    let text = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(out.status.success(), "{name} failed:\n{text}");
    text.lines()
        .filter_map(|l| l.strip_prefix("T| "))
        .map(str::to_string)
        .collect()
}

const T_RED: &str = r#"
// Reduction loops over unpacked arrays: OR/AND, x/z elements, both operand
// orders, stepped and down-counting loops, a shared module-scope `integer`,
// `for (int ...)`, a two-state accumulator, and observers of the result.
module tb;
  reg  [7:0] src [0:9];
  wire [7:0] arr [0:9];
  genvar g;
  generate for (g = 0; g < 10; g = g + 1) begin : c
    assign arr[g] = src[g];
  end endgenerate
  integer k1, k2, k3, k4, k6, k7;
  reg [7:0] o1, a1, o2, o3, o4, a2;
  bit [7:0] t1;
  integer kk1, kk2;
  reg clk = 0; reg [7:0] q = 0, accq; integer kq;
  always @(posedge clk) begin accq = 8'h00; for (kq = 0; kq < 10; kq = kq + 1) accq = accq | src[kq]; q <= accq; end
  always @* begin o1 = 0; for (k1 = 0; k1 < 10; k1 = k1 + 1) o1 = o1 | arr[k1]; kk1 = k1; end
  always @* begin a1 = 8'hff; for (k2 = 0; k2 < 10; k2 = k2 + 1) a1 = arr[k2] & a1; kk2 = k2; end
  always @* begin o2 = 8'h01; for (k3 = 9; k3 >= 0; k3 = k3 - 1) o2 = o2 | arr[k3]; end
  always @* begin o3 = 0; for (k4 = 1; k4 < 10; k4 = k4 + 2) o3 = (o3 | (arr[k4] >> 1)); end
  always @* begin o4 = 0; for (int j = 0; j < 10; j++) o4 = o4 | {arr[j][3:0], arr[9 - j][7:4]}; end
  always @* begin t1 = 0; for (k6 = 0; k6 < 10; k6 = k6 + 1) t1 = t1 | arr[k6]; end
  always @* begin a2 = 8'h0f; for (k7 = 0; k7 < 0; k7 = k7 + 1) a2 = a2 & arr[k7]; end
  integer i;
  task show;
    $display("T| t=%0t o1=%h a1=%h o2=%h o3=%h o4=%h t1=%h a2=%h k=%0d,%0d,%0d,%0d,%0d,%0d kk=%0d,%0d q=%h",
             $time, o1, a1, o2, o3, o4, t1, a2, k1, k2, k3, k4, k6, k7, kk1, kk2, q);
  endtask
  initial begin
    #1;
    for (i = 0; i < 10; i = i + 1) src[i] = 0;
    #1 clk = ~clk; show;
    src[3] = 8'h11; #1 show;
    src[7] = 8'h80; #1 show;
    src[3] = 8'h00; #1 show;
    for (i = 0; i < 10; i = i + 1) src[i] = 8'hff;
    #1 clk = ~clk; show;
    src[5] = 8'hf0; #1 show;
    src[2] = 8'b1x0z_1x0z; #1 show;
    src[2] = 8'hff; src[6] = 8'bzzzz_0000; #1 show;
    src[6] = 8'hff; #1 show;
    for (i = 0; i < 10; i = i + 1) src[i] = i * 8'h13;
    #1 clk = ~clk; show;
    $display("T| done");
    $finish;
  end
endmodule
"#;

#[test]
fn reduction_loops_match_reference() {
    assert_eq!(
        run("t_red", T_RED),
        [
            "t=2 o1=00 a1=00 o2=01 o3=00 o4=00 t1=00 a2=0f k=10,10,-1,11,10,0 kk=10,10 q=00",
            "t=3 o1=11 a1=00 o2=11 o3=08 o4=11 t1=11 a2=0f k=10,10,-1,11,10,0 kk=10,10 q=11",
            "t=4 o1=91 a1=00 o2=91 o3=48 o4=19 t1=91 a2=0f k=10,10,-1,11,10,0 kk=10,10 q=11",
            "t=5 o1=80 a1=00 o2=81 o3=40 o4=08 t1=80 a2=0f k=10,10,-1,11,10,0 kk=10,10 q=11",
            "t=6 o1=ff a1=ff o2=ff o3=7f o4=ff t1=ff a2=0f k=10,10,-1,11,10,0 kk=10,10 q=11",
            "t=7 o1=ff a1=f0 o2=ff o3=7f o4=ff t1=ff a2=0f k=10,10,-1,11,10,0 kk=10,10 q=11",
            "t=8 o1=ff a1=X0 o2=ff o3=7f o4=ff t1=ff a2=0f k=10,10,-1,11,10,0 kk=10,10 q=11",
            "t=9 o1=ff a1=x0 o2=ff o3=7f o4=ff t1=ff a2=0f k=10,10,-1,11,10,0 kk=10,10 q=11",
            "t=10 o1=ff a1=f0 o2=ff o3=7f o4=ff t1=ff a2=0f k=10,10,-1,11,10,0 kk=10,10 q=11",
            "t=11 o1=ff a1=00 o2=ff o3=7f o4=ff t1=ff a2=0f k=10,10,-1,11,10,0 kk=10,10 q=11",
            "done",
        ]
    );
}

/// The x-plane two-state executor (an x read re-run) stores into a
/// two-state variable: x/z bits must read 0 there, as the four-state
/// stores make them.
const T_TWO: &str = r#"
module tb;
  reg [7:0] r = 8'bxxxx_0000;
  bit [7:0] t, u;
  always @* begin t = 0; t = t | r; end
  always @* begin u = 8'h0f; u = u | r; u = u | 8'h01; end
  initial begin #1 $display("T| t=%b u=%b", t, u); r = 8'h0f; #1 $display("T| t=%b u=%b", t, u); r = 8'bz; #1 $display("T| t=%b u=%b", t, u); $finish; end
endmodule
"#;

#[test]
fn x_plane_stores_into_two_state_drop_xz() {
    assert_eq!(
        run("t_two", T_TWO),
        [
            "t=00000000 u=00000000",
            "t=00001111 u=00001111",
            "t=00000000 u=00001111",
        ]
    );
}
