//! §12.7.1, §12.7.3: a foreach or for loop variable shadows a same-named block
//! local only inside the loop; the local stays declared after it. Expected
//! values are the reference simulator's.

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

/// A class handle named like an earlier foreach index.
#[test]
fn block_local_survives_loop_var() {
    const SRC: &str = r#"class K; int v; endclass
module tb;
  int a [string];
  initial begin
    int n;
    K k = new();
    a["x"] = 1;
    foreach (a[k]) n++;
    k.v = 3;
    $display("T|v=%0d n=%0d", k.v, n);
  end
endmodule
"#;
    assert_eq!(t_lines(SRC), ["T|v=3 n=1",]);
}
