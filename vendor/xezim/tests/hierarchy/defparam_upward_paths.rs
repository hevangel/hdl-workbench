//! §23.10.1/§23.8: a `defparam` path resolves like any hierarchical name,
//! including upward: its first segment may name an instance declared in an
//! enclosing scope (`m.l.P`, a sibling `l2.P`, an uncle `m2.l2.P`) or the
//! definition name of an enclosing instance (`mid2.l.P`). Only paths starting
//! at a child of the declaring scope or at a top were applied; the others
//! were silently ignored and the parameters kept their defaults. Expected
//! output cross-checked against the reference simulator.

use xezim::simulate;

const SRC: &str = r#"
module leaf;
  parameter P = 1;
  initial #1 $display("%m P=%0d", P);
endmodule
module cfg;
  defparam m.l.P = 3;
endmodule
module cfg2;
  defparam mid2.l.P = 4;
endmodule
module cfg3;
  defparam m2.l2.P = 7;
  defparam l2.P = 8;
endmodule
module mid;
  leaf l();
  leaf l2();
  cfg c();
  cfg3 c3();
endmodule
module mid2;
  leaf l();
  leaf l2();
  cfg2 c();
endmodule
module mid3;
  leaf l();
  defparam l.P = 6;
endmodule
module top;
  mid m();
  mid2 m2();
  mid3 m3();
endmodule
"#;

#[test]
fn upward_defparam_paths_apply() {
    let sim = simulate(SRC, 100).expect("simulate failed");
    let mut msgs: Vec<String> = sim.output.iter().map(|o| o.message.clone()).collect();
    msgs.sort();
    assert_eq!(
        msgs,
        [
            "top.m.l P=3",
            "top.m.l2 P=8",
            "top.m2.l P=4",
            "top.m2.l2 P=7",
            "top.m3.l P=6",
        ],
    );
}
