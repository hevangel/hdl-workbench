//! §6.18, §7.4.5: a typedef alias of a dimensioned typedef (`typedef A B;`,
//! `typedef A C [2];`, across packages) carries the base's unpacked dimensions
//! after its own (#312). Expected values are the reference simulator's.

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

/// Fixed, queue and associative aliases at module, package and local scope.
#[test]
fn typedef_alias_unpacked_dims() {
    const SRC: &str = r#"// §6.18/§7.4.5: typedef aliases of dimensioned typedefs.
typedef int fa_t [0:3];
typedef fa_t fa2_t;
typedef fa2_t fa3_t [2];
typedef int q_t [$];
typedef q_t q2_t;
typedef struct { int x; } el_t;
typedef el_t aa_t [string];
typedef aa_t aa2_t;
typedef aa2_t aa3_t;
package ap;
  typedef int pa_t [string];
  typedef pa_t pa2_t;
  pa2_t pv;
endpackage
typedef ap::pa2_t pa3_t;
module tb;
  fa2_t v2; fa3_t v3; q2_t q; aa3_t a; pa3_t p3;
  int n, s;
  function automatic int loc();
    aa3_t l; el_t e; e.x = 4; l["k"] = e;
    n = 0; foreach (l[k]) n++;
    return n * 10 + l["k"].x;
  endfunction
  initial begin
    el_t e;
    foreach (v2[i]) v2[i] = i + 1;
    n = 0; s = 0; foreach (v2[i]) begin n++; s += v2[i]; end
    $display("T|v2 n=%0d s=%0d size=%0d", n, s, $size(v2));
    foreach (v3[i, j]) v3[i][j] = i * 10 + j;
    n = 0; s = 0; foreach (v3[i, j]) begin n++; s += v3[i][j]; end
    $display("T|v3 n=%0d s=%0d dims=%0d %0d %0d", n, s, $dimensions(v3), $size(v3, 1), $size(v3, 2));
    q.push_back(4); q.push_back(9);
    $display("T|q size=%0d max=%0d", q.size(), q.max()[0]);
    e.x = 2; a["a"] = e; e.x = 5; a["b"] = e;
    n = 0; s = 0; foreach (a[k]) begin n++; s += a[k].x; end
    $display("T|a n=%0d s=%0d", n, s);
    ap::pv["k"] = 7; p3["j"] = 8; p3["i"] = 1;
    $display("T|pkg pv=%0d p3=%0d", ap::pv.num(), p3.num());
    $display("T|loc %0d", loc());
  end
endmodule
"#;
    assert_eq!(
        t_lines(SRC),
        [
            "T|a n=2 s=7",
            "T|loc 14",
            "T|pkg pv=1 p3=2",
            "T|q size=2 max=9",
            "T|v2 n=4 s=10 size=4",
            "T|v3 n=8 s=52 dims=3 2 4",
        ]
    );
}
