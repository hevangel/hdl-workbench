//! §7.8.6: reading a NONEXISTENT associative-array element yields the element
//! type's default — 0 for a 2-state type, x for a 4-state one — with the
//! element's width and signedness. So `seen[k]++` on a new key of `int
//! seen[int]` is 1 and `integer iseen[int]; iseen[k]++` stays x. Three gaps:
//!
//! * the default carried no signedness, so `seen[k] -= 2` on a new `int` key
//!   stored 4294967294 and `seen[k] < 0` was false;
//! * a CLASS-property array read a missing key as a 32-bit 0 whatever the
//!   element type (`integer`/`logic` counted from 0 instead of staying x);
//! * a subroutine- or block-local array read a missing key as a 1-bit x, so
//!   every counter built with `++` / `+=` came out x.
//!
//! Writes had the mirror-image gap: an element kept the RHS's signedness, so
//! `bit [3:0] aa[int]; aa[k] = 9` read back -7 and `byte b[int]; b[k] =
//! 8'hF0` read 240; an element now takes its declared type's signedness (and
//! a subroutine-local array its declared width).
//!
//! Every expectation was cross-checked against the reference simulator.

use xezim::simulate;

#[test]
fn missing_key_reads_element_type_default() {
    let src = r#"
class ac7;
  int seen[int];
  integer iseen[int];
  int sseen[string];
  logic [7:0] lseen[int];
  byte bq[int];
  function void bump(int k);
    seen[k]++; iseen[k]++; sseen[$sformatf("k%0d", k)] += 5; lseen[k]--; bq[k] -= 1;
  endfunction
  function int m(int n);
    int h[int];
    for (int i = 0; i < n; i++) h[i % 2]++;
    return h[0] * 10 + h[1];
  endfunction
endclass
module top;
  int seen[int];
  integer iseen[int];
  int sk[string];
  logic [7:0] lg[int];
  bit [7:0] bb[int];
  byte by[string];
  shortint ash[string];
  int k;
  function int f6(); int h[string]; h["a"]--; return h["a"]; endfunction
  function automatic void g7(output int o); int h[int]; h[2]++; h[2]++; o = h[2]; endfunction
  task automatic t1(output int o); int h[int]; h[3]++; h[3] += 2; o = h[3]; endtask
  initial begin
    automatic ac7 o = new;
    int r;
    k = 5;
    seen[k]++; iseen[k]++; sk["a"]++; lg[k]++; bb[k]--; by["z"] -= 3; ash["q"]--;
    $display("A %0d %0d %0d %0d %0d %0d %0d", seen[k], iseen[k], sk["a"], lg[k], bb[k], by["z"], ash["q"]);
    seen[7] += 4; seen[8] -= 2; seen[9] *= 3; seen[10] |= 6; seen[11] <<= 1; ++seen[12]; --seen[13];
    $display("B %0d %0d %0d %0d %0d %0d %0d", seen[7], seen[8], seen[9], seen[10], seen[11], seen[12], seen[13]);
    $display("C %0d %0d", seen[13] < 0, seen[99]);
    o.bump(3); o.bump(3);
    $display("D %0d %0d %0d %0d %0d", o.seen[3], o.iseen[3], o.sseen["k3"], o.lseen[3], o.bq[3]);
    o.seen[4]++; o.sseen["q"] += 2;
    $display("E %0d %0d", o.seen[4], o.sseen["q"]);
    begin
      automatic int loc[int];
      automatic int locs[string];
      loc[1]++; loc[1]++; locs["x"] += 9; loc[2] -= 4;
      $display("F %0d %0d %0d n=%0d", loc[1], locs["x"], loc[2], loc.num());
    end
    g7(r);
    $display("G %0d %0d", r, f6());
    t1(r);
    $display("H %0d %0d", r, o.m(5));
  end
endmodule
"#;
    let sim = simulate(src, 10).expect("simulate failed");
    let o: Vec<String> = sim.output.iter().map(|o| o.message.clone()).collect();
    for want in [
        "A 1 x 1 x 255 -3 -1",
        "B 4 -2 0 6 0 1 -1",
        "C 1 0",
        "D 2 x 10 x -2",
        "E 1 2",
        "F 2 9 -4 n=2",
        "G 2 -1",
        "H 3 32",
    ] {
        assert!(
            o.iter().any(|l| l.trim() == want),
            "missing {want:?} in {o:?}"
        );
    }
}

#[test]
fn stores_take_declared_element_type() {
    let src = r#"
class c8;
  byte bm[string];
  bit [3:0] nm[int];
  function new(); bm["x"] = -3; nm[1] = 9; endfunction
endclass
module top;
  bit [3:0] nb[int];
  int ai[int];
  byte by[int];
  task automatic tl(output int o1, output int o2);
    logic [3:0] h[int];
    byte g[string];
    h[1] = 8'hEF; g["k"] = 8'hF0; g["k"] += 1;
    o1 = h[1]; o2 = g["k"];
  endtask
  initial begin
    automatic c8 o = new;
    int r1, r2;
    nb[0] = 9; ai[1] = 3'd7; by[2] = 8'hF0;
    $display("A %0d %0d %0d %0d %0d", nb[0], ai[1], by[2], nb[0] > 8, by[2] < 0);
    $display("B %0d %0d %0d", o.bm["x"], o.nm[1], o.nm[1] > 8);
    by[2] += 1; nb[0] += 9;
    $display("C %0d %0d", by[2], nb[0]);
    tl(r1, r2);
    $display("D %0d %0d", r1, r2);
  end
endmodule
"#;
    let sim = simulate(src, 10).expect("simulate failed");
    let o: Vec<String> = sim.output.iter().map(|o| o.message.clone()).collect();
    for want in ["A 9 7 -16 1 1", "B -3 9 1", "C -15 2", "D 15 -15"] {
        assert!(
            o.iter().any(|l| l.trim() == want),
            "missing {want:?} in {o:?}"
        );
    }
}
