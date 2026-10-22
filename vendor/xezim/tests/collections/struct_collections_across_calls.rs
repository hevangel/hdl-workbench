//! §7.10 / §13.5.2 — collections and compound structs crossing a subroutine
//! boundary. Reference-validated.
//!
//! 1. **A task bound no queue / dynamic-array / associative formal at all.**
//!    Only functions did. The body operated on storage named after the FORMAL,
//!    so `task push(ref int q[$]); q.push_back(v);` left the caller's queue
//!    empty — and that formal-named storage persisted between calls, so the
//!    next call saw the previous one's elements. An `input` formal read them
//!    too, which is worse than an empty result: it is another call's data.
//! 2. **Elements of a struct COLLECTION crossed as scalars.** An element of an
//!    unpacked-struct queue or array is a set of member leaves; copying `q[j]`
//!    alone moved nothing, so a `ref pkt_t q[$]` came back the right SIZE with
//!    every member zero, and a `pkt_t a[3]` formal read zero throughout.
//! 3. **A nested or dimensioned member of a struct was unbound in a frame.**
//!    Seeding and binding walked only top-level members, but `s.inner.x` and
//!    `s.arr[i]` have no storage above the leaf.
//! 4. **A struct with an ARRAY member read x for every element in an instance.**
//!    Only the per-element leaves were registered, not the member's bounds, so
//!    the index degraded to a bit-select of a 1-bit unknown.

use xezim::simulate;

fn u(sim: &xezim::compiler::Simulator, n: &str) -> u64 {
    sim.get_signal(n)
        .or_else(|| sim.get_signal(&format!("tb.{}", n)))
        .unwrap_or_else(|| panic!("signal not found: {}", n))
        .to_u64()
        .unwrap_or_else(|| panic!("{} not u64-able (x/z?)", n))
}

/// Queue, dynamic-array and associative formals of a TASK, every direction.
#[test]
fn task_collection_formals_bind_and_write_back() {
    let src = r#"
module tb;
  int qr[$], qi[$], qo[$], qin[$], dyn[], as[string];
  int r_n, i_n, o_n, read_n, d_n, d0, as_k;
  task automatic by_ref  (ref    int q[$]); q.push_back(7); endtask
  task automatic by_inout(inout  int q[$]); q.push_back(7); endtask
  task automatic by_out  (output int q[$]); q.push_back(7); endtask
  task automatic by_in   (input  int q[$], output int n); n = q.size(); endtask
  task automatic grow    (ref int d[]);      d = new[2]; d[0] = 5; endtask
  task automatic set_as  (ref int a[string]); a["k"] = 9; endtask
  initial begin
    by_ref(qr); by_inout(qi); by_out(qo);
    qin.push_back(1); qin.push_back(2);
    by_in(qin, read_n);
    grow(dyn);
    set_as(as);
    #1;
    r_n = qr.size(); i_n = qi.size(); o_n = qo.size();
    d_n = dyn.size(); d0 = dyn[0]; as_k = as["k"];
  end
endmodule
"#;
    let sim = simulate(src, 50).expect("simulate failed");
    assert_eq!(u(&sim, "r_n"), 1, "ref queue formal");
    assert_eq!(u(&sim, "i_n"), 1, "inout queue formal");
    assert_eq!(u(&sim, "o_n"), 1, "output queue formal");
    assert_eq!(
        u(&sim, "read_n"),
        2,
        "an input formal sees the caller's queue, not a previous call's"
    );
    assert_eq!(
        (u(&sim, "d_n"), u(&sim, "d0")),
        (2, 5),
        "ref dynamic-array formal"
    );
    assert_eq!(u(&sim, "as_k"), 9, "ref associative formal");
}

/// Collections whose ELEMENT is an unpacked struct, in and out.
#[test]
fn struct_elements_cross_a_formal_boundary() {
    let src = r#"
typedef struct { int a; logic [7:0] b; } s_t;
module tb;
  s_t sq[$];
  s_t sarr[3];
  s_t filled[3];
  int sq_n, sq0a, sq0b, arr_sum, out_a;
  task automatic push_struct(ref s_t q[$]);
    s_t t; t.a = 3; t.b = 8'h33; q.push_back(t);
  endtask
  function automatic int sum_arr(input s_t ar[3]);
    return ar[0].a + ar[1].a + ar[2].a;
  endfunction
  task automatic fill(output s_t ar[3]); ar[0].a = 77; endtask
  initial begin
    push_struct(sq);
    sarr[0].a = 1; sarr[1].a = 2; sarr[2].a = 4;
    arr_sum = sum_arr(sarr);
    fill(filled);
    #1;
    sq_n = sq.size(); sq0a = sq[0].a; sq0b = sq[0].b;
    out_a = filled[0].a;
  end
endmodule
"#;
    let sim = simulate(src, 50).expect("simulate failed");
    assert_eq!(u(&sim, "sq_n"), 1);
    assert_eq!(
        (u(&sim, "sq0a"), u(&sim, "sq0b")),
        (3, 0x33),
        "queue-of-structs element members"
    );
    assert_eq!(u(&sim, "arr_sum"), 7, "struct-array input formal");
    assert_eq!(u(&sim, "out_a"), 77, "struct-array output formal");
}

/// A NESTED unpacked struct through every scope and boundary.
#[test]
fn nested_structs_through_scopes_and_boundaries() {
    let src = r#"
typedef struct { logic [7:0] x; logic [7:0] y; } inner_t;
typedef struct { inner_t i; logic [7:0] z; }     outer_t;
module leaf;
  outer_t o;
  initial begin o.i.x = 8'h11; o.i.y = 8'h12; o.z = 8'h13; end
endmodule
module tb;
  outer_t mo, copied, got, made;
  leaf u();
  int m_x, i_x, c_x, l_x, l_z, f_in, o_x, r_x, r_z;
  function automatic logic [7:0] read_local();
    outer_t t;
    t.i.x = 8'h21; t.i.y = 8'h22; t.z = 8'h23;
    l_z = t.z;
    return t.i.x;
  endfunction
  function automatic logic [7:0] take(input outer_t v); return v.i.y; endfunction
  task automatic give(output outer_t ov); ov.i.x = 8'h31; ov.z = 8'h33; endtask
  function automatic outer_t make_it();
    make_it.i.x = 8'h41; make_it.z = 8'h43;
  endfunction
  initial begin
    mo.i.x = 8'h01; mo.i.y = 8'h02; mo.z = 8'h03;
    copied = mo;
    #1;
    m_x = mo.i.x; i_x = u.o.i.x; c_x = copied.i.x;
    l_x = read_local();
    f_in = take(mo);
    give(got);   o_x = got.i.x;
    made = make_it(); r_x = made.i.x; r_z = made.z;
  end
endmodule
"#;
    let sim = simulate(src, 50).expect("simulate failed");
    assert_eq!(u(&sim, "m_x"), 0x01, "module scope");
    assert_eq!(u(&sim, "i_x"), 0x11, "instance scope");
    assert_eq!(u(&sim, "c_x"), 0x01, "whole nested copy");
    assert_eq!(
        (u(&sim, "l_x"), u(&sim, "l_z")),
        (0x21, 0x23),
        "subroutine local"
    );
    assert_eq!(u(&sim, "f_in"), 0x02, "nested member of an input formal");
    assert_eq!(u(&sim, "o_x"), 0x31, "nested member of an output formal");
    assert_eq!(
        (u(&sim, "r_x"), u(&sim, "r_z")),
        (0x41, 0x43),
        "nested struct return"
    );
}

/// A struct with an unpacked-ARRAY member, through every scope and boundary.
#[test]
fn struct_with_array_member_through_scopes() {
    let src = r#"
typedef struct { logic [7:0] tag; logic [7:0] arr [3]; } sa_t;
module leaf;
  sa_t s;
  initial begin s.tag = 8'h11; s.arr[1] = 8'hA1; end
endmodule
module tb;
  sa_t mod_s, got, made;
  leaf u();
  int m_a0, i_a1, l_a1, f_in, o_a1, r_a1;
  function automatic logic [7:0] read_local();
    sa_t t; t.tag = 8'h21; t.arr[1] = 8'hB1; return t.arr[1];
  endfunction
  function automatic logic [7:0] take(input sa_t x); return x.arr[1]; endfunction
  task automatic give(output sa_t o); o.tag = 8'h31; o.arr[1] = 8'hC1; endtask
  function automatic sa_t make_it(); make_it.tag = 8'h41; make_it.arr[1] = 8'hD1; endfunction
  initial begin
    mod_s.arr[0] = 8'h90; mod_s.arr[1] = 8'h91;
    #1;
    m_a0 = mod_s.arr[0];
    i_a1 = u.s.arr[1];
    l_a1 = read_local();
    f_in = take(mod_s);
    give(got);   o_a1 = got.arr[1];
    made = make_it(); r_a1 = made.arr[1];
  end
endmodule
"#;
    let sim = simulate(src, 50).expect("simulate failed");
    assert_eq!(u(&sim, "m_a0"), 0x90, "module scope");
    assert_eq!(u(&sim, "i_a1"), 0xA1, "array member inside an instance");
    assert_eq!(u(&sim, "l_a1"), 0xB1, "subroutine local");
    assert_eq!(u(&sim, "f_in"), 0x91, "array member of an input formal");
    assert_eq!(u(&sim, "o_a1"), 0xC1, "array member of an output formal");
    assert_eq!(u(&sim, "r_a1"), 0xD1, "returned struct's array member");
}
