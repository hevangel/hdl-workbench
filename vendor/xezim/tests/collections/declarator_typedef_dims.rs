//! §7.4.5, §6.18: a variable's declarator dimensions come before its
//! typedef's, multi-dimensional package arrays keep their shape, and a typedef'd
//! collection declared in a generate block is a collection (#312). Expected
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

/// Package `int v[0:1][0:3]` and `fa_t v[0:1]`.
#[test]
fn package_multi_dim_arrays() {
    const SRC: &str = r#"package p2;
  int v_dir [0:1][0:3];
  typedef int fa_t [0:3];
  fa_t v [0:1];
endpackage
module tb;
  import p2::*;
  int m_dir [0:1][0:3];
  int n, s;
  initial begin
    p2::v_dir[1][3] = 4; p2::v_dir[0][0] = 1; m_dir[1][3] = 4; p2::v[1][2] = 5;
    n = 0; s = 0; foreach (p2::v_dir[i, j]) begin n++; s += p2::v_dir[i][j]; end
    $display("T|pkg n=%0d s=%0d r=%0d", n, s, p2::v_dir[1][3]);
    n = 0; s = 0; foreach (m_dir[i, j]) begin n++; s += m_dir[i][j]; end
    $display("T|mod n=%0d s=%0d", n, s);
    n = 0; s = 0; foreach (v_dir[i, j]) begin n++; s += v_dir[i][j]; end
    $display("T|imp n=%0d s=%0d", n, s);
    n = 0; s = 0; foreach (p2::v[i, j]) begin n++; s += p2::v[i][j]; end
    $display("T|td n=%0d s=%0d r=%0d dims=%0d", n, s, p2::v[1][2], $dimensions(p2::v));
  end
endmodule
"#;
    assert_eq!(
        t_lines(SRC),
        [
            "T|imp n=8 s=5",
            "T|mod n=8 s=4",
            "T|pkg n=8 s=5 r=4",
            "T|td n=8 s=5 r=5 dims=3",
        ]
    );
}

/// Typedef'd associative and fixed arrays in a generate-for block.
#[test]
fn generate_block_typedef_collections() {
    const SRC: &str = r#"package gp; typedef int ia_t [string]; endpackage
typedef int uia_t [string];
typedef int fa_t [0:2];
module tb;
  for (genvar g = 0; g < 1; g++) begin : gb
    gp::ia_t a; uia_t b; fa_t c;
    initial begin
      int n;
      a["x"] = 1; b["y"] = 2;
      n = 0; foreach (a[k]) n++; $display("T|pkg n=%0d", n);
      n = 0; foreach (b[k]) n++; $display("T|unit n=%0d", n);
      n = 0; foreach (c[k]) n++; $display("T|fixed n=%0d", n);
    end
  end
endmodule
"#;
    assert_eq!(t_lines(SRC), ["T|fixed n=3", "T|pkg n=1", "T|unit n=1",]);
}
