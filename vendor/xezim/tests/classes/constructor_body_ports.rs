//! §8.7: a class constructor takes its arguments in a port list only; its
//! body cannot declare non-ANSI ports. Other methods may. The reference
//! simulator agrees on both.

use xezim::simulate;

#[test]
fn constructor_with_body_ports() {
    let src = r#"
module test;
  class C;
    function new;
      input x;
    endfunction
  endclass
  C c = new;
endmodule
"#;
    assert!(simulate(src, 10).is_err());
}

#[test]
fn method_with_body_ports() {
    let src = r#"
module test;
  class C;
    int v;
    function new(input int x = 1);
      v = x;
    endfunction
  endclass
  class D;
    function void f;
      input int x;
      $display("M|%0d", x);
    endfunction
  endclass
  C c = new;
  D d = new;
  initial begin
    d.f(3);
    $display("M|%0d", c.v);
  end
endmodule
"#;
    let sim = simulate(src, 10).expect("body ports on a method are legal");
    let lines: Vec<&str> = sim
        .output
        .iter()
        .filter_map(|o| o.message.strip_prefix("M|"))
        .collect();
    assert_eq!(lines, ["3", "1"], "{:?}", sim.output);
}
