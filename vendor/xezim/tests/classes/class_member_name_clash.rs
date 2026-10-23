//! §8.3: a class's properties, typedefs and enum constants share one name
//! space. The reference simulator rejects a typedef named like an enum
//! constant and runs the legal class with the same output.

use xezim::simulate;

#[test]
fn enum_constant_and_typedef_clash() {
    let src = r#"
module test;
  class C;
    enum { A = 10 } e;
    typedef int A;
  endclass
endmodule
"#;
    assert!(simulate(src, 10).is_err());
}

#[test]
fn distinct_class_member_names() {
    let src = r#"
module test;
  class C;
    enum { A = 10, B } e = B;
    typedef int T;
    T t = 3;
    typedef enum { X = 1, Y } xy_t;
    xy_t xy = Y;
  endclass
  initial begin
    C c = new;
    $display("C|%0d %0d", c.t, c.xy);
  end
endmodule
"#;
    let sim = simulate(src, 10).expect("distinct member names are legal");
    assert!(
        sim.output.iter().any(|o| o.message == "C|3 2"),
        "{:?}",
        sim.output
    );
}
