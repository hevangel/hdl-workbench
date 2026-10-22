//! §8.26.6.2 / §8.26.6.3: an interface class that inherits one parameter
//! name from two different base classes, or from two specializations of one
//! class, must declare that name itself. The reference simulator rejects both
//! and accepts the diamond through one specialization.

use xezim::simulate;

#[test]
fn conflicting_inherited_type_parameter() {
    let src = r#"
module class_tb ();
  interface class ic1#(type T = logic);
    pure virtual function void fn1(T a);
  endclass
  interface class ic2#(type T = logic);
    pure virtual function void fn2(T a);
  endclass
  interface class ic3#(type TYPE = logic) extends ic1#(TYPE), ic2#(TYPE);
  endclass
endmodule
"#;
    assert!(simulate(src, 10).is_err());
    let src = r#"
module class_tb ();
  interface class ibase#(type T = logic);
    pure virtual function void fn(T val);
  endclass
  interface class ic1 extends ibase#(bit);
    pure virtual function void fn1();
  endclass
  interface class ic2 extends ibase#(string);
    pure virtual function void fn2();
  endclass
  interface class ic3 extends ic1, ic2;
    pure virtual function void fn3();
  endclass
endmodule
"#;
    assert!(simulate(src, 10).is_err());
}

#[test]
fn resolved_inherited_type_parameter() {
    let src = r#"
module class_tb ();
  interface class ibase#(type T = logic);
    pure virtual function void fn(T val);
  endclass
  interface class ic1 extends ibase#(bit);
    pure virtual function void fn1();
  endclass
  interface class ic2 extends ibase#(bit);
    pure virtual function void fn2();
  endclass
  interface class ic3 extends ic1, ic2;
    pure virtual function void fn3();
  endclass
  interface class ic4#(type T = int) extends ic1, ibase#(string);
  endclass
  class impl implements ic3;
    virtual function void fn(bit val); $display("I|%b", val); endfunction
    virtual function void fn1(); endfunction
    virtual function void fn2(); endfunction
    virtual function void fn3(); endfunction
  endclass
  initial begin
    impl o = new;
    o.fn(1'b1);
  end
endmodule
"#;
    let sim = simulate(src, 10).expect("one specialization is legal");
    assert!(
        sim.output.iter().any(|o| o.message == "I|1"),
        "{:?}",
        sim.output
    );
}

// §8.26.6.3: a fully-parameterized diamond that reuses the same type name T
// through two different intermediate classes is legal when the leaf class
// declares T itself (issue #476).
#[test]
fn parameterized_diamond_redeclares_type() {
    let src = r#"
module class_tb ();
  interface class ibase#(type T);
    pure virtual function T get();
  endclass
  interface class ic1#(type T) extends ibase#(T);
  endclass
  interface class ic2#(type T) extends ibase#(T);
  endclass
  // Diamond: both ic1 and ic2 (and ibase) contribute T; ic3 declares it.
  interface class ic3#(type T) extends ic1#(T), ic2#(T);
  endclass
  class impl#(type T) implements ic3#(T);
    virtual function T get(); return 42; endfunction
  endclass
  initial begin
    impl#(int) o = new;
    $display("T|476|%0d", o.get());
  end
endmodule
"#;
    let sim = simulate(src, 10).expect("parameterized diamond with redeclaration is legal");
    assert!(
        sim.output.iter().any(|o| o.message == "T|476|42"),
        "{:?}",
        sim.output
    );
}

// §8.26.6.2: distinct parameter names from different bases (T vs U) do not
// collide when the leaf specializes both bases (issue #477).
#[test]
fn distinct_parameter_names_across_bases() {
    let src = r#"
module class_tb ();
  interface class ic1#(type T);
    pure virtual function T get();
  endclass
  interface class ic2#(int U);
    pure virtual function int get();
  endclass
  interface class ic3 extends ic1#(int), ic2#(42);
  endclass
  class impl implements ic3;
    virtual function int get(); return 42; endfunction
  endclass
  initial begin
    impl o = new;
    $display("T|477|%0d", o.get());
  end
endmodule
"#;
    let sim = simulate(src, 10).expect("distinct parameter names across bases are legal");
    assert!(
        sim.output.iter().any(|o| o.message == "T|477|42"),
        "{:?}",
        sim.output
    );
}
