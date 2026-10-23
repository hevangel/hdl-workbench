//! Unpacked-struct values passed through a TYPE-PARAMETER formal of a
//! parameterized class — the case `class C #(type T); T prop; function
//! new(T f); prop = f; ... endfunction` (and a method taking a `T` formal).
//!
//! Before this fix, `bind_unpacked_struct_arg` looked up the formal's declared
//! type name (`T1`) directly in the typedef table, missed (it is a type
//! parameter, not a typedef), and fell back to a single whole-value binding —
//! so every struct member read back `x`. The fix resolves the type parameter
//! through the callee instance's `type_bindings` (or the active specialization)
//! to the concrete bound type, then binds the actual member-wise.
//!
//! Covers (all INPUT formals — `output`/`inout` struct formals are a separate,
//! broader gap and are not exercised here):
//!   * constructor `new(T f)` with `prop = f` (struct passed by value),
//!   * whole-struct compare class-prop vs top-level variable after construction,
//!   * member-wise copy through a class-handle formal,
//!   * a regular method taking a `T` struct formal (read its members).

use xezim::simulate;

#[test]
fn type_param_struct_formal() {
    const SRC: &str = r#"
typedef struct {
  int a;
  int b;
} s_t;

class pair #(type T1 = int, type T2 = int);
  T1 first;
  T2 second;
  function new(T1 f, T2 s);   // TYPE-PARAMETER struct formals
    first  = f;
    second = s;
  endfunction
  function bit compare(pair #(T1, T2) rhs);
    return (first == rhs.first) && (second == rhs.second);
  endfunction
  function void copy(pair #(T1, T2) rhs);
    first  = rhs.first;
    second = rhs.second;
  endfunction
  // regular method with a T1 struct INPUT formal (reads its members)
  function int sum_field(T1 f);
    return f.a + f.b;
  endfunction
endclass

module top;
  int pass_count;
  initial begin
    s_t s1, s2;
    pair #(s_t, s_t) p1, p2;
    pass_count = 0;

    s1.a = 10; s1.b = 20;
    s2.a = 30; s2.b = 40;

    // constructor: struct actuals bound to type-parameter formals
    p1 = new(s1, s2);
    p2 = new(s1, s2);
    if (p1.first.a == 10 && p1.first.b == 20
        && p1.second.a == 30 && p1.second.b == 40) pass_count++;

    // whole-struct compare class-prop vs top-level struct variable
    if (p1.first == s1 && p1.second == s2) pass_count++;

    // method with type-parameter struct formal (read members from the formal)
    if (p1.sum_field(s1) == 30) pass_count++;

    // member-wise copy through a class-handle formal, then compare
    s2.a = 99;
    p2.copy(p1);
    if (p2.compare(p1)) pass_count++;
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
        pc, 4,
        "type-parameter struct formal (constructor + method) failed"
    );
}
