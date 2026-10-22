//! §8.25.1: outside its declaration, a parameterized class is named with a
//! parameter value list before `::`. The reference simulator rejects the bare
//! `par_cls::b` and runs the specialized forms with the same output.

use xezim::simulate;

#[test]
fn bare_parameterized_class_scope() {
    let src = r#"
module class_tb ();
  class par_cls #(int a = 25);
    parameter int b = 23;
  endclass
  par_cls #(15) inst;
  initial begin
    inst = new;
    $display(par_cls::b);
  end
endmodule
"#;
    assert!(simulate(src, 10).is_err());
}

#[test]
fn specialized_class_scope() {
    let src = r#"
module class_tb ();
  class par_cls #(int a = 25);
    parameter int b = 23;
    static function int twice(); return 2 * a; endfunction
  endclass
  class plain;
    static int k = 4;
  endclass
  typedef par_cls #(3) p3_t;
  initial $display("K|%0d %0d %0d %0d", par_cls#()::b, par_cls#(7)::twice(), p3_t::twice(), plain::k);
endmodule
"#;
    let sim = simulate(src, 10).expect("specialized scopes are legal");
    assert!(
        sim.output.iter().any(|o| o.message == "K|23 14 6 4"),
        "{:?}",
        sim.output
    );
}
