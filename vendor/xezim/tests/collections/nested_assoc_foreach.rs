//! §7.8, §12.7.3: `foreach (m[i, j])` over an associative array of associative
//! arrays walks each level's keys, each variable taking its level's key type
//! (#312). Expected values are the reference simulator's.

use xezim::simulate;

fn t_lines(src: &str) -> Vec<String> {
    let sim = simulate(src, 1_000).expect("simulate failed");
    let mut v: Vec<String> = sim
        .output
        .iter()
        .map(|o| o.message.clone())
        .filter(|m| m.starts_with("T|"))
        .collect();
    v.sort();
    v
}

/// String inner keys print as text.
#[test]
fn nested_assoc_foreach_int_string() {
    const SRC: &str = r#"module tb;
  int a [string];
  int m [int][string];
  initial begin
    a["xy"] = 1; m[1]["ab"] = 2;
    foreach (a[k]) $display("T|1d k=%s", k);
    foreach (m[i, j]) $display("T|2d i=%0d j=%s", i, j);
  end
endmodule
"#;
    assert_eq!(t_lines(SRC), ["T|1d k=xy", "T|2d i=1 j=ab",]);
}

/// Integer inner keys under a string-keyed outer level.
#[test]
fn nested_assoc_foreach_string_int() {
    const SRC: &str = r#"package p; int q1 [$]; endpackage
module tb;
  bit [7:0] bm [string][int];
  int lq [$];
  int n, s;
  initial begin
    bm["u"][2] = 8; bm["u"][9] = 1; bm["v"][0] = 3;
    foreach (bm[i, j]) begin n++; s += bm[i][j]; $display("T|i=%s j=%0d v=%0d", i, j, bm[i][j]); end
    $display("T|bm n=%0d s=%0d", n, s);
    p::q1.push_back(5); p::q1.push_back(2); lq.push_back(5); lq.push_back(2);
    $display("T|max pkg=%0d local=%0d", p::q1.max()[0], lq.max()[0]);
  end
endmodule
"#;
    assert_eq!(
        t_lines(SRC),
        [
            "T|bm n=3 s=12",
            "T|i=u j=2 v=8",
            "T|i=u j=9 v=1",
            "T|i=v j=0 v=3",
            "T|max pkg=5 local=5",
        ]
    );
}
