//! §8.15: `this.super.x` names the base-class member that `super.x` names,
//! behind a same-named member of the derived class. Expected line is the
//! reference simulator's.

use xezim::simulate;

#[test]
fn this_super_reaches_the_base_member() {
    let src = r#"
module top;
  class B;
    int x;
  endclass
  class C extends B;
    byte x;
    task set(int v);
      this.super.x = v;
      this.x = 7;
    endtask
    function int get();
      return this.super.x + x;
    endfunction
  endclass
  C c;
  initial begin
    c = new;
    c.set(1000);
    $display("U|%0d", c.get());
  end
endmodule
"#;
    let sim = simulate(src, 10).expect("simulate");
    assert!(
        sim.output.iter().any(|o| o.message == "U|1007"),
        "{:?}",
        sim.output
    );
}
