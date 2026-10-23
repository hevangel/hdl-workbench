//! §18.5.2: a non-virtual class must implement every pure constraint it
//! inherits. The reference simulator rejects the unimplemented case too, but
//! it accepts no `pure constraint` at all, so the legal case follows the LRM.

use xezim::simulate;

#[test]
fn pure_constraint_not_implemented() {
    let src = r#"
virtual class a;
  pure constraint c;
endclass
class a2 extends a;
endclass
module top;
endmodule
"#;
    assert!(simulate(src, 10).is_err());
}

#[test]
fn pure_constraint_implemented() {
    let src = r#"
virtual class a;
  rand int x;
  pure constraint c;
endclass
virtual class a1 extends a;
endclass
class a2 extends a1;
  constraint c { x inside {[3:3]}; }
endclass
module top;
  initial begin
    a2 o = new;
    void'(o.randomize());
    $display("R|%0d", o.x);
  end
endmodule
"#;
    let sim = simulate(src, 10).expect("an implemented pure constraint is legal");
    assert!(
        sim.output.iter().any(|o| o.message == "R|3"),
        "{:?}",
        sim.output
    );
}
