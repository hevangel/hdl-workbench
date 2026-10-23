//! §18.5.14 / §18.6.1: with a soft constraint present, a trial is left to
//! the joint solver, which honours every soft item it can. When an array is
//! sized by the constraints, the solver's give-up did not end that hand-off,
//! so the trial loop kept rejecting assignments that satisfied every
//! constraint and `randomize()` returned 0 (#256, second cause). Here the
//! solver gives up on an `int` element equal to a narrower unsigned value; the
//! reference simulator returns 1 for each class, with the soft item held.

use xezim::simulate;

const SRC: &str = r#"
typedef struct packed { bit [7:0] a; bit [7:0] b; } ps_t;

// The shape from the #256 discussion: a packed-struct field read by a
// foreach constraint over a dynamic array sized by the constraints.
class field_elem;
  rand ps_t p;
  rand int d[];
  rand bit flag;
  constraint c { p.a == 9; d.size() == 2; foreach (d[i]) d[i] == p.a; soft flag == 1; }
endclass

// The same with a plain unsigned scalar.
class scalar_elem;
  rand bit [7:0] a;
  rand int d[];
  rand bit flag;
  constraint c { a == 9; d.size() == 3; foreach (d[i]) d[i] == a; soft flag == 1; }
endclass

// The soft item conflicts with nothing else, so it must hold.
class soft_value;
  rand bit [7:0] a;
  rand int d[];
  rand bit [3:0] mode;
  constraint c { d.size() == 2; foreach (d[i]) d[i] == a; soft mode == 5; }
endclass

module top;
  initial begin
    field_elem x = new();
    scalar_elem y = new();
    soft_value z = new();
    int ok;
    repeat (2) begin
      ok = x.randomize();
      $display("field_elem ok=%0d a=%0d size=%0d d=%0d,%0d flag=%0d", ok, x.p.a, x.d.size(),
               x.d[0], x.d[1], x.flag);
      ok = y.randomize();
      $display("scalar_elem ok=%0d size=%0d d=%0d,%0d,%0d flag=%0d", ok, y.d.size(),
               y.d[0], y.d[1], y.d[2], y.flag);
      ok = z.randomize();
      $display("soft_value ok=%0d size=%0d same=%0d mode=%0d", ok, z.d.size(),
               z.d[0] == z.a && z.d[1] == z.a, z.mode);
    end
  end
endmodule
"#;

#[test]
fn soft_constraint_survives_solver_give_up() {
    let out: Vec<String> = simulate(SRC, 100)
        .expect("simulate failed")
        .output
        .iter()
        .map(|o| o.message.clone())
        .collect();
    let round = [
        "field_elem ok=1 a=9 size=2 d=9,9 flag=1",
        "scalar_elem ok=1 size=3 d=9,9,9 flag=1",
        "soft_value ok=1 size=2 same=1 mode=5",
    ];
    let expected: Vec<&str> = round.iter().chain(round.iter()).copied().collect();
    assert_eq!(out, expected, "{out:?}");
}
