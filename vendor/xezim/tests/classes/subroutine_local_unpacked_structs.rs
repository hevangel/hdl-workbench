//! §7.2 / §13.4.2 — UNPACKED-struct variables declared inside a function or
//! task. Reference-validated.
//!
//! Members of such a variable belong to the call frame, stored under their
//! dotted name — the convention formal binding already used. Nothing seeded
//! those keys for a LOCAL declaration, and neither the read nor the write path
//! for a member consulted the frame, so a member reference escaped to the
//! module signal table and resolved to whatever unrelated variable happened to
//! share the bare name.
//!
//! The result was silent cross-scope corruption, and it corrupted in the
//! READER: a function that wrote `s.a = 8'h32` and returned `s.a` returned the
//! CALLER's `s.a` instead, while its own write went somewhere neither side read.
//! The caller's value was not clobbered, so nothing looked wrong from outside —
//! the function just quietly computed with someone else's data. It only
//! appeared when some other scope declared a struct of the same name, which is
//! why the same function tested in isolation was correct.
//!
//! Shadowing a MODULE-scope name already worked (those locals are alpha-renamed);
//! it was process-block locals elsewhere in the design that collided.

use xezim::simulate;

fn u(sim: &xezim::compiler::Simulator, n: &str) -> u64 {
    sim.get_signal(n)
        .or_else(|| sim.get_signal(&format!("tb.{}", n)))
        .unwrap_or_else(|| panic!("signal not found: {}", n))
        .to_u64()
        .unwrap_or_else(|| panic!("{} not u64-able (x/z?)", n))
}

/// A function's local must not read another scope's same-named struct, and must
/// not write it either.
#[test]
fn function_local_struct_does_not_alias_a_process_local() {
    let src = r#"
typedef struct { logic [7:0] a; logic [7:0] b; } su_t;
module tb;
  int callee_reads, caller_after;
  function automatic logic [7:0] f();
    su_t ss;
    ss.a = 8'h32;
    callee_reads = ss.a;
    return 8'h00;
  endfunction
  initial begin
    su_t ss;
    ss.a = 8'h88;
    void'(f());
    caller_after = ss.a;
  end
endmodule
"#;
    let sim = simulate(src, 50).expect("simulate failed");
    assert_eq!(
        u(&sim, "callee_reads"),
        0x32,
        "the callee must read back its OWN write"
    );
    assert_eq!(
        u(&sim, "caller_after"),
        0x88,
        "and must not disturb the caller's"
    );
}

/// The collision can come from any scope: the calling block, another block, or
/// module scope.
#[test]
fn locals_shadow_every_outer_scope() {
    let src = r#"
typedef struct { logic [7:0] a; logic [7:0] b; } su_t;
module tb;
  su_t mt;
  int r_mod, r_same, r_other;
  function automatic logic [7:0] f_mod();   su_t mt; mt.a = 8'h31; return mt.a; endfunction
  function automatic logic [7:0] f_same();  su_t ss; ss.a = 8'h32; return ss.a; endfunction
  function automatic logic [7:0] f_other(); su_t oo; oo.a = 8'h33; return oo.a; endfunction
  initial begin
    su_t oo;
    oo.a = 8'h99;
  end
  initial begin
    su_t ss;
    ss.a = 8'h88;
    mt.a = 8'h77;
    r_mod = f_mod(); r_same = f_same(); r_other = f_other();
  end
endmodule
"#;
    let sim = simulate(src, 50).expect("simulate failed");
    assert_eq!(u(&sim, "r_mod"), 0x31, "shadowing a module-scope struct");
    assert_eq!(
        u(&sim, "r_same"),
        0x32,
        "shadowing a local of the CALLING block"
    );
    assert_eq!(
        u(&sim, "r_other"),
        0x33,
        "shadowing a local of another block"
    );
}

/// Tasks too, and a whole-struct pattern write into a subroutine local must
/// reach the same storage its member reads use.
#[test]
fn task_locals_and_whole_pattern_writes() {
    let src = r#"
typedef struct { logic [7:0] a; logic [7:0] b; } su_t;
module tb;
  int fn_a, tk_a, tk_b, pat_a, pat_b;
  function automatic logic [7:0] fn_local();
    su_t t;
    t.a = 8'h31; t.b = 8'h32;
    return t.a;
  endfunction
  task automatic tk_local(output logic [7:0] oa, output logic [7:0] ob);
    su_t t;
    t.a = 8'h41; t.b = 8'h42;
    oa = t.a; ob = t.b;
  endtask
  function automatic logic [7:0] pattern_then_member(output logic [7:0] second);
    su_t w;
    w = '{a:8'h51, b:8'h52};
    second = w.b;
    return w.a;
  endfunction
  initial begin
    su_t t;
    logic [7:0] a1, b1, p2;
    t.a = 8'h21; t.b = 8'h22;
    fn_a = fn_local();
    tk_local(a1, b1); tk_a = a1; tk_b = b1;
    pat_a = pattern_then_member(p2); pat_b = p2;
  end
endmodule
"#;
    let sim = simulate(src, 50).expect("simulate failed");
    assert_eq!(u(&sim, "fn_a"), 0x31, "function local");
    assert_eq!(
        (u(&sim, "tk_a"), u(&sim, "tk_b")),
        (0x41, 0x42),
        "task local"
    );
    assert_eq!(
        (u(&sim, "pat_a"), u(&sim, "pat_b")),
        (0x51, 0x52),
        "whole pattern into a local"
    );
}

/// Each call gets a FRESH automatic local — a member left unwritten by the
/// second call must not still hold the first call's value.
#[test]
fn automatic_locals_are_fresh_per_call() {
    let src = r#"
typedef struct { logic [7:0] a; logic [7:0] b; } su_t;
module tb;
  int first, second_unwritten;
  function automatic logic [7:0] f(input bit write_b);
    su_t s;
    s.a = 8'h10;
    if (write_b) s.b = 8'h20;
    return s.b;
  endfunction
  initial begin
    first = f(1'b1);
    second_unwritten = $isunknown(f(1'b0));
  end
endmodule
"#;
    let sim = simulate(src, 50).expect("simulate failed");
    assert_eq!(u(&sim, "first"), 0x20);
    assert_eq!(
        u(&sim, "second_unwritten"),
        1,
        "a fresh call must not see the previous one's member"
    );
}

/// §13.5.2 — UNPACKED-struct FORMALS. A task bound none of them, so its body
/// read x from every member of an input; and an output/inout/ref struct formal
/// of either a task or a function copied nothing back, leaving the caller's
/// variable untouched. Function INPUT formals already worked, which is what
/// made the gap look narrower than it was.
#[test]
fn unpacked_struct_formals_bind_and_copy_back() {
    let src = r#"
typedef struct { logic [7:0] a; logic [7:0] b; } su_t;
module tb;
  task automatic t_in (input su_t x, output logic [7:0] r1, output logic [7:0] r2);
    r1 = x.a; r2 = x.b;
  endtask
  function automatic logic [7:0] f_in(input su_t x);
    return x.a;
  endfunction
  task automatic t_out(output su_t o); o.a = 8'h7a; o.b = 8'h7b; endtask
  task automatic t_ref(ref    su_t r); r.a = 8'hF0; endtask
  task automatic t_inout(inout su_t io); io.b = io.a + 8'h01; endtask
  function automatic void f_out(output su_t o); o.a = 8'h6a; o.b = 8'h6b; endfunction
  su_t ui, o_u, ref_u, io_u, fo_u;
  int i1, i2, fi, oa, ob, ra, rb, ioa, iob, foa, fob;
  initial begin
    ui = '{a:8'h01, b:8'h02};
    t_in(ui, i1, i2);
    fi = f_in(ui);
    t_out(o_u);         oa = o_u.a;   ob = o_u.b;
    ref_u = '{a:8'h00, b:8'h0b}; t_ref(ref_u); ra = ref_u.a; rb = ref_u.b;
    io_u  = '{a:8'h30, b:8'h00};  t_inout(io_u); ioa = io_u.a; iob = io_u.b;
    f_out(fo_u);        foa = fo_u.a; fob = fo_u.b;
  end
endmodule
"#;
    let sim = simulate(src, 50).expect("simulate failed");
    assert_eq!(
        (u(&sim, "i1"), u(&sim, "i2")),
        (0x01, 0x02),
        "task input formal"
    );
    assert_eq!(u(&sim, "fi"), 0x01, "function input formal");
    assert_eq!(
        (u(&sim, "oa"), u(&sim, "ob")),
        (0x7a, 0x7b),
        "task output formal"
    );
    assert_eq!(
        (u(&sim, "ra"), u(&sim, "rb")),
        (0xF0, 0x0b),
        "ref formal writes through"
    );
    assert_eq!(
        (u(&sim, "ioa"), u(&sim, "iob")),
        (0x30, 0x31),
        "inout reads in and writes back"
    );
    assert_eq!(
        (u(&sim, "foa"), u(&sim, "fob")),
        (0x6a, 0x6b),
        "function output formal"
    );
}

/// §13.4.1 — struct RETURN types. The return variable is a variable of the
/// return type but had none of the structural metadata a declaration gets: for
/// a PACKED struct return, `f.a = ...` found no field layout and the write was
/// dropped (an identical write to a local of that type worked); for an UNPACKED
/// one there is no container at all, so the result came back x however the body
/// produced it — member writes, a whole pattern, or `return s;`.
#[test]
fn struct_return_types() {
    let src = r#"
typedef struct packed { logic [7:0] a; logic [7:0] b; } sp_t;
typedef struct        { logic [7:0] a; logic [7:0] b; } su_t;
module tb;
  function automatic sp_t p_member(input sp_t x);
    p_member.a = x.a + 8'h10; p_member.b = x.b + 8'h10;
  endfunction
  function automatic sp_t p_whole(input sp_t x);
    p_whole = '{a: x.a + 8'h10, b: x.b + 8'h10};
  endfunction
  function automatic su_t u_member(input su_t x);
    u_member.a = x.a + 8'h10; u_member.b = x.b + 8'h10;
  endfunction
  function automatic su_t u_whole(input su_t x);
    u_whole = '{a: x.a + 8'h10, b: x.b + 8'h10};
  endfunction
  function automatic su_t u_local(input su_t x);
    su_t t; t.a = x.a + 8'h10; t.b = x.b + 8'h10; return t;
  endfunction
  sp_t pi, r_pm, r_pw;
  su_t ui, r_um, r_uw, r_ul;
  int pma, pmb, pwa, pwb, uma, umb, uwa, uwb, ula, ulb;
  initial begin
    pi = '{a:8'h01, b:8'h02};
    ui = '{a:8'h01, b:8'h02};
    r_pm = p_member(pi); pma = r_pm.a; pmb = r_pm.b;
    r_pw = p_whole(pi);  pwa = r_pw.a; pwb = r_pw.b;
    r_um = u_member(ui); uma = r_um.a; umb = r_um.b;
    r_uw = u_whole(ui);  uwa = r_uw.a; uwb = r_uw.b;
    r_ul = u_local(ui);  ula = r_ul.a; ulb = r_ul.b;
  end
endmodule
"#;
    let sim = simulate(src, 50).expect("simulate failed");
    assert_eq!(
        (u(&sim, "pma"), u(&sim, "pmb")),
        (0x11, 0x12),
        "packed return, member writes"
    );
    assert_eq!(
        (u(&sim, "pwa"), u(&sim, "pwb")),
        (0x11, 0x12),
        "packed return, whole pattern"
    );
    assert_eq!(
        (u(&sim, "uma"), u(&sim, "umb")),
        (0x11, 0x12),
        "unpacked return, member writes"
    );
    assert_eq!(
        (u(&sim, "uwa"), u(&sim, "uwb")),
        (0x11, 0x12),
        "unpacked return, whole pattern"
    );
    assert_eq!(
        (u(&sim, "ula"), u(&sim, "ulb")),
        (0x11, 0x12),
        "unpacked return of a local"
    );
}
