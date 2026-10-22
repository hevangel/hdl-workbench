//! class-perf Step 8 — STRING formals and STRING returns on the compiled
//! class-method surface:
//!
//! * a `string` RETURN is a byte-vector register passed through untouched
//!   (width 0, no resize, no signedness) — `return s;` / `return name;`;
//! * a `string` FORMAL binds the actual UNRESIZED, exactly like the
//!   interpreter's formal-binding branch (no resize for Simple{String});
//! * string EQUALITY (`a == b`, `a == "lit"`) lowers to packed Eq — for
//!   pure-byte operands the packed compare's zero-extension matches the
//!   interpreter's string path (leading-NUL trim);
//! * a string formal passed THROUGH to a nested call (`echo(s)`), and an
//!   empty-string argument (`Value` width 0) round-trips via a string
//!   literal actual, not a `1'd0` integer;
//! * default argument values on string formals (`string s = "dflt"`).
//!
//! Byte-for-byte validated against the reference simulator via
//! /tmp/svrun/method_vm22.sv (TAG r1..r8 + TAG_DONE identical).
//!
//! The compiler BAILS (keeps the AST interpreter) on any non-pass-through
//! use of a string formal: indexing (`s[i]`), string methods (`s.len()`),
//! relational/arithmetic operands. Those stay correct via the interpreter.
//!
//! The gate is forced ON so a plain `cargo test` exercises the compiled
//! path.
use xezim::simulate;

fn gate_on() -> bool {
    super::compiled_method_test_env::eager()
}

fn u(sim: &xezim::compiler::Simulator, n: &str) -> u64 {
    sim.get_signal(n)
        .or_else(|| sim.get_signal(&format!("top.{}", n)))
        .unwrap_or_else(|| panic!("signal not found: {}", n))
        .to_u64()
        .unwrap_or_else(|| panic!("{} is x/z", n))
}

fn s(sim: &xezim::compiler::Simulator, n: &str) -> String {
    sim.get_signal(n)
        .or_else(|| sim.get_signal(&format!("top.{}", n)))
        .unwrap_or_else(|| panic!("signal not found: {}", n))
        .to_sv_string()
}

/// The method_vm22 surface: string returns, string-formal equality,
/// pass-through to nested calls, defaults.
#[test]
fn string_forms_end_to_end() {
    if !gate_on() {
        return;
    }
    let src = r#"
class C;
  string name;
  function new(string n = "");
    name = n;
  endfunction
  function string echo(string s);
    return s;
  endfunction
  function string echo_self();
    return name;
  endfunction
  function bit same(string a, string b);
    if (a == b) return 1;
    return 0;
  endfunction
  function string chain(string a, string b);
    if (a == b) return echo(a);
    return echo(b);
  endfunction
  function string deflt(string s = "dflt");
    return s;
  endfunction
endclass
module top;
  string r1, r2, r4, r5, r6, r7;
  int r3;
  initial begin
    C c = new("hi");
    r1 = c.echo("hello");
    r2 = c.echo_self();
    r3 = c.same("a", "a") + 2 * c.same("a", "b");
    r4 = c.chain("p", "q");
    r5 = c.chain("p", "p");
    r6 = c.deflt();
    r7 = c.deflt("z");
  end
endmodule
"#;
    let sim = simulate(src, 100).expect("simulation should run");
    assert_eq!(s(&sim, "r1"), "hello");
    assert_eq!(s(&sim, "r2"), "hi");
    assert_eq!(u(&sim, "r3"), 1);
    assert_eq!(s(&sim, "r4"), "q");
    assert_eq!(s(&sim, "r5"), "p");
    assert_eq!(s(&sim, "r6"), "dflt");
    assert_eq!(s(&sim, "r7"), "z");
}

/// The empty-string actual must round-trip as a width-0 string literal —
/// NOT as `1'd0` (the sized-hex coercion would hand the callee a 1-bit
/// value). Compare an empty against a NON-empty through the same path so a
/// width coercion cannot fake a pass.
#[test]
fn empty_string_actual_round_trips() {
    if !gate_on() {
        return;
    }
    let src = r#"
class D;
  function string echo(string s);
    return s;
  endfunction
  function bit same(string a, string b);
    if (a == b) return 1;
    return 0;
  endfunction
endclass
module top;
  string re;
  int rs;
  initial begin
    D d = new();
    re = d.echo("");
    rs = d.same(d.echo(""), "a");
  end
endmodule
"#;
    let sim = simulate(src, 100).expect("simulation should run");
    assert_eq!(s(&sim, "re"), "");
    assert_eq!(u(&sim, "rs"), 0);
}

/// Non-pass-through uses of a string formal (string methods like `len`)
/// keep the method on the AST interpreter — the all-or-nothing contract.
/// Results must still be correct, byte-for-byte with the reference
/// simulator (validated shape).
#[test]
fn string_formal_len_stays_interpreted() {
    if !gate_on() {
        return;
    }
    let src = r#"
class E;
  function int len_of(string s);
    return s.len();
  endfunction
endclass
module top;
  int rl;
  initial begin
    E e = new();
    rl = e.len_of("xyz");
  end
endmodule
"#;
    let sim = simulate(src, 100).expect("simulation should run");
    assert_eq!(u(&sim, "rl"), 3);
}
