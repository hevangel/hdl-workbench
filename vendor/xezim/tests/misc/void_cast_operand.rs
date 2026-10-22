//! §13.4.1: `void'(...)` discards the result of a function call; an operator
//! expression or a literal is no call. The reference simulator rejects these
//! operands and runs the call forms with the same output.

use xezim::simulate;

#[test]
fn void_cast_of_non_call() {
    for stmt in ["void'(1+2);", "void'(4'd3);", "void'(-f());", "void'((1));"] {
        let src = format!(
            "module test;\n  function int f(); return 1; endfunction\n  initial begin {stmt} end\nendmodule\n"
        );
        assert!(simulate(&src, 10).is_err(), "accepted:\n{src}");
    }
}

#[test]
fn void_cast_of_calls() {
    let src = r#"
module test;
  int n;
  function int f(); n++; return 1; endfunction
  function int h(); return 3; endfunction
  class C;
    function int g(); n += 10; return 2; endfunction
  endclass
  C c;
  initial begin
    c = new;
    void'(f());
    void'(c.g());
    void'($urandom);
    void'(h);
    $display("V|%0d", n);
  end
endmodule
"#;
    let sim = simulate(src, 10).expect("void casts of calls are legal");
    assert!(
        sim.output.iter().any(|o| o.message == "V|11"),
        "{:?}",
        sim.output
    );
}
