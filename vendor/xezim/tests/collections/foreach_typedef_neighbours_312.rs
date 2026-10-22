//! #312 neighbours: aliases, nested associative arrays, formals, returns and
//! class properties together. Expected values are the reference simulator's.

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

/// The survey's shapes in one module.
#[test]
fn foreach_typedef_neighbours() {
    const SRC: &str = r#"// #312 neighbours: alias chains, 2-D AAs, AA formals/returns, class AA props.
typedef struct { int x; } el_t;
typedef int fa_t [0:3];
typedef fa_t fa2_t;
typedef fa2_t fa3_t [2];          // alias with its own outer dim
typedef int q_t [$];
typedef q_t q2_t;
typedef el_t aa_t [string];
typedef aa_t aa2_t;
typedef int a2d_t [int][string];
typedef a2d_t a2d2_t;
package pp;
  typedef int pa_t [string];
  typedef pa_t pa2_t;
  pa2_t pv;
endpackage
typedef pp::pa2_t pa3_t;
class K;
  aa_t m_td; el_t m_dir [string]; aa2_t m_td2;
  function int sum();
    int s = 0;
    foreach (m_td[k]) s += m_td[k].x;
    foreach (m_dir[k]) s += 10 * m_dir[k].x;
    foreach (m_td2[k]) s += 100 * m_td2[k].x;
    return s;
  endfunction
endclass
module tb;
  fa2_t v2; fa3_t v3; q2_t q; aa2_t a; a2d_t m; a2d2_t m2; pa3_t p3;
  int n, s, m3d [int][string][int];
  bit [7:0] bm [string][int];
  function automatic aa_t mk(); el_t e; e.x = 3; mk["p"] = e; e.x = 4; mk["q"] = e; endfunction
  function automatic int cnt_in(input aa_t a); int c = 0; foreach (a[k]) c += a[k].x; return c; endfunction
  function automatic int cnt_in_dir(input el_t a [string]); int c = 0; foreach (a[k]) c += a[k].x; return c; endfunction
  task automatic mk_out(output aa_t a); el_t e; e.x = 6; a["r"] = e; endtask
  task automatic mk_out_dir(output el_t a [string]); el_t e; e.x = 7; a["r"] = e; endtask
  function automatic q_t mkq(); mkq = {5, 6, 7}; endfunction
  initial begin
    el_t e; aa_t r; el_t rd [string]; q_t qq; K k = new();
    foreach (v2[i]) v2[i] = i + 1;
    n = 0; s = 0; foreach (v2[i]) begin n++; s += v2[i]; end
    $display("T|v2 n=%0d s=%0d size=%0d", n, s, $size(v2));
    foreach (v3[i, j]) v3[i][j] = i * 10 + j;
    n = 0; s = 0; foreach (v3[i, j]) begin n++; s += v3[i][j]; end
    $display("T|v3 n=%0d s=%0d dims=%0d %0d %0d", n, s, $dimensions(v3), $size(v3, 1), $size(v3, 2));
    q.push_back(4); q.push_back(9);
    n = 0; s = 0; foreach (q[i]) begin n++; s += q[i]; end
    $display("T|q n=%0d s=%0d max=%0d", n, s, q.max()[0]);
    e.x = 2; a["a"] = e; e.x = 5; a["b"] = e;
    n = 0; s = 0; foreach (a[k]) begin n++; s += a[k].x; end
    $display("T|a2 n=%0d s=%0d num=%0d", n, s, a.num());
    m[1]["a"] = 10; m[1]["b"] = 20; m[2]["c"] = 30;
    n = 0; s = 0; foreach (m[i, j]) begin n++; s += m[i][j]; end
    $display("T|m n=%0d s=%0d", n, s);
    n = 0; foreach (m[i]) n++;
    $display("T|m outer n=%0d", n);
    m2[3]["x"] = 1; m2[4]["y"] = 2; m2[4]["z"] = 3;
    n = 0; s = 0; foreach (m2[i, j]) begin n++; s += m2[i][j]; $display("T|m2 i=%0d j=%s", i, j); end
    $display("T|m2 n=%0d s=%0d", n, s);
    m3d[1]["a"][5] = 1; m3d[1]["a"][6] = 2; m3d[2]["b"][7] = 4;
    n = 0; s = 0; foreach (m3d[i, j, l]) begin n++; s += m3d[i][j][l]; end
    $display("T|m3d n=%0d s=%0d", n, s);
    n = 0; s = 0; foreach (m3d[i, j]) n++;
    $display("T|m3d ij n=%0d", n);
    bm["u"][2] = 8; bm["u"][9] = 1; bm["v"][0] = 3;
    n = 0; s = 0; foreach (bm[i, j]) begin n++; s += bm[i][j]; end
    $display("T|bm n=%0d s=%0d", n, s);
    pp::pv["k"] = 7; p3["j"] = 8;
    n = 0; s = 0; foreach (pp::pv[k]) begin n++; s += pp::pv[k]; end
    foreach (p3[k]) begin n++; s += p3[k]; end
    $display("T|pa n=%0d s=%0d", n, s);
    r = mk();
    n = 0; s = 0; foreach (r[k]) begin n++; s += r[k].x; end
    $display("T|ret n=%0d s=%0d num=%0d", n, s, r.num());
    $display("T|in td=%0d dir=%0d", cnt_in(r), cnt_in_dir(r));
    mk_out(r);
    n = 0; s = 0; foreach (r[k]) begin n++; s += r[k].x; end
    $display("T|out n=%0d s=%0d", n, s);
    mk_out_dir(rd);
    n = 0; s = 0; foreach (rd[k]) begin n++; s += rd[k].x; end
    $display("T|outdir n=%0d s=%0d", n, s);
    qq = mkq();
    $display("T|retq size=%0d sum=%0d", qq.size(), qq.sum());
    e.x = 1; k.m_td["a"] = e; e.x = 2; k.m_dir["a"] = e; e.x = 3; k.m_td2["z"] = e;
    $display("T|cls sum=%0d", k.sum());
  end
endmodule
"#;
    assert_eq!(
        t_lines(SRC),
        [
            "T|a2 n=2 s=7 num=2",
            "T|bm n=3 s=12",
            "T|cls sum=321",
            "T|in td=7 dir=7",
            "T|m n=3 s=60",
            "T|m outer n=2",
            "T|m2 i=3 j=x",
            "T|m2 i=4 j=y",
            "T|m2 i=4 j=z",
            "T|m2 n=3 s=6",
            "T|m3d ij n=2",
            "T|m3d n=3 s=7",
            "T|out n=1 s=6",
            "T|outdir n=1 s=7",
            "T|pa n=2 s=15",
            "T|q n=2 s=13 max=9",
            "T|ret n=2 s=7 num=2",
            "T|retq size=3 sum=18",
            "T|v2 n=4 s=10 size=4",
            "T|v3 n=8 s=52 dims=3 2 4",
        ]
    );
}
