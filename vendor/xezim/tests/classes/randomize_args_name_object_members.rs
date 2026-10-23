//! §18.11: the arguments of `obj.randomize(v, w)` name members of `obj`, not
//! of the calling scope, so they are not undeclared identifiers there. The
//! reference simulator also accepts this design (and additionally randomizes
//! `v` and `w`, which is not asserted here).

use xezim::simulate;

#[test]
fn randomize_arguments_are_members_of_the_object() {
    let src = r#"
module top;
  class a;
    rand int x = 0, y = 0;
    int v = 0, w = 0;
    constraint c { x < v && y > w; }
  endclass
  class env;
    a obj = new;
    task run();
      int ok;
      ok = obj.randomize(v, w);
      $display("R|%0d %0d", obj.x, obj.y);
    endtask
  endclass
  env e;
  initial begin
    e = new;
    e.run();
  end
endmodule
"#;
    let sim = simulate(src, 10).expect("randomize(v, w) arguments are obj's members");
    assert!(
        sim.output.iter().any(|o| o.message == "R|0 0"),
        "{:?}",
        sim.output
    );
}
