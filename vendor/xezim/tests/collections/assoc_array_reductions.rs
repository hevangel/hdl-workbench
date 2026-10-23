//! §7.12.3 array reductions over an ASSOCIATIVE array — `aa.sum()`,
//! `product`, `and`, `or`, `xor`, with or without a `with` clause, at module
//! scope and on a class property. The reductions walked indices `0..size`,
//! which are not an associative array's keys, so `aa.sum()` over keys -2, 1
//! and 7 returned 0. They now walk the populated keys in key order (§7.8.4),
//! and `item.index` in a `with` clause is the key.
//!
//! Every expectation was cross-checked against the reference simulator.

use xezim::simulate;

#[test]
fn reductions_walk_associative_keys() {
    let src = r#"
class ar;
  int m[int];
  byte bm[string];
  function new(); m[3] = 4; m[10] = 5; bm["x"] = -3; bm["y"] = 7; endfunction
  function int ms(); return m.sum() with (item * 3); endfunction
endclass
module top;
  int aa[int];
  int sq[string];
  bit [3:0] nb[int];
  initial begin
    automatic ar o = new;
    aa[1] = 5; aa[7] = 6; aa[-2] = 10;
    sq["a"] = 2; sq["b"] = 3;
    nb[0] = 9; nb[5] = 9;
    $display("A %0d %0d %0d %0d %0d", aa.sum(), aa.product(), aa.and(), aa.or(), aa.xor());
    $display("B %0d %0d %0d", sq.sum(), nb.sum(), nb.sum() with (int'(item)));
    $display("C %0d %0d %0d", aa.sum() with (item * 2), sq.sum() with (item * 10), aa.sum() with (item.index));
    $display("D %0d %0d %0d %0d", o.m.sum(), o.m.sum() with (item + 1), o.ms(), o.bm.sum());
    $display("E %0d", o.m.product() with (item.index));
  end
endmodule
"#;
    let sim = simulate(src, 10).expect("simulate failed");
    let o: Vec<String> = sim.output.iter().map(|o| o.message.clone()).collect();
    for want in [
        "A 21 300 0 15 9",
        "B 5 2 18",
        "C 42 50 6",
        "D 9 11 27 4",
        "E 30",
    ] {
        assert!(
            o.iter().any(|l| l.trim() == want),
            "missing {want:?} in {o:?}"
        );
    }
}
