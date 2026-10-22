//! §26.3: a variable name that two packages declare has separate storage in
//! each, whether named `P::v` or through an import (#312). Expected values are
//! the reference simulator's.

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

/// Associative arrays, queues and scalars named alike in two packages.
#[test]
fn same_named_package_collections() {
    const SRC: &str = r#"package p1; int aa [string]; int sc; int q [$]; endpackage
package p2; int aa [string]; int sc; int q [$]; endpackage
module tb;
  import p2::*;
  initial begin
    p1::aa["a"] = 1; p2::aa["b"] = 2; p2::aa["c"] = 3; aa["d"] = 4;
    p1::sc = 5; p2::sc = 6; p1::q.push_back(1); q.push_back(2); q.push_back(3);
    $display("T|p1=%0d p2=%0d imp=%0d sc=%0d/%0d/%0d q=%0d/%0d", p1::aa.num(), p2::aa.num(), aa.num(), p1::sc, p2::sc, sc, p1::q.size(), p2::q.size());
  end
endmodule
"#;
    assert_eq!(t_lines(SRC), ["T|p1=1 p2=3 imp=3 sc=5/6/6 q=1/2",]);
}
