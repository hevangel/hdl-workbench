//! A member select on a packed-struct subroutine FORMAL or local must read the
//! call frame, not the signal table.
//!
//! Inlining a non-root module flattens `s.a` into the dotted identifier `s.a`
//! (`rewrite_expr_impl`'s MemberAccess arm). That is right for a module-scope
//! aggregate — the rewrite has just prefixed it to `<inst>.s`, and
//! `packed_struct_fields` carries a layout under that scoped key. It is wrong
//! for a formal or local: those keep their bare name and live in the CALL
//! FRAME. The flattened name then took the packed-leaf path, whose base lookup
//! consulted signals ONLY, so the member read came back x — or, where a
//! same-named module-scope object existed, silently returned THAT object's
//! field instead.
//!
//! Two details made this hard to see: the WHOLE formal always read correctly
//! (only `.field` selects broke), and the same task in the ROOT module worked,
//! because root bodies are never rewritten. It needs `struct packed`
//! specifically — an unpacked struct stores members as separate `base.member`
//! frame keys, so the flattened name happens to match.

use xezim::simulate;

fn notes(src: &str) -> Vec<String> {
    let sim = simulate(src, 1_000_000).expect("simulate failed");
    sim.output
        .iter()
        .map(|o| o.message.trim().to_string())
        .filter(|l| l.starts_with("NOTE:"))
        .collect()
}

/// The distilled case: a packed-struct formal of a task in a NON-root module.
#[test]
fn packed_struct_formal_members_read_in_a_nested_module() {
    let src = r#"
package sp;
  typedef struct packed { bit [7:0] hi; bit [7:0] lo; } pair_t;
endpackage
module leaf();
  import sp::*;
  task Show(input pair_t p);
    $display("NOTE: hi=%0d lo=%0d whole=%0h", p.hi, p.lo, p);
  endtask
endmodule
module top;
  import sp::*;
  leaf u_leaf();
  initial begin
    pair_t v;
    v.hi = 5;
    v.lo = 6;
    u_leaf.Show(v);
    $finish;
  end
endmodule
"#;
    assert_eq!(notes(src), vec!["NOTE: hi=5 lo=6 whole=506"]);
}

/// A same-named module-scope object must NOT capture the formal's member reads —
/// this is the silent-wrong-value form, which is worse than the x form.
#[test]
fn a_same_named_module_variable_does_not_shadow_the_formal() {
    let src = r#"
package sp2;
  typedef struct packed { bit [7:0] hi; bit [7:0] lo; } pair_t;
endpackage
module leaf();
  import sp2::*;
  pair_t p;                      // module-scope, same name as the formal
  initial begin p.hi = 1; p.lo = 2; end
  task Show(input pair_t p);
    $display("NOTE: hi=%0d lo=%0d", p.hi, p.lo);
  endtask
endmodule
module top;
  import sp2::*;
  leaf u_leaf();
  initial begin
    pair_t v;
    v.hi = 5;
    v.lo = 6;
    #1;
    u_leaf.Show(v);
    $finish;
  end
endmodule
"#;
    assert_eq!(
        notes(src),
        vec!["NOTE: hi=5 lo=6"],
        "the formal must win, not the module variable"
    );
}

/// A packed-struct LOCAL declared inside the nested task has the same shape.
#[test]
fn packed_struct_local_members_read_in_a_nested_module() {
    let src = r#"
package sp3;
  typedef struct packed { bit [7:0] hi; bit [7:0] lo; } pair_t;
endpackage
module leaf();
  import sp3::*;
  task Build();
    pair_t t;
    t.hi = 9;
    t.lo = 4;
    $display("NOTE: hi=%0d lo=%0d", t.hi, t.lo);
  endtask
endmodule
module top;
  leaf u_leaf();
  initial begin
    u_leaf.Build();
    $finish;
  end
endmodule
"#;
    assert_eq!(notes(src), vec!["NOTE: hi=9 lo=4"]);
}

/// A module-scope packed struct must still resolve through the flattened path —
/// the fix is frame-FIRST, not frame-only.
#[test]
fn module_scope_packed_struct_members_still_read() {
    let src = r#"
package sp4;
  typedef struct packed { bit [7:0] hi; bit [7:0] lo; } pair_t;
endpackage
module leaf();
  import sp4::*;
  pair_t m;
  task Show();
    $display("NOTE: hi=%0d lo=%0d", m.hi, m.lo);
  endtask
endmodule
module top;
  leaf u_leaf();
  initial begin
    u_leaf.m.hi = 7;
    u_leaf.m.lo = 8;
    #1;
    u_leaf.Show();
    $finish;
  end
endmodule
"#;
    assert_eq!(notes(src), vec!["NOTE: hi=7 lo=8"]);
}
