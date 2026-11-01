//! §8.3.2/§6.20.2/§26.3 (issue #388): WHOLE-COLLECTION references to
//! class-scope ARRAY constants — the `$size`/`$left`/`$high` query family,
//! `inside` membership, `foreach`, and whole-array copy — on a class
//! localparam array, a static fixed-array property, a foreign class's
//! constant, and a package array read from inside a class method.
//!
//! The element-read arm answers one index at a time; the whole-array
//! contexts need the bounds and an element source up front, and no
//! module-scope shape row exists for a class-scope constant. Before the
//! resolver they collapsed: `$size(pkg::Cls::T)` reported the operand's
//! bit width (32), `v inside {Cls::T}` never matched, `foreach` iterated
//! nothing, and `dst = Cls::T` left the destination at its defaults.
//! `resolve_class_scope_array_const` answers every context from the same
//! per-element cells the element reads use — the materialized
//! `class_param_arrays` cell for a param/localparam constant, the
//! `Cls::name[i]` signals for a static fixed array, and the `pkg::T[i]`
//! signals for a package localparam (which parses as MemberAccess inside
//! a method body, so the flat-Ident arms never saw it).

use xezim::simulate;

fn out(src: &str) -> String {
    let sim = simulate(src, 200).expect("simulate failed");
    sim.output
        .iter()
        .map(|o| o.message.as_str())
        .collect::<Vec<_>>()
        .join("\n")
}

/// A class localparam array: queries and bulk reads through the
/// package-qualified, foreign-class and in-method bare spellings.
#[test]
fn localparam_array_whole_refs() {
    const SRC: &str = r#"
package p;
  class C;
    localparam int T [4] = '{1, 2, 4, 8};
    function int sz(); return $size(T); endfunction
    function int sum();
      int s = 0;
      foreach (T[k]) s += T[k];
      return s;
    endfunction
    function bit has(int v); return (v inside {T}); endfunction
  endclass
endpackage
module top;
  import p::*;
  int copy [4];
  initial begin
    automatic C c = new;
    copy = p::C::T;
    $display("Q %0d %0d %0d %0d", $size(p::C::T), $left(p::C::T), $high(p::C::T), $size(C::T));
    $display("M %0d %0d %0d", c.sz(), c.sum(), c.has(4));
    $display("Y %0d %0d", copy[0], copy[3]);
    $finish;
  end
endmodule
"#;
    let o = out(SRC);
    assert!(
        o.contains("Q 4 0 3 4"),
        "queries on the class-scope constant must report its bounds, not a bit width:\n{}",
        o
    );
    assert!(
        o.contains("M 4 15 1"),
        "in-method $size/foreach/inside over the bare name:\n{}",
        o
    );
    assert!(
        o.contains("Y 1 8"),
        "whole-array copy from the class-scope constant:\n{}",
        o
    );
}

/// A static fixed-array property: the shared `Cls::name` store answers
/// queries and copies in every spelling.
#[test]
fn static_fixed_array_whole_refs() {
    const SRC: &str = r#"
package p;
  class C;
    static int SA [3] = '{5, 6, 7};
  endclass
endpackage
module top;
  import p::*;
  int copy [3];
  initial begin
    copy = p::C::SA;
    $display("S %0d %0d %0d %0d", $size(p::C::SA), $size(C::SA), copy[0], copy[2]);
    $display("E %0d", p::C::SA[1]);
    $finish;
  end
endmodule
"#;
    let o = out(SRC);
    assert!(
        o.contains("S 3 3 5 7"),
        "queries and copy on the static fixed-array property:\n{}",
        o
    );
    assert!(o.contains("E 6"), "element read is unchanged:\n{}", o);
}

/// A foreign class's constant, including a parameterized one.
#[test]
fn foreign_class_param_array_whole_refs() {
    const SRC: &str = r#"
package other;
  class C #(parameter int W = 3);
    localparam int T [3] = '{W, W + 1, W + 2};
  endclass
endpackage
module top;
  int copy [3];
  initial begin
    copy = other::C::T;
    $display("F %0d %0d %0d", $size(other::C::T), other::C::T[2], copy[2]);
    $finish;
  end
endmodule
"#;
    let o = out(SRC);
    assert!(
        o.contains("F 3 5 5"),
        "foreign class constant: query, element and copy:\n{}",
        o
    );
}

/// A package array read from inside a class: the constructor's
/// whole-array copy into an instance field, and `inside` in a method.
#[test]
fn class_field_from_package_array() {
    const SRC: &str = r#"
package p;
  localparam int unsigned T [4] = '{1, 2, 4, 8};
  localparam int unsigned S [2] = '{16, 32};
endpackage
class C;
  import p::*;
  int unsigned arr_fld [4];
  function new(); arr_fld = p::T; endfunction
  function bit has(int v); return (v inside {p::S}); endfunction
endclass
module top;
  initial begin
    automatic C c = new;
    $display("C %0d %0d %0d", c.arr_fld[3], c.has(16), c.has(17));
    $finish;
  end
endmodule
"#;
    let o = out(SRC);
    assert!(
        o.contains("C 8 1 0"),
        "ctor copy into the instance field and method inside:\n{}",
        o
    );
}