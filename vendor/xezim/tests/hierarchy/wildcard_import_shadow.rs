//! §26.3 — a local declaration shadows a name a WILDCARD import brought in,
//! and the shadowed name stays reachable package-qualified as `pkg::NAME`.
//!
//! Enum members of an imported package are registered under their BARE name in
//! one flat design-wide map, so both halves of that rule were broken when the
//! shadowed name was an enum member:
//!
//!   * The declaration walk removes the imported member when it reaches the
//!     local declaration, but `process_typedef` runs again for the same package
//!     after that walk (once per import site and per re-export hop) and each of
//!     those re-inserted the member — silently undoing the shadow. The local
//!     inherited the ENUM's width and value, so `int INIT_STATE = 500;` under
//!     `import p::*` became a 2-bit signal reading 0, and a class handle
//!     declared over a member name was unusable.
//!
//!   * `pkg::NAME` resolved through that same bare-name entry, so once the
//!     shadow was honoured the qualified form fell through to the LOCAL. It
//!     read the local's value truncated to the enum's width — which for the
//!     first member is 0, i.e. a wrong answer that still looks like a valid
//!     enum. `.name()` then stringified the local's bits.
//!
//! Both directions are pinned here: the local keeps its own type, and the
//! qualified reference keeps reaching the enum.

use xezim::simulate;

fn lookup(sim: &xezim::compiler::Simulator, name: &str) -> u64 {
    sim.get_signal(name)
        .or_else(|| sim.get_signal(&format!("top.{}", name)))
        .unwrap_or_else(|| panic!("signal not found: {}", name))
        .to_u64()
        .unwrap_or_else(|| panic!("signal {} not u64-able", name))
}

/// The package every test imports. `A`/`B`/`C` are deliberately short so a
/// local declaration can shadow them with an unrelated type.
const PKG: &str = r#"
package pk;
  typedef enum logic [1:0] { A = 2'b00, B = 2'b01, C = 2'b10 } e_t;
endpackage
class kls;
  int n;
  function new(); n = 42; endfunction
endclass
"#;

/// A local `int`/`real`/class handle declared over a wildcard-imported enum
/// member keeps its OWN type and initializer. Each of these read the enum's
/// value at the enum's width before the fix.
#[test]
fn local_declaration_shadows_imported_enum_member() {
    let src = format!(
        r#"{PKG}
module top;
  import pk::*;
  int  A = 500;      // shadows enum member A (value 0)
  real B = 3.5;      // shadows enum member B (value 1)
  kls  C;            // shadows enum member C (value 2)

  int a_val, b_is_35, c_field, a_bits;
  initial begin
    C = new();
    a_val   = A;
    b_is_35 = (B == 3.5);
    c_field = C.n;
    a_bits  = $bits(A);
  end
endmodule
"#
    );
    let sim = simulate(&src, 5).expect("simulate failed");
    assert_eq!(
        lookup(&sim, "a_val"),
        500,
        "local int keeps its initializer, not enum A's 0"
    );
    assert_eq!(
        lookup(&sim, "b_is_35"),
        1,
        "local real keeps 3.5, not enum B's 1"
    );
    assert_eq!(
        lookup(&sim, "c_field"),
        42,
        "local class handle is usable, not enum C's bits"
    );
    assert_eq!(
        lookup(&sim, "a_bits"),
        32,
        "local int is 32 bits, not the enum's 2"
    );
}

/// A procedural assignment to the shadowed local must not be masked to the
/// enum's width — the narrowing was the visible half of the width bug.
#[test]
fn assignment_to_shadowed_local_is_not_masked_to_enum_width() {
    let src = format!(
        r#"{PKG}
module top;
  import pk::*;
  int A;
  int seen;
  initial begin
    A = 700;        // 700 & 2'b11 == 0 if the enum's width wins
    seen = A;
  end
endmodule
"#
    );
    let sim = simulate(&src, 5).expect("simulate failed");
    assert_eq!(lookup(&sim, "seen"), 700, "shadowed local is 32 bits wide");
}

/// §26.3's other half: shadowing the bare name must NOT make the member
/// unreachable. `pk::C` has to keep yielding 2 even though `C` is a local.
#[test]
fn shadowed_member_still_reachable_package_qualified() {
    let src = format!(
        r#"{PKG}
module top;
  import pk::*;
  int A = 501;      // low two bits are 1, so a fall-through to the local
  kls C;            // cannot pass for enum A's 0 by coincidence
  e_t v;
  int qa, qc, qa_name_ok, qc_name_ok;
  initial begin
    C = new();
    v = pk::A; qa = v; qa_name_ok = (v.name() == "A");
    v = pk::C; qc = v; qc_name_ok = (v.name() == "C");
  end
endmodule
"#
    );
    let sim = simulate(&src, 5).expect("simulate failed");
    assert_eq!(
        lookup(&sim, "qa"),
        0,
        "pk::A is enum A, not the local int's low bits"
    );
    assert_eq!(
        lookup(&sim, "qc"),
        2,
        "pk::C is enum C, not the local handle's id"
    );
    assert_eq!(lookup(&sim, "qa_name_ok"), 1, "pk::A stringifies as \"A\"");
    assert_eq!(lookup(&sim, "qc_name_ok"), 1, "pk::C stringifies as \"C\"");
}

/// `pkg::MEMBER` inside a function/task body, with no shadowing anywhere.
///
/// The elaborator only collapses `MemberAccess` into a two-segment `Ident` for
/// always/initial/continuous-assign code, so a subroutine body keeps the
/// `MemberAccess` shape — and that arm resolved a CLASS base but never a
/// package one, then fell through to its object-handle fallback and returned 0.
/// Every package-qualified enum constant read inside a subroutine was 0.
#[test]
fn package_qualified_member_resolves_inside_a_subroutine() {
    let src = format!(
        r#"{PKG}
module top;
  int from_func, from_task, from_method, from_initial;

  function automatic int f(); return pk::C; endfunction
  task automatic t(output int o); o = pk::B; endtask

  initial begin
    from_initial = pk::C;
    from_func = f();
    t(from_task);
  end
endmodule
"#
    );
    let sim = simulate(&src, 5).expect("simulate failed");
    assert_eq!(lookup(&sim, "from_initial"), 2, "pk::C in an initial block");
    assert_eq!(
        lookup(&sim, "from_func"),
        2,
        "pk::C in a function body, not 0"
    );
    assert_eq!(lookup(&sim, "from_task"), 1, "pk::B in a task body, not 0");
}

/// Members that nothing shadows keep resolving bare AND qualified — the fix
/// must not cost the ordinary case.
#[test]
fn unshadowed_members_resolve_bare_and_qualified() {
    let src = format!(
        r#"{PKG}
module top;
  import pk::*;
  int A = 500;        // only A is shadowed
  e_t v;
  int bare_b, bare_c, qual_b, qual_c, name_c_ok;
  initial begin
    v = B;     bare_b = v;
    v = C;     bare_c = v;
    v = pk::B; qual_b = v;
    v = pk::C; qual_c = v; name_c_ok = (v.name() == "C");
  end
endmodule
"#
    );
    let sim = simulate(&src, 5).expect("simulate failed");
    assert_eq!(lookup(&sim, "bare_b"), 1, "unshadowed B resolves bare");
    assert_eq!(lookup(&sim, "bare_c"), 2, "unshadowed C resolves bare");
    assert_eq!(lookup(&sim, "qual_b"), 1, "unshadowed B resolves qualified");
    assert_eq!(lookup(&sim, "qual_c"), 2, "unshadowed C resolves qualified");
    assert_eq!(
        lookup(&sim, "name_c_ok"),
        1,
        "unshadowed C stringifies as \"C\""
    );
}
