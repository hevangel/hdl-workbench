//! §18.5.9: inline constraints that tie two rand sub-objects together
//! (`a.x < b.y`) or a sub-object to an enclosing member (`a.x == this.n`)
//! are solved jointly with the sub-objects' own constraints. Each
//! sub-object was drawn on its own and the tie left to retries, which ran
//! out now and then (a few hundred calls failed a handful of times). A
//! sub-object's `post_randomize` sees the final values. The expected line
//! was cross-checked against the reference simulator.

use xezim::simulate;

const SRC: &str = r#"
class sub;
  rand bit [7:0] x;
  rand bit [7:0] y;
  bit [7:0] seen_x;
  constraint c { x < 200; }
  function void post_randomize(); seen_x = x; endfunction
endclass
class top_c;
  rand sub a;
  rand sub b;
  rand bit [7:0] n;
  function new(); a = new; b = new; endfunction
endclass
module top;
  top_c t;
  int ok, fails, stale;
  initial begin
    t = new;
    for (int i = 0; i < 300; i++) begin
      ok = t.randomize() with { a.x < b.y; b.y < 3; };
      if (!ok || !(t.a.x < t.b.y) || t.b.y >= 3) fails++;
      if (t.a.seen_x != t.a.x || t.b.seen_x != t.b.x) stale++;
    end
    for (int i = 0; i < 300; i++) begin
      ok = t.randomize() with { a.x == this.n; n == 199; };
      if (!ok || t.a.x != 199 || t.n != 199) fails++;
      if (t.a.seen_x != t.a.x) stale++;
    end
    for (int i = 0; i < 300; i++) begin
      ok = t.randomize() with { a.x + b.x == 255; a.x == b.y; b.y == 77; };
      if (!ok || t.a.x != 77 || t.b.x != 178 || t.b.y != 77) fails++;
    end
    ok = t.randomize() with { a.x > 250; };
    $display("fails=%0d stale=%0d unsat_ok=%0d", fails, stale, ok);
  end
endmodule
"#;

#[test]
fn cross_object_inline_constraints_solve_jointly() {
    let out: Vec<String> = simulate(SRC, 100)
        .expect("simulate failed")
        .output
        .iter()
        .map(|o| o.message.clone())
        .collect();
    assert_eq!(out, ["fails=0 stale=0 unsat_ok=0"], "{out:?}");
}
