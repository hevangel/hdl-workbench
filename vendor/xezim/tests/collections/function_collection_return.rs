//! §13.4, §13.4.1: a function returning a queue, dynamic or associative array
//! through a typedef — filled through its return variable or by `return` —
//! hands its elements to the caller, and `f().size()` / `f().num()` work on the
//! result (#312). Expected values are the reference simulator's.

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

/// Return variable and `return` forms, int and struct elements.
#[test]
fn function_returns_collection() {
    const SRC: &str = r#"typedef struct { int x; } el_t;
typedef el_t aa_t [string];
typedef int ia_t [string];
typedef int q_t [$];
typedef int d_t [];
module tb;
  function automatic ia_t rl(); ia_t t; t["p"] = 3; t["q"] = 4; return t; endfunction
  function automatic ia_t fn(); fn["p"] = 3; fn["q"] = 4; endfunction
  function automatic aa_t rls(); aa_t t; el_t e; e.x = 3; t["p"] = e; e.x = 4; t["q"] = e; return t; endfunction
  function automatic aa_t fns(); el_t e; e.x = 3; fns["p"] = e; e.x = 4; fns["q"] = e; endfunction
  function automatic q_t fq(); fq = {5, 6, 7}; endfunction
  function automatic q_t fq2(); fq2.push_back(5); fq2.push_back(6); endfunction
  function automatic d_t fd(); fd = new[3]; fd[1] = 4; endfunction
  initial begin
    ia_t a; aa_t s; q_t q; d_t d; int t;
    a = rl(); $display("T|rl num=%0d q=%0d", a.num(), a["q"]);
    a.delete(); a = fn(); $display("T|fn num=%0d q=%0d", a.num(), a["q"]);
    s = rls(); t = 0; foreach (s[k]) t += s[k].x; $display("T|rls num=%0d s=%0d", s.num(), t);
    s.delete(); s = fns(); t = 0; foreach (s[k]) t += s[k].x; $display("T|fns num=%0d s=%0d", s.num(), t);
    q = fq(); $display("T|fq size=%0d sum=%0d", q.size(), q.sum());
    q = fq2(); $display("T|fq2 size=%0d sum=%0d", q.size(), q.sum());
    d = fd(); $display("T|fd size=%0d d1=%0d", d.size(), d[1]);
    $display("T|direct num=%0d size=%0d", rl().num(), fq().size());
  end
endmodule
"#;
    assert_eq!(
        t_lines(SRC),
        [
            "T|direct num=2 size=3",
            "T|fd size=3 d1=4",
            "T|fn num=2 q=4",
            "T|fns num=2 s=7",
            "T|fq size=3 sum=18",
            "T|fq2 size=2 sum=11",
            "T|rl num=2 q=4",
            "T|rls num=2 s=7",
        ]
    );
}

/// Returned arrays passed on as input formals.
#[test]
fn function_collection_formals_and_returns() {
    const SRC: &str = r#"typedef struct { int x; } el_t;
typedef el_t aa_t [string];
typedef int ia_t [string];
typedef int q_t [$];
module tb;
  function automatic ia_t mki(); mki["p"] = 3; mki["q"] = 4; endfunction
  function automatic aa_t mks(); el_t e; e.x = 3; mks["p"] = e; e.x = 4; mks["q"] = e; endfunction
  function automatic int in_i(input int a [string]); int c = 0; foreach (a[k]) c += a[k]; return c; endfunction
  function automatic int in_s(input el_t a [string]); int c = 0; foreach (a[k]) c += a[k].x; return c; endfunction
  function automatic int in_st(input aa_t a); int c = 0; foreach (a[k]) c += a[k].x; return c; endfunction
  task automatic out_i(output int a [string]); a["r"] = 6; endtask
  task automatic out_s(output el_t a [string]); el_t e; e.x = 7; a["r"] = e; endtask
  function automatic q_t mkq(); mkq = {5, 6, 7}; endfunction
  function automatic q_t mkq2(); q_t r; r = {5, 6, 7}; return r; endfunction
  initial begin
    ia_t ri; aa_t rs; int oi [string]; el_t os [string]; q_t qq; int s;
    ri = mki(); $display("T|ret-int num=%0d", ri.num());
    rs = mks(); s = 0; foreach (rs[k]) s += rs[k].x; $display("T|ret-struct num=%0d s=%0d", rs.num(), s);
    begin ia_t ri2; aa_t rs2; el_t e; ri2["a"]=3; ri2["b"]=4; e.x=3; rs2["a"]=e; e.x=4; rs2["b"]=e; $display("T|in2 int=%0d struct=%0d struct-td=%0d", in_i(ri2), in_s(rs2), in_st(rs2)); end
    $display("T|in int=%0d struct=%0d struct-td=%0d", in_i(ri), in_s(rs), in_st(rs));
    out_i(oi); $display("T|out-int num=%0d v=%0d", oi.num(), oi["r"]);
    out_s(os); $display("T|out-struct num=%0d v=%0d", os.num(), os["r"].x);
    qq = mkq(); $display("T|retq size=%0d sum=%0d", qq.size(), qq.sum());
    qq = mkq2(); $display("T|retq2 size=%0d sum=%0d", qq.size(), qq.sum());
  end
endmodule
"#;
    assert_eq!(
        t_lines(SRC),
        [
            "T|in int=7 struct=7 struct-td=7",
            "T|in2 int=7 struct=7 struct-td=7",
            "T|out-int num=1 v=6",
            "T|out-struct num=1 v=7",
            "T|ret-int num=2",
            "T|ret-struct num=2 s=7",
            "T|retq size=3 sum=18",
            "T|retq2 size=3 sum=18",
        ]
    );
}
