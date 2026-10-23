//! §8.10 / §13.3: in a method of a class with an `int s` property, a bare
//! `s = v` writes that property even when another class has an unpacked
//! struct property `s` and the module has a struct variable `s`. The write
//! used to take the struct-copy path and land in the module struct. Runs at
//! tier 0 (a child process compiles every method on its first call) and at
//! the default tier, where the first 1000 calls run on the interpreter.
//! Expected values come from the reference simulator.

use xezim::simulate;

#[test]
fn int_property_write_ignores_other_struct_named_alike() {
    let _ = super::compiled_method_test_env::eager();
    let src = r#"
module top;
  typedef struct { int a; int b; } st_t;
  st_t s;
  class A;
    int s;
    function void set(int v); s = v; endfunction
  endclass
  class SP;
    st_t s;
    function void cp(int v); st_t t; t.a = v; t.b = -v; s = t; endfunction
  endclass
  initial begin
    automatic A a = new;
    automatic SP sp = new;
    s.a = 7; s.b = 8;
    for (int i = 0; i < 1500; i++) begin
      a.set(i);
      sp.cp(i);
      if (i == 2) $display("T|i=2 a.s=%0d sp.s=%0d/%0d mod.s=%0d/%0d", a.s, sp.s.a, sp.s.b, s.a, s.b);
    end
    $display("T|a.s=%0d sp.s=%0d/%0d mod.s=%0d/%0d", a.s, sp.s.a, sp.s.b, s.a, s.b);
  end
endmodule
"#;
    let sim = simulate(src, 10).expect("simulate failed");
    let out: Vec<&str> = sim
        .output
        .iter()
        .map(|o| o.message.trim())
        .filter(|l| l.starts_with("T|"))
        .collect();
    assert_eq!(
        out,
        [
            "T|i=2 a.s=2 sp.s=2/-2 mod.s=7/8",
            "T|a.s=1499 sp.s=1499/-1499 mod.s=7/8",
        ]
    );
}
