//! Unpacked-struct values passed through `output` / `inout` / `ref` formals.
//!
//! Before this fix, a struct `output`/`inout`/`ref` formal was bound
//! member-wise into per-member locals (`o.a`, `o.b`) but:
//!   * never registered for write-back (the binding `continue`d before the
//!     `output_bindings` push), and
//!   * the body's `o.a = v` write was silently dropped (the `MemberAccess`
//!     write arm didn't look in `local_stack` for dotted keys).
//! so the caller's actual was left untouched (`sout.a` read back `x`).
//!
//! Covers: a module `output` formal, an `inout` formal (caller value flows in
//! and back), a `ref` formal, a concrete and a type-parameter method
//! `output` formal, and an `output` whose actual is a CLASS PROPERTY.

use xezim::simulate;

#[test]
fn struct_output_inout_ref_formals() {
    const SRC: &str = r#"
typedef struct {
  int a;
  int b;
} s_t;

// output: formal starts fresh, body fills it, write back.
function void make_pair(output s_t o);
  o.a = 7;
  o.b = 8;
endfunction

// inout: caller's value flows in, body modifies, flows back.
function void bump(inout s_t io);
  io.a = io.a + 100;
  io.b = io.b + 200;
endfunction

// ref: same write-back path as inout.
function automatic void scale(ref s_t r);
  r.a = r.a * 2;
  r.b = r.b * 2;
endfunction

// type-parameter method output formal.
class producer #(type T = int);
  function void fill(output T o);
    o.a = 11;
    o.b = 22;
  endfunction
endclass

// a class whose PROPERTY is the actual of an output formal.
class box;
  s_t item;
  function void load(output s_t o);
    o.a = 42;
    o.b = 43;
  endfunction
endclass

module top;
  int pass_count;
  initial begin
    s_t s, sout;
    producer #(s_t) p;
    box bx;
    pass_count = 0;

    make_pair(sout);
    if (sout.a == 7 && sout.b == 8) pass_count++;

    s.a = 1; s.b = 2;
    bump(s);
    if (s.a == 101 && s.b == 202) pass_count++;

    s.a = 3; s.b = 4;
    scale(s);
    if (s.a == 6 && s.b == 8) pass_count++;

    p = new();
    p.fill(sout);
    if (sout.a == 11 && sout.b == 22) pass_count++;

    bx = new();
    bx.load(bx.item);
    if (bx.item.a == 42 && bx.item.b == 43) pass_count++;
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
        pc, 5,
        "output/inout/ref struct formals (module fn, method, type-param, class-prop actual) failed"
    );
}
