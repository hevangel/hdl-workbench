//! §18.5.2: it shall be an error to declare a `pure constraint` in a
//! non-abstract (concrete) class. A `pure constraint` may only appear in a
//! `virtual` class; it is a meaningless obligation on a concrete class,
//! which can never be subclassed to fulfill it.

use xezim::simulate;

#[test]
fn pure_constraint_in_concrete_class_is_illegal() {
    let src = r#"
class a2;
  rand int x;
  pure constraint c { x inside {[1:5]}; }
endclass
module top;
  initial begin
    a2 o = new;
    void'(o.randomize());
  end
endmodule
"#;
    let err = match simulate(src, 10) {
        Err(e) => e,
        Ok(_) => panic!("a non-abstract class may not declare a pure constraint"),
    };
    assert!(err.contains("§18.5.2"), "unexpected diagnostic: {}", err);
    assert!(
        err.contains("declared in a virtual (abstract) class"),
        "unexpected diagnostic: {}",
        err
    );
}

#[test]
fn pure_constraint_in_virtual_class_is_legal() {
    let src = r#"
virtual class a;
  rand int x;
  pure constraint c { x inside {[1:5]}; }
endclass
class a2 extends a;
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
    let sim = simulate(src, 10).expect("a virtual class may declare a pure constraint");
    assert!(
        sim.output.iter().any(|o| o.message == "R|3"),
        "{:?}",
        sim.output
    );
}

#[test]
fn inherited_pure_constraint_still_must_be_implemented() {
    // The original §18.5.2 rule is unchanged: a concrete subclass that
    // inherits an unimplemented pure constraint still errors.
    let src = r#"
virtual class a;
  rand int x;
  pure constraint c;
endclass
class a2 extends a;
endclass
module top; endmodule
"#;
    let err = match simulate(src, 10) {
        Err(e) => e,
        Ok(_) => panic!("a concrete subclass must implement inherited pure constraints"),
    };
    assert!(
        err.contains("does not implement the inherited pure constraint 'c'"),
        "unexpected diagnostic: {}",
        err
    );
}
