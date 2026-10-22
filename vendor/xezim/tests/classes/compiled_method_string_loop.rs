//! class-perf task (a), second shape: string-verb calls INSIDE a
//! register-backed `for (int k = ...)` loop.
//!
//! A method whose body runs `for (int k = 0; k < s.len(); k++)` and calls a
//! pure string verb (`s.getc(k)`, `s.substr(...)`, ...) previously fell back
//! to the AST interpreter for two stacked reasons:
//!   - `s.getc`/`s.len` parse as a bare `ExprKind::MemberAccess` in the
//!     method-mode compiler, so `string_method_shape` (which only matched a
//!     dotted `Ident`) — and hence `compile_string_method` — declined; and
//!   - a counter-increment body statement `n++` lowers to
//!     `StatementKind::Expr(Unary PostIncr n)` (not `BlockingAssign`), which
//!     `for_body_is_simple` rejected, so the `For_init_vardecl` register path
//!     never even admitted the loop.
//!
//! Both are now admitted. A pure string verb lowers to a StrOp (not routed
//! through the interpreter's string-formal dispatcher), and the admissions
//! are merely OPTIMISTIC: any body statement that later cannot compile while
//! `reg_var_loop_depth > 0` still bails the whole loop back to the AST
//! interpreter, so gate-ON and gate-OFF are byte-identical. Expectations are
//! reference-simulator-validated (the `module top` shape runs byte-for-byte
//! identically under the reference simulator, gate OFF, gate ON, and
//! compiled). The gate is forced ON so a plain `cargo test` exercises the
//! compiled path.
use xezim::simulate;

fn u(sim: &xezim::compiler::Simulator, n: &str) -> u64 {
    sim.get_signal(n)
        .or_else(|| sim.get_signal(&format!("top.{}", n)))
        .unwrap_or_else(|| panic!("signal not found: {}", n))
        .to_u64()
        .unwrap_or_else(|| panic!("{} is x/z", n))
}

// A deterministically-probed method that exercises BOTH `s.len()` in the
// loop condition and `s.getc(k)` in the body, folding every char into a
// checksum integer that is byte-identical between the AST interpreter and
// the compiled path.
const HOT_SRC: &str = r#"
class Svc;
  // `count_edit` folds each non-space char (and its index) into a checksum.
  function automatic int count_edit(string s);
    int n;
    int k;
    int acc;
    n = 0;
    acc = 0;
    for (k = 0; k < s.len(); k++) begin
      byte ch;
      ch = s.getc(k);
      if (ch != " ") begin
        n++;
        acc = acc * 31 + ch;
      end
      acc = acc + k * 7;
    end
    return n * 1_000_000 + acc;
  endfunction
endclass

module top;
  integer a;
  integer b;
  initial begin
    Svc sv;
    sv = new();
    a = sv.count_edit("  hello UVM report body  ");
    b = sv.count_edit("zzzz"); // no spaces -> all counted
  end
endmodule
"#;

#[test]
fn string_verb_calls_inside_vardecl_for_match_reference() {
    if !super::compiled_method_test_env::eager() {
        return;
    }
    let sim = simulate(HOT_SRC, 100).expect("compiled simulation should run");
    // Reference-validated (reference simulator) and byte-identical across gate ON/OFF:
    // TAG a=-1598707712 b=7762830. `a` folded a 16-char UVM body through the
    // *31+k*7 checksum, overflowing a 32-bit int into the reference value
    // (as a u64 signal read that is 2^32 + (-1598707712) = 2696259584); both
    // are pinned exactly as the reference simulator prints.
    assert_eq!(
        u(&sim, "a"),
        2696259584,
        "compiled a must equal the reference checksum"
    );
    assert_eq!(
        u(&sim, "b"),
        7762830,
        "compiled b must equal the reference checksum"
    );
}

#[test]
fn string_verb_for_loop_gate_on_off_byte_identical() {
    if !super::compiled_method_test_env::policies(&[("0", "0"), ("1", "0")]) {
        return;
    }
    let sim = simulate(HOT_SRC, 100).expect("simulation should run");
    assert_eq!(u(&sim, "a"), 2696259584);
    assert_eq!(u(&sim, "b"), 7762830);
}
