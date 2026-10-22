//! IEEE 1800-2023 §13.4.2 / §7.2 / §12.7.3: subroutines copying an
//! UNPACKED-STRUCT formal, binding a struct-formal from a positional pattern
//! literal, resolving a struct type-PARAMETER formal on a parameterized class,
//! and `foreach` over a queue/associative array of STRINGS.
//!
//! Four independent simulator fixes:
//!   1. A whole-struct assignment whose RHS is a METHOD/FUNCTION formal
//!      (`y = arg`) never registered the formal's struct type, so
//!      `p_elem_type(arg)` missed it and the RHS evaluated to a zeroed
//!      container (members x/0) even though `arg.member` read correctly.
//!   2. Binding a struct formal from a positional pattern literal
//!      (`f('{a, b})`) read the members via `arg.member` — member access on a
//!      literal base returns x in this simulator — so it bound zeros.
//!   3. A struct formal of a parameterized class (`class C #(type T); f(T)`)
//!      registered only the raw `TypeReference(T)`, which `p_elem_type` could
//!      not resolve to the concrete struct — again zeroed.
//!   4. `foreach (key_queue[i])` over a QUEUE/ASSOC ARRAY OF STRINGS was
//!      mistaken for `foreach` over a scalar string (it iterated characters),
//!      so the loop body never ran on the string queue (zero iterations).
use std::process::Command;

fn xezim() -> String {
    env!("CARGO_BIN_EXE_xezim").to_string()
}

fn run(src: &str, tag: &str) -> String {
    let path = format!("/tmp/struct_formal_{tag}.sv");
    std::fs::write(&path, src).unwrap();
    let out = Command::new(xezim())
        .args(["--simulate", "-s", "top", &path])
        .output()
        .expect("run xezim");
    String::from_utf8_lossy(&out.stdout).into_owned()
}

fn passes(out: &str, tag: &str) -> bool {
    out.lines()
        .any(|l| l.contains(tag) && l.contains("TAG_PASS"))
}

#[test]
fn struct_function_formal_copy() {
    // A whole-struct copy whose RHS is a function's struct formal must carry
    // the caller's members (`item.a`=3, `item.b`=4), not zero them.
    let src = r#"module top;
  typedef struct { int a; int b; } s2;
  function void f(s2 item);
    s2 l;
    l.a = 99; l.b = 98;
    l = item;
    if (l.a == 3 && l.b == 4) $display("TAG_PASS fn-formal-copy");
    else $display("TAG_FAIL fn-formal-copy a=%0d b=%0d", l.a, l.b);
  endfunction
  initial begin
    s2 v = '{3, 4};
    f(v);
  end
endmodule"#;
    let out = run(src, "fncopy");
    assert!(
        passes(&out, "fn-formal-copy"),
        "function struct-formal whole copy failed:\n{out}"
    );
}

#[test]
fn class_method_struct_literal_formal() {
    // A class method formal bound from a positional struct-pattern literal
    // stores both members (not zeros).
    let src = r#"module top;
  typedef struct { int a; int b; } s2;
  class Q;
    s2 q[$];
    function void pushv(s2 item); q.push_back(item); endfunction
  endclass
  initial begin
    Q qo = new;
    qo.pushv('{6, 7});
    if (qo.q[0].a == 6 && qo.q[0].b == 7) $display("TAG_PASS method-literal-formal");
    else $display("TAG_FAIL method-literal-formal a=%0d b=%0d", qo.q[0].a, qo.q[0].b);
  end
endmodule"#;
    let out = run(src, "methlit");
    assert!(
        passes(&out, "method-literal-formal"),
        "class-method struct literal formal failed:\n{out}"
    );
}

#[test]
fn param_class_struct_typearg_formal() {
    // Parameterized-class `T` bound to a struct: a whole-struct read of the T
    // formal resolves the concrete struct type through the instance bindings.
    let src = r#"module top;
  typedef struct { int a; int b; } s2;
  class P #(type T = int);
    T m;
    function void setf(T v); m = v; endfunction
  endclass
  initial begin
    P#(s2) p = new;
    p.setf('{8, 9});
    if (p.m.a == 8 && p.m.b == 9) $display("TAG_PASS param-struct-formal");
    else $display("TAG_FAIL param-struct-formal a=%0d b=%0d", p.m.a, p.m.b);
  end
endmodule"#;
    let out = run(src, "paramstruct");
    assert!(
        passes(&out, "param-struct-formal"),
        "parameterized-class struct formal failed:\n{out}"
    );
}

#[test]
fn foreach_string_queue_and_assoc() {
    // `foreach` over a QUEUE OF STRINGS and an ASSOC ARRAY OF STRINGS must
    // iterate ELEMENTS, not be mistaken for a scalar-string character loop.
    let src = r#"module top;
  initial begin
    string q[$];
    int n = 0;
    string m[int];
    int n2 = 0;
    q.push_back("aa"); q.push_back("bb");
    foreach (q[i]) n++;
    m[5] = "five";
    foreach (m[k]) n2++;
    if (n == 2 && n2 == 1) $display("TAG_PASS foreach-string-cols");
    else $display("TAG_FAIL foreach-string-cols n=%0d n2=%0d", n, n2);
  end
endmodule"#;
    let out = run(src, "fe_strcol");
    assert!(
        passes(&out, "foreach-string-cols"),
        "foreach over string collections failed:\n{out}"
    );
}
