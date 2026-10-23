//! §13.5: argument binding — an argument past the last formal, or a formal
//! left without an actual and without a default, is an error; defaults and
//! named arguments still bind. Each rejected case is also rejected by the
//! reference simulator.

use xezim::simulate;

fn rejected(src: &str) -> String {
    match simulate(src, 100) {
        Ok(_) => panic!("accepted an illegal design:\n{src}"),
        Err(e) => e,
    }
}

#[test]
fn arguments_bind_to_formals() {
    let e = rejected(
        "module top; function int f(int a); return a; endfunction\n\
         initial $display(f(1, 2)); endmodule",
    );
    assert!(e.contains("too many arguments"), "{e}");
    rejected(
        "module top; function int f(int a = 1, int b = 2); return a; endfunction\n\
         initial $display(f( , , )); endmodule",
    );
    let e = rejected(
        "module top; task t(input int a, output int b); b = a; endtask\n\
         initial t(1); endmodule",
    );
    assert!(e.contains("no actual argument"), "{e}");
}

#[test]
fn defaults_and_named_arguments_bind() {
    let src = r#"
module top;
  function automatic int sum(int a, int b = 2, int c = 3);
    return a + b + c;
  endfunction
  task automatic t(input int a, output int b, input int c = 4);
    b = a + c;
  endtask
  int i, o;
  initial begin
    i = sum(1) + sum(1, 1) + sum(.a(1), .c(1)) + sum(1, , 0);
    t(1, o);
    $display("A|%0d %0d", i, o);
  end
endmodule
"#;
    let sim = simulate(src, 10).expect("legal calls must run");
    assert!(
        sim.output.iter().any(|o| o.message == "A|18 5"),
        "{:?}",
        sim.output
    );
}
