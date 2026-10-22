//! §7.10.2, §10.7, §10.10: a value stored into a queue or dynamic array —
//! by `push_back`, `push_front`, `insert`, an indexed write or an
//! unpacked-array concatenation — is converted to the element type: an
//! integral element takes its width, signedness and 2-state value set, a
//! real argument rounds, and string, real, class-handle and unpacked-struct
//! elements are stored as they are. Module, class, static, block-local and
//! subroutine-local collections all behave alike.
//!
//! Expected values come from the reference simulator.

use xezim::simulate;

fn t_lines(src: &str) -> Vec<String> {
    simulate(src, 100)
        .expect("simulate failed")
        .output
        .iter()
        .map(|o| o.message.clone())
        .filter(|m| m.starts_with("T|"))
        .collect()
}

/// Pushes and inserts onto module, block-local, function-local, class and
/// static queues; non-integral queues are untouched.
#[test]
fn push_and_insert_convert_to_the_element_type() {
    let src = r#"
typedef struct packed { bit [3:0] a; bit [3:0] b; } ps_t;
typedef struct { int a; byte b; } us_t;
class K; int v; function new(int x); v = x; endfunction endclass
class A;
  byte q[$];
  bit [3:0] n[$];
  shortint s[$];
  logic [11:0] l[$];
  int unsigned u[$];
  static byte sq[$];
  byte d[];
  byte aa[int];
  ps_t pq[$];
  string strq[$];
  real rq[$];
  K kq[$];
  us_t uq[$];
  function void f();
    byte lq[$];
    q.push_back(300); q.push_front(301); q.insert(1, 302);
    lq.push_back(303); lq.push_front(304); lq.insert(1, 305);
    $display("T|loc %0d %0d %0d", lq[0], lq[1], lq[2]);
    sq.push_back(306);
  endfunction
endclass
module top;
  byte mq[$];
  bit [3:0] mn[$];
  byte mcat[$];
  byte md[];
  byte ma[int];
  initial begin
    A a = new; K k = new(7); us_t us;
    byte lq[$];
    mq.push_back(300); mq.push_front(301); mq.insert(1, 302);
    mn.push_back(5'h13); mn.push_front(-1);
    $display("T|mod %0d %0d %0d %0d %0d", mq[0], mq[1], mq[2], mn[0], mn[1]);
    lq.push_back(300); lq.insert(0, 301);
    $display("T|ini %0d %0d", lq[0], lq[1]);
    a.f();
    $display("T|cls %0d %0d %0d %0d", a.q[0], a.q[1], a.q[2], A::sq[0]);
    a.n.push_back(5'h13); a.s.push_back(70000); a.l.push_back(13'h1fff); a.u.push_back(-1);
    $display("T|w %0d %0d %0d %0d", a.n[0], a.s[0], a.l[0], a.u[0]);
    a.q = {a.q, 307}; mcat = {mcat, 308};
    $display("T|cat %0d %0d %0d", a.q[3], mcat[0], a.q.size());
    a.d = new[2]; a.d[0] = 309; md = new[1]; md[0] = 310;
    a.aa[1] = 311; ma[2] = 312;
    $display("T|da %0d %0d %0d %0d", a.d[0], md[0], a.aa[1], ma[2]);
    a.pq.push_back(9'h1A5);
    $display("T|ps %h %0d", a.pq[0], $bits(a.pq[0]));
    a.strq.push_back("hello"); a.rq.push_back(2.75); a.kq.push_back(k);
    us.a = 1000; us.b = 5; a.uq.push_back(us);
    $display("T|oth %s %0.2f %0d %0d %0d", a.strq[0], a.rq[0], a.kq[0].v, a.uq[0].a, a.uq[0].b);
    a.rq.push_back(3); a.strq.push_front("x");
    $display("T|oth2 %0.2f %s %0d", a.rq[1], a.strq[0], a.strq.size());
  end
endmodule
"#;
    assert_eq!(
        t_lines(src),
        [
            "T|mod 45 46 44 15 3",
            "T|ini 45 44",
            "T|loc 48 49 47",
            "T|cls 45 46 44 50",
            "T|w 3 4464 4095 4294967295",
            "T|cat 51 52 4",
            "T|da 53 54 55 56",
            "T|ps a5 8",
            "T|oth hello 2.75 7 1000 5",
            "T|oth2 3.00 x 2",
        ]
    );
}

/// Indexed writes, 4-state values, enums, signedness and concatenation onto
/// class queue and dynamic-array members.
#[test]
fn indexed_stores_and_concatenation_convert_to_the_element_type() {
    let src = r#"
typedef struct packed { bit [3:0] a; bit [3:0] b; } ps_t;
typedef enum bit [2:0] {E0, E1, E7=7} e_t;
class A;
  byte q[$];
  bit [3:0] n[$];
  int unsigned u[$];
  logic [11:0] l[$];
  ps_t pq[$];
  e_t eq[$];
  byte d[];
  bit [3:0] aa[string];
endclass
module top;
  initial begin
    A a = new; byte lq[$]; byte x;
    a.q.push_back(1); a.q[0] = 300;
    a.n.push_back(1); a.n[0] = 5'h13;
    a.u.push_back(1); a.u[0] = -1;
    a.pq.push_back(0); a.pq[0] = 9'h1A5;
    lq.push_back(1); lq[0] = 300;
    $display("T|ix %0d %0d %0d %h %0d", a.q[0], a.n[0], a.u[0], a.pq[0], lq[0]);
    a.l.push_back(13'h1zzz); a.l.push_back(1'bx);
    a.eq.push_back(E7); a.eq.push_back(e_t'(9));
    a.d = new[1]; a.d[0] = 300; a.d = {a.d, 301};
    a.aa["k"] = 5'h1F;
    x = 8'sh80; a.q.push_back(x); a.u.push_back(x);
    $display("T|m %h %h %0d %0d %0d %0d %0d %0d %0d", a.l[0], a.l[1], a.eq[0], a.eq[1], a.d[0], a.d[1], a.aa["k"], a.q[1], a.u[1]);
    $display("T|neg %0d %0d %0d", a.q[1] < 0, a.u[1] > 0, $bits(a.q[1]));
  end
endmodule
"#;
    assert_eq!(
        t_lines(src),
        [
            "T|ix 44 3 4294967295 a5 44",
            "T|m zzz 00X 7 1 44 45 15 -128 4294967168",
            "T|neg 1 1 8",
        ]
    );
}

/// Queues of a class type parameter (instance and static, per
/// specialization), an inherited queue member, and a real pushed onto an
/// integral queue.
#[test]
fn type_parameter_and_inherited_queues() {
    let src = r#"
typedef struct packed { bit [3:0] a; bit [3:0] b; } ps_t;
typedef enum bit [2:0] {E0, E1, E7=7} e_t;
class K; int v = 9; endclass
class P #(type T = int);
  T q[$];
  static T sq[$];
  function void f(T x); q.push_back(x); endfunction
endclass
class B; byte bq[$]; endclass
class D extends B; endclass
module top;
  initial begin
    P #(bit [3:0]) p4 = new; P #(string) ps = new; P #(K) pk = new; P #(ps_t) pp = new;
    P #(shortint) psh = new; P #(e_t) pe = new; P #(real) pr = new; P #(logic [5:0]) pl = new;
    D d = new; K k = new; byte bq[$]; logic lq[$];
    p4.q.push_back(5'h13); ps.q.push_back("abc"); pk.q.push_back(k); pp.q.push_back(9'h1A5);
    psh.q.push_back(70000); pe.q.push_back(e_t'(9)); pr.q.push_back(2.5); pl.q.push_back(8'hFF);
    psh.f(70001);
    $display("T|tp %0d %s %0d %h %0d %0d %0d %0.2f %0d", p4.q[0], ps.q[0], pk.q[0].v, pp.q[0], psh.q[0], psh.q[1], pe.q[0], pr.q[0], pl.q[0]);
    d.bq.push_back(300); d.bq.insert(0, 301);
    bq.push_back(2.75); lq.push_back(2'b10);
    $display("T|der %0d %0d %0d %0d", d.bq[0], d.bq[1], bq[0], lq[0]);
    P#(byte)::sq.push_back(300);
    $display("T|stp %0d", P#(byte)::sq[0]);
  end
endmodule
"#;
    assert_eq!(
        t_lines(src),
        [
            "T|tp 3 abc 9 a5 4464 4465 1 2.50 63",
            "T|der 45 44 3 0",
            "T|stp 44",
        ]
    );
}

/// Subroutine-local queues and dynamic arrays (method, function, task).
#[test]
fn subroutine_local_collections() {
    let src = r#"
class A;
  function void f(); byte lq[$]; int unsigned uq[$]; lq.push_back(1); lq[0] = 300; uq.push_back(1); uq[0] = -1; lq = {lq, 301};
    $display("T|lix %0d %0d %0d", lq[0], uq[0], lq[1]); endfunction
endclass
module top;
  function automatic void g(); byte lq[$]; lq.push_back(1); lq[0] = 300; lq.insert(0, 302); $display("T|fix %0d %0d", lq[0], lq[1]); endfunction
  task automatic t(); byte d[]; d = new[2]; d[1] = 300; $display("T|tix %0d", d[1]); endtask
  initial begin A a = new; a.f(); g(); t(); end
endmodule
"#;
    assert_eq!(
        t_lines(src),
        ["T|lix 44 4294967295 45", "T|fix 46 44", "T|tix 44"]
    );
}

/// §8.25/§6.12: a queue of a type parameter takes the element type of the
/// object's own specialization: a real, shortreal or string binding stores
/// values unchanged (push_back, push_front, insert, indexed write, a method),
/// a byte binding truncates; the same for a static queue per
/// specialization and through `extends Pq#(real)`.
#[test]
fn type_parameter_queues_follow_the_specialization() {
    let src = r#"
class K; int v = 9; endclass
typedef struct { int a; byte b; } us_t;
class Pq #(type T = int);
  T q[$];
  static T sq[$];
  function void f(T x); q.push_back(x); endfunction
endclass
class D extends Pq#(real); endclass
module top;
  Pq#(real) pq; Pq#(shortreal) ps; Pq#(string) pst; Pq#(byte) pb; Pq#(K) pk; Pq#(us_t) pu; D d; Pq pi;
  initial begin
    K k = new; us_t u;
    pq = new; ps = new; pst = new; pb = new; pk = new; pu = new; d = new; pi = new;
    pq.q.push_back(1.5); pq.q.push_front(2.25); pq.q.insert(1, 3.75); pq.q.push_back(0); pq.q[3] = 4.5; pq.f(5.125);
    $display("T|real %f %f %f %f %f %0d", pq.q[0], pq.q[1], pq.q[2], pq.q[3], pq.q[4], pq.q.size());
    ps.q.push_back(1.5); ps.q.push_front(2.25); ps.q.insert(1, 3.75); ps.q[0] = 6.5;
    $display("T|sreal %f %f %f", ps.q[0], ps.q[1], ps.q[2]);
    pst.q.push_back("abc"); pst.q.push_front("hello world"); pst.q.insert(1, "x"); pst.q[2] = "yz";
    $display("T|str %s %s %s", pst.q[0], pst.q[1], pst.q[2]);
    pb.q.push_back(300); pb.q.push_front(2.75); pb.q.insert(1, 302); pb.q[2] = -129; pb.f(301);
    $display("T|byte %0d %0d %0d %0d", pb.q[0], pb.q[1], pb.q[2], pb.q[3]);
    pk.q.push_back(k); u.a = 100000; u.b = 7; pu.q.push_back(u);
    $display("T|oth %0d %0d %0d", pk.q[0].v, pu.q[0].a, pu.q[0].b);
    Pq#(real)::sq.push_back(1.5); Pq#(real)::sq.push_front(2.25); Pq#(real)::sq.insert(1, 3.75);
    Pq#(byte)::sq.push_back(300);
    $display("T|static %f %f %f %0d", Pq#(real)::sq[0], Pq#(real)::sq[1], Pq#(real)::sq[2], Pq#(byte)::sq[0]);
    d.q.push_back(1.5); d.q.push_front(2.25); d.q.insert(1, 3.75); d.q[0] = 7.5; d.f(8.25);
    $display("T|der %f %f %f %f", d.q[0], d.q[1], d.q[2], d.q[3]);
    pi.q.push_back(1.5); pi.q.push_back(300);
    $display("T|dflt %0d %0d", pi.q[0], pi.q[1]);
  end
endmodule
"#;
    assert_eq!(
        t_lines(src),
        [
            "T|real 2.250000 3.750000 1.500000 4.500000 5.125000 5",
            "T|sreal 6.500000 3.750000 1.500000",
            "T|str hello world x yz",
            "T|byte 3 46 127 45",
            "T|oth 9 100000 7",
            "T|static 2.250000 3.750000 1.500000 44",
            "T|der 7.500000 3.750000 1.500000 8.250000",
            "T|dflt 2 300",
        ]
    );
}

/// §8.25: the specialization of a class variable declared in a package, in
/// a module (with a `realtime` argument) or as a class property.
#[test]
fn specialization_of_package_module_and_property_variables() {
    let src = r#"
package pk;
  class Pq #(type T = int); T q[$]; endclass
  Pq#(realtime) pv;
  Pq#(byte) pb;
endpackage
import pk::*;
class H; Pq#(real) m; function new(); m = new; m.q.push_back(1.5); endfunction endclass
module top;
  Pq#(realtime) pt;
  initial begin
    H h = new; pt = new; pt.q.push_back(2.5); pk::pv = new; pk::pv.q.push_back(3.5); pk::pb = new; pk::pb.q.push_back(300);
    $display("T|ctx %f %f %f %0d", h.m.q[0], pt.q[0], pk::pv.q[0], pk::pb.q[0]);
  end
endmodule
"#;
    assert_eq!(t_lines(src), ["T|ctx 1.500000 2.500000 3.500000 44"]);
}
