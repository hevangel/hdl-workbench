//! A class property that is a scalar INTEGER (not a struct) assigned in a
//! method that a same-named UNPACKED struct elsewhere shadows.
//!
//! `var_decl_types` is keyed by the bare name and shared by every scope
//! (#258), so an unrelated module-scope unpacked struct variable `s` makes
//! `struct_copy_target("s")` answer a struct. UVM's field-automation macros
//! assign `s = __rdone__` for an `int s` (e.g. the 3497 autocfgprec test)
//! while some other scope declares a struct `s`: the whole-struct copy branch
//! diverted the write to the integral property and left `s` stuck at 0.
//!
//! (The class-handle case is covered by `class_formal_named_like_struct`; this
//! is the integral-property variant fixed alongside it.)

use xezim::simulate;

fn messages(src: &str) -> Vec<String> {
    simulate(src, 1000)
        .expect("simulate failed")
        .output
        .iter()
        .map(|o| o.message.clone())
        .collect()
}

const COMMON: &str = r#"
typedef struct { bit [7:0] a; } s_t;
"#;

/// A module-scope unpacked struct variable `s` must not divert `s = v` inside
/// a method writing `this.s` (an `int` property).
#[test]
fn integral_property_write_not_diverted_by_module_struct() {
    let src = format!(
        "{COMMON}{}",
        r#"
class cls;
  int s;
  function void set_s(int val);
    s = val;
  endfunction
endclass

module top;
  s_t s; // unrelated unpacked struct variable sharing the bare name
  initial begin
    automatic cls c = new;
    c.s = 0;
    c.set_s(9);
    if (c.s == 9) $display("TAG_PASS");
    else $display("TAG_FAIL s=%0d", c.s);
  end
endmodule
"#
    );
    assert_eq!(messages(&src), ["TAG_PASS"]);
}

/// The same write through a bare-name assignment produces the exact value the
/// UVM field-op uses (`s = __rdone__`), exercised through several small
/// integers to make sure nothing lands in the struct's leaf instead.
#[test]
fn integral_property_assign_survives_struct_shadow() {
    let src = format!(
        "{COMMON}{}",
        r#"
class cls;
  int s;
  int v;
  function void set(int a, int b);
    v = a;
    s = b;
  endfunction
endclass

module top;
  s_t s;
  initial begin
    automatic cls c = new;
    int i;
    for (i = 0; i < 3; i++) begin
      c.set(i, 10 + i);
      if (c.s != 10 + i || c.v != i)
        $display("TAG_FAIL i=%0d s=%0d v=%0d", i, c.s, c.v);
    end
    if (c.s == 12 && c.v == 2) $display("TAG_PASS");
    else $display("TAG_FAIL s=%0d v=%0d", c.s, c.v);
  end
endmodule
"#
    );
    assert_eq!(messages(&src), ["TAG_PASS"]);
}

/// A method-, block- or task-local unpacked struct named like an `int`
/// property is the local: its leaves live in the frame under `<name>.<member>`,
/// so the name alone is not a frame key. The copy must land in the local.
#[test]
fn struct_local_named_like_int_property_is_local() {
    let src = r#"
typedef struct { int a; int b; } st_t;
class C;
  int s;
  int r;
  function void f();
    st_t s;
    st_t u;
    u.a = 31; u.b = 32;
    s = u;
    $display("T|f local s=%0d,%0d (31,32)", s.a, s.b);
  endfunction
  function void g();
    st_t u;
    u.a = 41; u.b = 42;
    begin
      st_t r;
      r = u;
      $display("T|g block r=%0d,%0d (41,42)", r.a, r.b);
    end
  endfunction
  task t();
    st_t s;
    st_t u;
    u.a = 51; u.b = 52;
    #1;
    s = u;
    $display("T|t local s=%0d,%0d (51,52)", s.a, s.b);
  endtask
endclass
module top;
  initial begin
    C c = new;
    c.s = 3; c.r = 4;
    c.f();
    c.g();
    c.t();
    $display("T|props s=%0d r=%0d (3,4)", c.s, c.r);
  end
endmodule
"#;
    let got: Vec<String> = messages(src)
        .into_iter()
        .filter(|m| m.starts_with("T|"))
        .collect();
    assert_eq!(
        got,
        [
            "T|f local s=31,32 (31,32)",
            "T|g block r=41,42 (41,42)",
            "T|t local s=51,52 (51,52)",
            "T|props s=3 r=4 (3,4)",
        ]
    );
}
/// An `int` property named like a module variable of a package struct type:
/// identifier, literal and call right-hand sides (`s = get()`) all write the
/// property, never the module struct.
#[test]
fn int_property_named_like_package_struct_variable() {
    let src = r#"
package P;
  typedef struct { int a; int b; } st_t;
endpackage
module top;
  import P::*;
  st_t s;
  st_t sp;
  class C;
    int s;
    st_t sp;
    struct { int a; int b; } si;
    int v = 7;
    function int get(); return 11; endfunction
    function void set_ident(); s = v; endfunction
    function void set_call(); s = get(); endfunction
    function void set_lit(); s = 5; endfunction
    function void set_struct_prop();
      st_t t;
      t.a = 1; t.b = 2;
      sp = t;
    endfunction
    function void set_local_struct();
      st_t s;
      st_t u;
      u.a = 31; u.b = 32;
      s = u;
      $display("T|local struct s.a=%0d s.b=%0d", s.a, s.b);
    endfunction
    function void set_inline();
      si.a = 3; si.b = 4;
      begin int si; si = 9; end
    endfunction
  endclass
  initial begin
    C c = new;
    c.set_ident(); $display("T|ident s=%0d (7)", c.s);
    c.set_lit();   $display("T|lit s=%0d (5)", c.s);
    c.set_call();  $display("T|call s=%0d (11)", c.s);
    c.set_struct_prop(); $display("T|struct prop sp=%0d,%0d", c.sp.a, c.sp.b);
    c.set_local_struct(); $display("T|after local, prop s=%0d (11)", c.s);
    c.set_inline(); $display("T|inline si=%0d,%0d", c.si.a, c.si.b);
    $display("T|global s=%0d,%0d sp=%0d,%0d", s.a, s.b, sp.a, sp.b);
  end
endmodule
"#;
    let got: Vec<String> = messages(src)
        .into_iter()
        .filter(|m| m.starts_with("T|"))
        .collect();
    assert_eq!(
        got,
        [
            "T|ident s=7 (7)",
            "T|lit s=5 (5)",
            "T|call s=11 (11)",
            "T|struct prop sp=1,2",
            "T|local struct s.a=31 s.b=32",
            "T|after local, prop s=11 (11)",
            "T|inline si=3,4",
            "T|global s=0,0 sp=0,0",
        ]
    );
}
/// A struct-typed property, an `int` property and a block-local struct, each
/// named like a module struct variable.
#[test]
fn struct_property_and_block_struct_named_like_globals() {
    let src = r#"
typedef struct { int a; int b; } st_t;
module top;
  st_t s;
  st_t k;
  class C;
    st_t s;           // genuinely struct-typed property, same name as global
    int k;
    function void f();
      st_t t;
      t.a = 4; t.b = 5;
      s = t;
    endfunction
    function void g();
      int j = 9;
      k = j;
    endfunction
    function void h();
      st_t t2;
      t2.a = 1; t2.b = 2;
      begin
        st_t k;      // local struct shadowing int property
        k = t2;
        $display("T|h local k=%0d,%0d", k.a, k.b);
      end
    endfunction
  endclass
  initial begin
    C c = new;
    c.f(); $display("T|struct prop s=%0d,%0d (4,5)", c.s.a, c.s.b);
    $display("T|global s=%0d,%0d (0,0)", s.a, s.b);
    c.g(); $display("T|int prop k=%0d (9)", c.k);
    c.h(); $display("T|after h k=%0d (9) global k=%0d,%0d", c.k, k.a, k.b);
  end
endmodule
"#;
    let got: Vec<String> = messages(src)
        .into_iter()
        .filter(|m| m.starts_with("T|"))
        .collect();
    assert_eq!(
        got,
        [
            "T|struct prop s=4,5 (4,5)",
            "T|global s=0,0 (0,0)",
            "T|int prop k=9 (9)",
            "T|h local k=1,2",
            "T|after h k=9 (9) global k=0,0",
        ]
    );
}
