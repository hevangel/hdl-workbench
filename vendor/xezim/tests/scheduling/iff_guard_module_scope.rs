//! §9.4.2.3 — `@(posedge clk iff g)` in module-scope processes: guards in
//! sub-module instances, generate blocks and over an automatic local of a
//! named block. These take the direct (no context swap) evaluation path.
//! Expected output from the reference simulator.
use xezim::simulate;

fn tagged(src: &str) -> Vec<String> {
    let sim = simulate(src, 10_000).expect("simulate failed");
    sim.output
        .iter()
        .filter_map(|o| o.message.strip_prefix("T|").map(str::to_string))
        .collect()
}

const SRC: &str = r#"
module sub(input logic clk, input int id);
  logic en = 0;
  int n = 0;
  int k;
  initial #(17 + 20*id) en = 1;
  initial begin : blk
    k = id + 1;
    repeat (2) begin
      @(posedge clk iff (en && n < 5 && k > 0));
      n++;
      $display("T|sub id=%0d t=%0t n=%0d k=%0d", id, $time, n, k);
    end
  end
endmodule
module top;
  logic clk = 0;
  logic en = 1;
  int cnt = 0;
  always #5 clk = ~clk;
  always @(posedge clk iff en) cnt <= cnt + 1;
  sub s0(clk, 0);
  sub s1(clk, 1);
  for (genvar g = 0; g < 2; g++) begin : gb
    logic ge = 0;
    initial #(30 + 10*g) ge = 1;
    initial begin
      @(posedge clk iff ge);
      $display("T|gen g=%0d t=%0t", g, $time);
    end
  end
  initial begin : blk
    automatic int lim = 3;
    @(posedge clk iff cnt == lim);
    $display("T|init lim t=%0t cnt=%0d", $time, cnt);
    #60 $display("T|cnt=%0d", cnt);
    $finish;
  end
endmodule
"#;

#[test]
fn module_scope_iff_guards() {
    // Lines printed in the same time step may come in either order.
    let mut got = tagged(SRC);
    got.sort();
    let mut want = [
        "sub id=0 t=25 n=1 k=1",
        "init lim t=35 cnt=3",
        "gen g=0 t=35",
        "sub id=0 t=35 n=2 k=1",
        "gen g=1 t=45",
        "sub id=1 t=45 n=1 k=2",
        "sub id=1 t=55 n=2 k=2",
        "cnt=9",
    ];
    want.sort();
    assert_eq!(got, want);
}
