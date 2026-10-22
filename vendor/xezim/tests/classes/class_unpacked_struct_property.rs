//! Unpacked-struct class properties: whole-struct assign / compare / copy /
//! clone on a parameterized class.
//!
//! Exercises:
//!   * external whole-struct write `obj.first = var` (decomposed member-wise),
//!   * whole-struct compare class-prop vs class-prop and class-prop vs var,
//!   * member-wise copy through a method.
//!
//! Note: struct arguments passed through a *constructor* (`new(s1,s2)` with
//! `first = f` inside) depend on whole-struct READ of a top-level/local
//! variable, which is a separate known gap and is intentionally not exercised
//! here.

use xezim::simulate;

#[test]
fn test_unpacked_struct_class_properties() {
    const SRC: &str = r#"
typedef struct {
  int a;
  int b;
} s_t;

class pair #(type T1 = int, type T2 = int);
  T1 first;
  T2 second;
  function new(string name);
    // no struct args: members are assigned externally (UVM pattern)
  endfunction
  function bit compare(pair #(T1, T2) rhs);
    return (first == rhs.first) && (second == rhs.second);
  endfunction
  function void copy(pair #(T1, T2) rhs);
    first = rhs.first;
    second = rhs.second;
  endfunction
endclass

module tb;
  int pass_count;
  initial begin
    s_t s0, s1, s2;
    pair #(s_t, s_t) a, c;
    pass_count = 0;

    s0.a =  0;   s0.b =  0;
    s1.a = -100; s1.b = -200;
    s2.a =  300; s2.b =  400;

    a = new("a"); a.first = s0; a.second = s0;
    c = new("c"); c.first = s1; c.second = s2;

    // whole-struct compare: class-prop vs top-level struct variable
    if (a.first == s0 && c.first == s1) pass_count++;
    // whole-struct compare via method: differing -> compare() must be 0
    if (!c.compare(a)) pass_count++;
    // member-wise copy makes them equal -> compare() must be 1
    c.copy(a);
    if (c.compare(a)) pass_count++;
  end
endmodule
"#;
    let sim = simulate(SRC, 100).expect("simulate failed");
    let pc: u64 = sim
        .get_signal("pass_count")
        .unwrap_or_else(|| panic!("signal not found: pass_count"))
        .to_u64()
        .unwrap_or_else(|| panic!("pass_count not u64-able"));
    assert_eq!(
        pc, 3,
        "unpacked struct class property assign/compare/copy failed"
    );
}
