//! §13.5: an associative formal declared through a typedef is bound as an
//! associative array in a task, and unpacked-struct elements are copied in
//! and back out (#312). Expected values are the reference simulator's.

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

/// `output`/`ref` formals of a typedef'd associative array of structs.
#[test]
fn assoc_output_formal_typedef() {
    const SRC: &str = r#"typedef struct { int x; } el_t;
typedef el_t aa_t [string];
module tb;
  task automatic mk_out(output aa_t a); el_t e; e.x = 6; a["r"] = e; $display("T|in-task num=%0d rx=%0d", a.num(), a["r"].x); endtask
  task automatic mk_out2(output el_t a [string]); el_t e; e.x = 6; a["r"] = e; $display("T|in-task2 num=%0d rx=%0d", a.num(), a["r"].x); endtask
  task automatic mk_ref(ref aa_t a); el_t e; e.x = 6; a["r"] = e; endtask
  initial begin
    aa_t r; el_t e; int n, s;
    e.x = 3; r["p"] = e;
    mk_out(r); n = 0; s = 0; foreach (r[k]) begin n++; s += r[k].x; $display("T|k=%s", k); end
    $display("T|out1 n=%0d s=%0d", n, s);
    e.x = 3; r["p"] = e;
    mk_out2(r); n = 0; s = 0; foreach (r[k]) begin n++; s += r[k].x; $display("T|k=%s", k); end
    $display("T|out2 n=%0d s=%0d", n, s);
    r.delete(); e.x = 3; r["p"] = e;
    mk_ref(r); n = 0; s = 0; foreach (r[k]) begin n++; s += r[k].x; end
    $display("T|ref n=%0d s=%0d", n, s);
  end
endmodule
"#;
    assert_eq!(
        t_lines(SRC),
        [
            "T|in-task num=1 rx=6",
            "T|in-task2 num=1 rx=6",
            "T|k=r",
            "T|k=r",
            "T|out1 n=1 s=6",
            "T|out2 n=1 s=6",
            "T|ref n=2 s=9",
        ]
    );
}

/// An output formal replaces the actual's contents.
#[test]
fn assoc_output_formal_replaces_contents() {
    const SRC: &str = r#"typedef struct { int x; } el_t;
typedef el_t aa_t [string];
module tb;
  function automatic int cnt_in(input aa_t a); int c = 0; foreach (a[k]) c += a[k].x; return c; endfunction
  task automatic mk_out(output aa_t a); el_t e; e.x = 6; a["r"] = e; endtask
  task automatic mk_out_i(output int a [string]); a["r"] = 6; endtask
  initial begin
    aa_t r; el_t e; int n, s; int ri [string];
    e.x = 3; r["p"] = e;
    mk_out(r); n = 0; s = 0; foreach (r[k]) begin n++; s += r[k].x; end
    $display("T|out1 n=%0d s=%0d", n, s);
    e.x = 3; r["p"] = e;
    $display("T|cnt=%0d", cnt_in(r));
    mk_out(r); n = 0; s = 0; foreach (r[k]) begin n++; s += r[k].x; end
    $display("T|out2 n=%0d s=%0d", n, s);
    ri["p"] = 1; mk_out_i(ri); $display("T|outi n=%0d", ri.num());
  end
endmodule
"#;
    assert_eq!(
        t_lines(SRC),
        ["T|cnt=9", "T|out1 n=1 s=6", "T|out2 n=1 s=6", "T|outi n=1",]
    );
}
