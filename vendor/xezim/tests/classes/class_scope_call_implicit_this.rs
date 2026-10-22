//! IEEE 1800-2023 §8.23: inside a non-static method of a class or of a class
//! derived from it, `Class::m()` names `Class`'s `m` on the implicit `this`,
//! bound non-virtually. Only outside such a scope is a non-static method
//! through `::` illegal (§8.10, `nonstatic_method_class_scope_fatal.rs`).
//! Expected values are the reference simulator's.

use xezim::simulate;

const SRC: &str = r#"
class Base;
  int x = 5;
  function int m(); return x + 1; endfunction
  virtual function int vm(); return 100 + x; endfunction
  function void setx(int v); x = v; endfunction
  function int inst(); return Base::m() + 100; endfunction
endclass
typedef Base base_t;
class Mid extends Base;
  function int m(); return 70; endfunction
endclass
class Leaf extends Mid;
  function int m(); return 90; endfunction
  virtual function int vm(); return 200; endfunction
  function int own2(); return Leaf::m() + 1; endfunction
  function void run();
    $display("T|base_m=%0d base_vm=%0d alias=%0d mid=%0d own=%0d own2=%0d",
             Base::m(), Base::vm(), base_t::m(), Mid::m(), m(), own2());
    Base::setx(9);
    $display("T|x=%0d base_m=%0d inst=%0d", x, Base::m(), Base::inst());
  endfunction
endclass
module top;
  initial begin
    Leaf l = new;
    l.run();
    $display("T|done");
  end
endmodule
"#;

#[test]
fn class_scope_call_on_implicit_this() {
    let _ = super::compiled_method_test_env::eager();
    let sim = simulate(SRC, 10).expect("simulate failed");
    let o: Vec<String> = sim
        .output
        .iter()
        .map(|o| o.message.trim().to_string())
        .collect();
    for want in [
        "T|base_m=6 base_vm=105 alias=6 mid=70 own=90 own2=91",
        "T|x=9 base_m=10 inst=110",
        "T|done",
    ] {
        assert!(o.iter().any(|l| l == want), "missing {want:?} in {o:?}");
    }
    assert!(
        !o.iter().any(|l| l.contains("Illegal")),
        "unexpected fatal: {o:?}"
    );
}
