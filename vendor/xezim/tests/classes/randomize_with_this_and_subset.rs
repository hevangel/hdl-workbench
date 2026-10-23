//! §18.7 / §18.11 — `randomize(…) with {…}` without a receiver inside a class
//! method (`this` is implied) and with a member list (`randomize(y) with`).
//! The bare form evaluated the plain call and dropped the inline block, so
//! `randomize() with { x == k; }` returned 1 with an unconstrained `x`; the
//! member list was ignored under `with`, so a non-listed member was solved
//! instead of being held as state (`randomize(y) with { x == 5; }` rewrote
//! `x`). Counts cross-checked against the reference simulator.

use xezim::simulate;

const SRC: &str = r#"
class item;
  rand bit [15:0] x;
  rand bit [15:0] y;
  bit [15:0] base = 16'h40;
  constraint c { x < 16'h1000; }
  function void f(bit [15:0] k);
    int ok = 0, bad = 0;
    for (int i = 0; i < 5; i++) begin
      if (!randomize() with { x == k + i; y == base; }) bad++;
      else if (x == k + i && y == base) ok++;
    end
    $display("F bare: ok=%0d fail=%0d", ok, bad);
  endfunction
  function void g(bit [15:0] k);
    int ok = 0, bad = 0;
    for (int i = 0; i < 5; i++) begin
      if (!this.randomize() with { x == k + i; }) bad++;
      else if (x == k + i) ok++;
    end
    $display("G this: ok=%0d fail=%0d", ok, bad);
  endfunction
  function void h(bit [15:0] k);
    int ok = 0, bad = 0;
    x = 16'h7;
    for (int i = 0; i < 5; i++) begin
      if (!randomize(y) with { y == k + i; x == 16'h7; }) bad++;
      else if (y == k + i && x == 16'h7) ok++;
    end
    $display("H bare subset: ok=%0d fail=%0d", ok, bad);
    ok = 0; bad = 0;
    for (int i = 0; i < 5; i++) begin
      if (!randomize(y) with { y == k + i; x == 16'h8; }) bad++;
      else ok++;
    end
    $display("H2 bare subset state conflict: ok=%0d fail=%0d x=%0h", ok, bad, x);
  endfunction
endclass
module top;
  initial begin
    item it = new;
    int ok, bad;
    it.f(16'h20);
    it.g(16'h30);
    it.h(16'h30);
    it.x = 16'h9;
    ok = 0; bad = 0;
    for (int i = 0; i < 5; i++) begin
      if (!it.randomize(y) with { y == 16'h100 + i; x == 16'h9; }) bad++;
      else if (it.y == 16'h100 + i && it.x == 16'h9) ok++;
    end
    $display("S subset: ok=%0d fail=%0d", ok, bad);
    ok = 0; bad = 0;
    for (int i = 0; i < 5; i++) begin
      if (!it.randomize(y) with { x == 16'h5; }) bad++;
      else ok++;
    end
    $display("S2 subset state conflict: ok=%0d fail=%0d x=%0h", ok, bad, it.x);
    ok = 0; bad = 0;
    for (int i = 0; i < 5; i++) begin
      if (!it.randomize(x, y) with { x == y + 1; y < 16'h10; }) bad++;
      else if (it.x == it.y + 1 && it.y < 16'h10) ok++;
    end
    $display("S3 two-name subset: ok=%0d fail=%0d", ok, bad);
  end
endmodule
"#;

#[test]
fn bare_and_subset_randomize_with() {
    let sim = simulate(SRC, 100).expect("simulate failed");
    let got: Vec<&str> = sim
        .output
        .iter()
        .map(|o| o.message.as_str())
        .filter(|m| m.contains(": ok="))
        .collect();
    assert_eq!(
        got,
        [
            "F bare: ok=5 fail=0",
            "G this: ok=5 fail=0",
            "H bare subset: ok=5 fail=0",
            "H2 bare subset state conflict: ok=0 fail=5 x=7",
            "S subset: ok=5 fail=0",
            "S2 subset state conflict: ok=0 fail=5 x=9",
            "S3 two-name subset: ok=5 fail=0",
        ]
    );
}
