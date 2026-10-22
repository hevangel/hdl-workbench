//! An UNPACKED ARRAY member of an unpacked struct (`struct { logic [15:0] a [4]; }`).
//!
//! Its elements are individual signals (`s.a[0]` …) and the base `s.a` is not
//! a signal at all, so `s.a[i] = v` matched no lvalue arm and degraded into a
//! bit-select of a phantom scalar — the write was discarded and the member read
//! back x forever. Reads had the mirror-image problem.
//!
//! That is quiet in the worst way: a scoreboard whose expected-value struct has
//! an array member compares x against every DUT output, so it reports a
//! mismatch on every lane of every cycle while the DUT is perfectly correct.
//! Found from exactly such a testbench.
//!
//! A module-level struct has its element signals pre-registered; a PROCEDURAL
//! LOCAL one has nothing registered, so the write creates the element (only for
//! a member path — a plain local vector's bit-write is never diverted).
//!
//! Also fixed here: reading an array member of a QUEUE ELEMENT
//! (`q[0].arr[i]`, a MemberAccess base rather than an Ident), and a
//! HIERARCHICAL select on a packed-2D struct member reached through a PORT
//! (`dut.result.plus_one[i]`) — the port collapses onto the parent's signal,
//! so the element-width map is keyed by THAT name and the lookup has to go
//! through `port_aliases`; without it the select read a single bit.
//!
//! And finally the PROCEDURAL-LOCAL queue of structs, which stored no members
//! at all (every push/pop yielded zeros): the runtime registered a local's
//! declared type only when the declarator had NO dimensions, so a local
//! collection had no element type. `queue_elem_struct` then returned None and
//! `push_back` fell back to a packed scalar copy — and an unpacked struct has
//! no container signal, so every member was lost. Module-scope queues were
//! unaffected because elaboration registers them.
//!
//! Verified byte-identical to a reference simulator.

use xezim::simulate;

fn u(sim: &xezim::compiler::Simulator, n: &str) -> u64 {
    sim.get_signal(n)
        .or_else(|| sim.get_signal(&format!("top.{}", n)))
        .unwrap_or_else(|| panic!("signal not found: {}", n))
        .to_u64()
        .unwrap_or_else(|| panic!("{} not u64-able (x/z?)", n))
}

/// Write then read back an array member of a module-level struct, and copy the
/// whole struct.
#[test]
fn module_level_struct_array_member_round_trips() {
    let src = r#"
module top;
  typedef struct { logic [15:0] arr [4]; int scalar; } s_t;
  s_t a, b;
  int r0, r3, sc, c0, c3, csc;
  initial begin
    a.arr[0] = 16'h1111; a.arr[3] = 16'h4444; a.scalar = 42;
    r0 = a.arr[0]; r3 = a.arr[3]; sc = a.scalar;
    b = a;
    c0 = b.arr[0]; c3 = b.arr[3]; csc = b.scalar;
  end
endmodule
"#;
    let sim = simulate(src, 20).expect("simulate failed");
    assert_eq!(
        u(&sim, "r0"),
        0x1111,
        "element 0 reads back what was written"
    );
    assert_eq!(u(&sim, "r3"), 0x4444, "element 3 likewise");
    assert_eq!(u(&sim, "sc"), 42, "the scalar member still works");
    assert_eq!(
        u(&sim, "c0"),
        0x1111,
        "struct copy carries the array member"
    );
    assert_eq!(u(&sim, "c3"), 0x4444);
    assert_eq!(u(&sim, "csc"), 42, "and the scalar");
}

/// A PROCEDURAL-LOCAL struct: nothing is pre-registered, so the element is
/// created on first write.
#[test]
fn procedural_local_struct_array_member_round_trips() {
    let src = r#"
module top;
  typedef struct { logic [15:0] arr [4]; int scalar; } s_t;
  s_t mod_level;
  int l0, l3, m0, msc;
  initial begin
    s_t loc;
    loc.arr[0] = 16'h1111; loc.arr[3] = 16'h4444; loc.scalar = 7;
    l0 = loc.arr[0]; l3 = loc.arr[3];
    mod_level = loc;
    m0 = mod_level.arr[0]; msc = mod_level.scalar;
  end
endmodule
"#;
    let sim = simulate(src, 20).expect("simulate failed");
    assert_eq!(
        u(&sim, "l0"),
        0x1111,
        "a local struct's array member stores"
    );
    assert_eq!(u(&sim, "l3"), 0x4444);
    assert_eq!(
        u(&sim, "m0"),
        0x1111,
        "and copies out to a module-level struct"
    );
    assert_eq!(u(&sim, "msc"), 7);
}

/// A variable index, and each element independent of its siblings.
#[test]
fn array_member_elements_are_independent_under_a_variable_index() {
    let src = r#"
module top;
  typedef struct { logic [15:0] arr [4]; } s_t;
  s_t s;
  int e0, e1, e2, e3;
  initial begin
    for (int i = 0; i < 4; i++) s.arr[i] = 16'h1000 + i;
    e0 = s.arr[0]; e1 = s.arr[1]; e2 = s.arr[2]; e3 = s.arr[3];
  end
endmodule
"#;
    let sim = simulate(src, 20).expect("simulate failed");
    assert_eq!(u(&sim, "e0"), 0x1000);
    assert_eq!(u(&sim, "e1"), 0x1001);
    assert_eq!(u(&sim, "e2"), 0x1002);
    assert_eq!(u(&sim, "e3"), 0x1003);
}

/// §7.4.2: every fixed unpacked dimension on a struct member is part of the
/// member's array shape; a later dimension is not a packed bit-select.
#[test]
fn multidimensional_struct_member_keeps_each_dimension() {
    let src = r#"
typedef struct {
  logic [7:0] lanes [0:1][2:3];
} parcel_t;

module top;
  logic [7:0] stimulus = '0;
  parcel_t parcel;
  logic [7:0] result;
  int row = 1;
  int column = 3;

  always_comb parcel.lanes[row][column] = stimulus;
  assign result = parcel.lanes[1][3] ^ 8'hA5;

  initial begin
    #1 stimulus = 8'h11;
    #1 stimulus = 8'h22;
    #1 stimulus = 8'h33;
    #1;
  end
endmodule
"#;
    let sim = simulate(src, 20).expect("simulate failed");
    assert_eq!(u(&sim, "parcel.lanes[1][3]") & 0xFF, 0x33);
    assert_eq!(u(&sim, "result") & 0xFF, 0x96);
}

/// The guard: an ordinary packed-vector bit-write must NOT be diverted into the
/// element path — `v` is a signal of its own, the discriminator the fix uses.
#[test]
fn packed_vector_bit_writes_are_unaffected() {
    let src = r#"
module top;
  logic [7:0] v;
  logic [3:0][7:0] p2;
  int rv, rp;
  initial begin
    v = 8'h00;
    v[3] = 1'b1;
    v[0] = 1'b1;
    rv = v;
    p2 = '0;
    p2[2] = 8'hAB;
    rp = p2[2];
  end
endmodule
"#;
    let sim = simulate(src, 20).expect("simulate failed");
    assert_eq!(u(&sim, "rv"), 0x09, "bit-writes still target bits");
    assert_eq!(u(&sim, "rp"), 0xAB, "packed 2D element write unaffected");
}

/// An array member of a QUEUE ELEMENT — the base is a MemberAccess over an
/// Index, not a plain identifier.
#[test]
fn array_member_of_a_queue_element() {
    let src = r#"
module top;
  typedef struct { logic [15:0] arr [4]; int scalar; } s_t;
  s_t q[$];
  s_t src, got;
  int e0, esc, p0, p3, psc;
  initial begin
    src.arr[0] = 16'h1111; src.arr[3] = 16'h4444; src.scalar = 42;
    q.push_back(src);
    e0 = q[0].arr[0]; esc = q[0].scalar;
    got = q.pop_front();
    p0 = got.arr[0]; p3 = got.arr[3]; psc = got.scalar;
  end
endmodule
"#;
    let sim = simulate(src, 20).expect("simulate failed");
    assert_eq!(u(&sim, "e0"), 0x1111, "q[0].arr[0] reads the element");
    assert_eq!(u(&sim, "esc"), 42, "and the scalar member");
    assert_eq!(u(&sim, "p0"), 0x1111, "pop still carries the array");
    assert_eq!(u(&sim, "p3"), 0x4444);
    assert_eq!(u(&sim, "psc"), 42);
}

/// A hierarchical select on a packed-2D struct member reached through a PORT.
/// The port collapses onto the parent's signal, so the element width is keyed
/// by the parent's name — without following `port_aliases` this read one bit.
#[test]
fn hierarchical_packed_2d_struct_member_select_through_a_port() {
    let src = r#"
typedef struct packed { logic [3:0][15:0] plus_one; logic [3:0][15:0] xor_mask; } res_t;
module inner (output logic [3:0][15:0] bus);
  assign bus = {16'h4444, 16'h3333, 16'h2222, 16'h1111};
endmodule
module mid (output res_t result);
  logic [3:0][15:0] out_bus;
  inner u_i (.bus(out_bus));
  assign result.plus_one = out_bus;
  assign result.xor_mask = out_bus;
endmodule
module top;
  res_t r;
  mid dut (.result(r));
  int loc0, loc3, hier0, hier3, sub0;
  initial begin
    #1;
    loc0 = r.plus_one[0];            hier0 = dut.result.plus_one[0];
    loc3 = r.plus_one[3];            hier3 = dut.result.plus_one[3];
    sub0 = dut.u_i.bus[0];
  end
endmodule
"#;
    let sim = simulate(src, 50).expect("simulate failed");
    assert_eq!(u(&sim, "loc0"), 0x1111, "local select");
    assert_eq!(
        u(&sim, "hier0"),
        0x1111,
        "hierarchical select through a port"
    );
    assert_eq!(u(&sim, "loc3"), 0x4444);
    assert_eq!(u(&sim, "hier3"), 0x4444);
    assert_eq!(
        u(&sim, "sub0"),
        0x1111,
        "and a plain hierarchical 2D element"
    );
}

/// A struct with an unpacked-array member through a PROCEDURAL-LOCAL queue,
/// in both directions (local source, module-level source).
#[test]
fn procedural_local_queue_of_structs_keeps_its_members() {
    let src = r#"
module top;
  typedef struct { logic [15:0] arr [4]; int scalar; } s_t;
  s_t mod_src;
  int f_a, f_s, s_a, s_s, m_a, m_s;
  initial begin
    s_t src1, src2, g;
    s_t lq[$];
    src1.arr[0] = 16'h1111; src1.scalar = 1;
    src2.arr[0] = 16'h2222; src2.scalar = 2;
    mod_src.arr[0] = 16'h3333; mod_src.scalar = 3;
    lq.push_back(src1);
    lq.push_back(src2);
    g = lq.pop_front(); f_a = g.arr[0]; f_s = g.scalar;
    g = lq.pop_front(); s_a = g.arr[0]; s_s = g.scalar;
    lq.push_back(mod_src);
    g = lq.pop_front(); m_a = g.arr[0]; m_s = g.scalar;
  end
endmodule
"#;
    let sim = simulate(src, 20).expect("simulate failed");
    assert_eq!(u(&sim, "f_a"), 0x1111, "first push survives");
    assert_eq!(u(&sim, "f_s"), 1);
    assert_eq!(u(&sim, "s_a"), 0x2222, "and the second is distinct");
    assert_eq!(u(&sim, "s_s"), 2);
    assert_eq!(
        u(&sim, "m_a"),
        0x3333,
        "a module-level struct pushes in too"
    );
    assert_eq!(u(&sim, "m_s"), 3);
}

/// A local queue of plain scalars must be unaffected by registering element
/// types for dimensioned locals.
#[test]
fn local_queues_of_scalars_are_unaffected() {
    let src = r#"
module top;
  int a, b, n;
  initial begin
    int q[$];
    string sq[$];
    string s;
    q.push_back(11); q.push_back(22);
    a = q.pop_front(); b = q.pop_back(); n = q.size();
    sq.push_back("hi");
    s = sq.pop_front();
  end
endmodule
"#;
    let sim = simulate(src, 20).expect("simulate failed");
    assert_eq!(u(&sim, "a"), 11);
    assert_eq!(u(&sim, "b"), 22);
    assert_eq!(u(&sim, "n"), 0, "queue drained");
}
