//! §26.3: `P::name` names an item declared in package `P`, and must be a
//! function where it is called; a task, or a void function, is not called in
//! an expression (§13.3, §13.4.1). Each rejected case is also rejected by the
//! reference simulator, and the legal module prints the same line there.

use xezim::simulate;

fn rejected(src: &str) -> String {
    match simulate(src, 100) {
        Ok(_) => panic!("accepted an illegal design:\n{src}"),
        Err(e) => e,
    }
}

#[test]
fn package_scoped_names_must_exist() {
    let e = rejected("package P; endpackage\nmodule top; int y; initial y = P::f(10); endmodule");
    assert!(e.contains("not declared in package 'P'"), "{e}");
    rejected("int x;\npackage P; endpackage\nmodule top; int y; initial y = P::x; endmodule");
    rejected("package P; int x; endpackage\nmodule top; int y; initial y = P::x(10); endmodule");
}

#[test]
fn tasks_and_void_functions_have_no_value() {
    let e = rejected("module top; task t; endtask initial begin int x; x = t() + 1; end endmodule");
    assert!(e.contains("§13.3"), "{e}");
    rejected(
        "package P; task t(int x); endtask endpackage\n\
         module top; int y; initial y = P::t(10); endmodule",
    );
    let e = rejected(
        "module top; function void f; endfunction initial begin int x; x = f() + 1; end endmodule",
    );
    assert!(e.contains("§13.4.1"), "{e}");
}

#[test]
fn package_items_in_legal_uses() {
    let src = r#"
package P;
  parameter int W = 4;
  typedef enum {A, B, C} e_t;
  int v;
  function int f(int a); return a + W; endfunction
  function void g(int a); v = a; endfunction
  task t(output int o); o = 9; endtask
  class K; int k = 5; endclass
endpackage
module top;
  P::e_t e;
  P::K k;
  int o;
  initial begin
    e = P::C;
    P::g(2);
    void'(P::f(1));
    P::t(o);
    P::v = P::v + 1;
    k = new;
    $display("L3|%0d %0d %0d %0d %0d %0d", e, P::f(1), P::v, o, k.k, P::W);
  end
endmodule
"#;
    let sim = simulate(src, 10).expect("legal package uses must run");
    assert!(
        sim.output.iter().any(|o| o.message == "L3|2 5 3 9 5 4"),
        "{:?}",
        sim.output
    );
}
