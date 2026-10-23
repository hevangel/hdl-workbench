//! Issue #280: the built-in count methods are `function int` — SIGNED
//! 32-bit — per IEEE 1800-2023 §7.5.2 (dynamic array `size()`), §7.9.1
//! (associative `num()`/`size()`), §7.10.2.1 (queue `size()`), §6.16.1
//! (string `len()`), §6.19.5.3 (enum `num()`) and §15.4.3 (mailbox
//! `num()`); the §20.6.2 / §20.7 query functions (`$bits`, `$size`,
//! `$unpacked_dimensions`, ...) return `integer`. They read back unsigned, so
//! `q.size() - 2` on an empty queue was 4294967294, `%d` padded to 10
//! columns instead of 11, and `for (i = 0; i < q.size() - 2; i++)` on a
//! one-element queue ran ~4e9 times instead of none.
//!
//! The §11.8.1 rules still apply the other way round: mixing the signed
//! count with an UNSIGNED operand makes the whole expression unsigned. Every
//! expected value below comes from the reference simulator.

use xezim::simulate;

fn t_lines(src: &str) -> Vec<String> {
    simulate(src, 100)
        .expect("simulate")
        .output
        .iter()
        .map(|o| o.message.trim_end().to_string())
        .filter(|l| l.starts_with("T|"))
        .collect()
}

/// The issue's reproducer (the loop is guarded so a regression fails
/// instead of spinning).
#[test]
fn issue_280_reproducer() {
    let got = t_lines(
        r#"
module top;
  int q[$]; int d[]; int a[int];
  initial begin
    int n;
    $display("T|q.size()-2=%0d", q.size() - 2);
    $display("T|d.size()-5<0=%0d", d.size() - 5 < 0);
    $display("T|a.num()-1=%0d", a.num() - 1);
    q.push_back(7);
    $display("T|pad=[%d]", q.size());
    n = 0;
    for (int i = 0; i < q.size() - 2; i++) begin n++; if (n > 5) break; end
    $display("T|loop n=%0d", n);
    $display("T|done");
  end
endmodule
"#,
    );
    assert_eq!(
        got,
        [
            "T|q.size()-2=-2",
            "T|d.size()-5<0=1",
            "T|a.num()-1=-1",
            "T|pad=[          1]",
            "T|loop n=0",
            "T|done",
        ],
    );
}

/// §11.8.1: signed only while every operand is signed; an unsigned operand
/// (or a 64-bit unsigned one) turns the expression unsigned, a ternary with
/// an unsigned arm is unsigned, and `/`, `%`, `>>>` follow the signedness.
#[test]
fn expression_signedness_rules() {
    let got = t_lines(
        r#"
module top;
  int q[$]; int d[]; int a[int];
  int unsigned u; bit [31:0] b32; int r; shortint sh;
  longint L; logic [63:0] w64;
  int unsigned big = 32'hFFFF_FFFF;
  initial begin
    $display("T|A1 %0d", q.size() < -1);
    $display("T|A2 %0d", q.size() - 2 > 5);
    $display("T|A3 %0d", q.size() - 2 > 32'd5);
    $display("T|A4 %0d", q.size() - 2 < u);
    $display("T|A5 %0d", q.size() < big);
    $display("T|A6 %0d", q.size() - 1 < big);
    $display("T|A7 %0d", q.size() - 1 < b32);
    L = q.size() - 2; $display("T|A8 %0d", L);
    w64 = q.size() - 2; $display("T|A9 %h", w64);
    L = q.size() - 32'd2; $display("T|A10 %0d", L);
    $display("T|A11 %0d", -q.size());
    $display("T|A12 %0d", (r == 0) ? q.size() - 1 : 32'd0);
    $display("T|A13 %0d", (r == 0) ? q.size() - 1 : 0);
    $display("T|A14 %h", {q.size()});
    $display("T|A15 %0d", $bits(q.size()));
    $display("T|A16 %0d", (q.size() - 4) >>> 1);
    $display("T|A17 %0d", (q.size() - 4) / 2);
    $display("T|A18 %0d", $unsigned(q.size() - 1));
    $display("T|A19 %0d", (q.size() - 5) % 3);
    $display("T|A20 %0d", q.size() * -3);
    $display("T|A21 %0d", q.size() - 1 < 64'd0);
    $display("T|A22 %0d", q.size() - 1 < 64'sd0);
    sh = q.size() - 1; $display("T|A23 %0d", sh);
    $display("T|A24 %f", q.size() - 1.5);
    $display("T|A25 %0d", q.size() + d.size() - 1);
    $display("T|A26 %0d", q.size() - 1 == -1);
    $display("T|A27 %0d", q.size() - 1 == 32'hFFFF_FFFF);
    $display("T|A28 %0d", q.size() - a.size() - 1);
  end
endmodule
"#,
    );
    assert_eq!(
        got,
        [
            "T|A1 0",
            "T|A2 0",
            "T|A3 1",
            "T|A4 0",
            "T|A5 1",
            "T|A6 0",
            "T|A7 0",
            "T|A8 -2",
            "T|A9 fffffffffffffffe",
            "T|A10 -2",
            "T|A11 0",
            "T|A12 4294967295",
            "T|A13 -1",
            "T|A14 00000000",
            "T|A15 32",
            "T|A16 -2",
            "T|A17 -2",
            "T|A18 4294967295",
            "T|A19 -2",
            "T|A20 0",
            "T|A21 0",
            "T|A22 1",
            "T|A23 -1",
            "T|A24 -1.500000",
            "T|A25 -1",
            "T|A26 1",
            "T|A27 1",
            "T|A28 -1",
        ],
    );
}

/// String `len()`, associative `size()`/`num()`, and collections that are
/// class properties or unpacked-struct members.
#[test]
fn strings_assoc_class_and_struct_members() {
    let got = t_lines(
        r#"
class C;
  int q[$]; int a[string]; int d[];
endclass
typedef struct { int q[$]; int d[]; } S;
module top;
  int a[int]; string s;
  C c; S st;
  initial begin
    c = new;
    s = "ab";
    $display("T|S1 %0d", s.len() - 5);
    $display("T|S2 %0d", s.len() - 5 < 0);
    $display("T|N1 %0d", a.size() - 1);
    $display("T|N2 %0d", a.num() - 1 < 0);
    $display("T|N3 [%d]", a.num());
    $display("T|C1 %0d", c.q.size() - 2);
    $display("T|C2 %0d", c.a.num() - 1);
    $display("T|C3 %0d", c.d.size() - 1 < 0);
    $display("T|C4 %0d", st.q.size() - 2);
    $display("T|C5 %0d", st.d.size() - 1 < 0);
  end
endmodule
"#,
    );
    assert_eq!(
        got,
        [
            "T|S1 -3",
            "T|S2 1",
            "T|N1 -1",
            "T|N2 1",
            "T|N3 [          0]",
            "T|C1 -2",
            "T|C2 -1",
            "T|C3 1",
            "T|C4 -2",
            "T|C5 1",
        ],
    );
}

/// Compiled bodies: class methods and tasks, automatic functions with local
/// collections, an `always` block, plus enum and mailbox `num()`.
#[test]
fn methods_functions_always_enum_and_mailbox() {
    let got = t_lines(
        r#"
typedef enum {RED, GREEN, BLUE} color_e;
class Holder;
  int q[$]; int a[string]; int d[]; string s;
  int unsigned big = 32'hFFFF_FFFF;
  function int f1(); return q.size() - 2; endfunction
  function bit f2(); return q.size() - 1 < 0; endfunction
  function bit f3(); return this.q.size() - 1 < 0; endfunction
  function bit f4(); return a.num() - 1 < 0; endfunction
  function bit f5(); return d.size() - 1 < 0; endfunction
  function bit f6(); return s.len() - 1 < 0; endfunction
  function bit f7(); return q.size() - 1 < big; endfunction
  function int f8();
    int n = 0;
    for (int i = 0; i < q.size() - 2; i++) begin n++; if (n > 3) break; end
    return n;
  endfunction
  function int f9();
    int lq[$]; int la[int];
    return (lq.size() - 1) + (la.num() - 1);
  endfunction
  task t1();
    int x;
    x = q.size() - 5;
    $display("T|H t1 x=%0d lt=%0d pad=[%d]", x, q.size() - 1 < 0, q.size());
  endtask
endclass
module top;
  Holder h;
  color_e c;
  mailbox #(int) mb;
  int q[$];
  logic [7:0] r8;
  int cnt;
  always @(r8) begin
    cnt = 0;
    while (cnt < q.size() - 1) begin cnt++; if (cnt > 3) break; end
    $display("T|ALW cnt=%0d sub=%0d", cnt, q.size() - 3);
  end
  function automatic int ff(input int k);
    int lq[$];
    repeat (k) lq.push_back(1);
    return lq.size() - 3;
  endfunction
  initial begin
    h = new;
    mb = new;
    $display("T|H f1=%0d f2=%0d f3=%0d f4=%0d f5=%0d f6=%0d f7=%0d f8=%0d f9=%0d",
             h.f1(), h.f2(), h.f3(), h.f4(), h.f5(), h.f6(), h.f7(), h.f8(), h.f9());
    h.t1();
    $display("T|E num-4=%0d pad=[%d]", c.num() - 4, c.num());
    $display("T|M num-1=%0d lt=%0d", mb.num() - 1, mb.num() - 1 < 0);
    $display("T|FF %0d %0d", ff(1), ff(5));
    q.push_back(1); q.push_back(2);
    #1 r8 = 1;
    #1 $display("T|done");
  end
endmodule
"#,
    );
    assert_eq!(
        got,
        [
            "T|H f1=-2 f2=1 f3=1 f4=1 f5=1 f6=1 f7=0 f8=0 f9=-2",
            "T|H t1 x=-5 lt=1 pad=[          0]",
            "T|E num-4=-1 pad=[          3]",
            "T|M num-1=-1 lt=1",
            "T|FF -2 2",
            "T|ALW cnt=1 sub=-1",
            "T|done",
        ],
    );
}

/// §20.6.2 / §20.7: the array query functions return `integer`. `$size` of
/// an associative array is its entry count, and `$bits` of a queue is its
/// current size in bits.
#[test]
fn array_query_functions() {
    let got = t_lines(
        r#"
module top;
  int q[$]; int d[]; int a[int]; int fx[4]; int fx2[2][3];
  initial begin
    $display("T|Q1 %0d", $size(q) - 2);
    $display("T|Q2 %0d", $size(fx) - 5);
    $display("T|Q3 %0d", $high(q));
    $display("T|Q4 %0d", $right(q));
    $display("T|Q5 %0d", $left(fx) - 1);
    $display("T|Q6 %0d", $high(fx) - 4);
    $display("T|Q7 %0d", $low(fx) - 1);
    $display("T|Q8 %0d", $increment(fx));
    $display("T|Q9 %0d", $dimensions(fx2) - 3);
    $display("T|Q10 %0d", $unpacked_dimensions(fx2) - 3);
    $display("T|Q11 %0d", $size(fx2, 2) - 4);
    $display("T|Q12 [%d]", $size(fx));
    $display("T|Q13 %0d", $size(d) - 1 < 0);
    $display("T|Q14 %0d", $bits(fx) - 200);
    $display("T|Q15 [%d]", $bits(fx));
    $display("T|Q16 %0d", $increment(q));
    $display("T|Q17 %0d", $high(d) < 0);
    $display("T|Q18 %0d", $size(a) - 1);
    $display("T|Q19 [%d]", $high(fx));
    $display("T|Q20 %0d", $bits(q));
    a[3] = 1; a[9] = 2;
    q.push_back(1); q.push_back(2);
    $display("T|Q21 %0d %0d %0d", $size(a), a.num(), $bits(q));
  end
endmodule
"#,
    );
    assert_eq!(
        got,
        [
            "T|Q1 -2",
            "T|Q2 -1",
            "T|Q3 -1",
            "T|Q4 -1",
            "T|Q5 -1",
            "T|Q6 -1",
            "T|Q7 -1",
            "T|Q8 -1",
            "T|Q9 0",
            "T|Q10 -1",
            "T|Q11 -1",
            "T|Q12 [          4]",
            "T|Q13 1",
            "T|Q14 -72",
            "T|Q15 [        128]",
            "T|Q16 -1",
            "T|Q17 1",
            "T|Q18 -1",
            "T|Q19 [          3]",
            "T|Q20 0",
            "T|Q21 2 2 64",
        ],
    );
}

/// Mixed contexts: widening into 64 bits, casts, `$sformatf`, `case`,
/// `inside`, shifts, `**`, reals and sized signed/unsigned literals.
#[test]
fn widening_casts_and_mixed_operands() {
    let got = t_lines(
        r#"
module top;
  int q[$]; int d[];
  logic [63:0] w; bit [7:0] b8; int unsigned un; longint L; real r;
  string s;
  initial begin
    w = q.size() + 32'hFFFF_FFFF; $display("T|B1 %h", w);
    w = q.size() - 1; $display("T|B2 %h", w);
    w = q.size() - 1 + 64'd0; $display("T|B3 %h", w);
    w = (q.size() - 1) + 64'd0; $display("T|B4 %h", w);
    w = {q.size() - 1}; $display("T|B5 %h", w);
    w = $signed(q.size()) - 1; $display("T|B6 %h", w);
    w = $unsigned(q.size()) - 1; $display("T|B7 %h", w);
    b8 = q.size() - 1; $display("T|B8 %h", b8);
    un = q.size() - 1; $display("T|B9 %0d", un);
    L = q.size() - 64'd1; $display("T|B10 %0d", L);
    r = (q.size() - 1) * 1.0; $display("T|B11 %f", r);
    $display("T|B12 %0d", q.size() - d.size() < 0);
    case (q.size() - 1) -1: $display("T|B13 m1"); default: $display("T|B13 dflt"); endcase
    $display("T|B14 %0d", q.size() - 1 >= 0);
    s = $sformatf("%d|%0d|%h|%b", q.size() - 1, d.size() - 2, q.size() - 1, s.len() - 1);
    $display("T|B15 %s", s);
    $display("T|B16 %0d", (q.size() - 1) >> 28);
    $display("T|B17 %0d", (q.size() - 1) >>> 28);
    $display("T|B18 %0d", q.size() ** 2 - 1);
    $display("T|B19 %0d", (q.size() - 2) ** 2);
    $display("T|B20 %0d", -(d.size() + 1));
    $display("T|B21 %0d", ~q.size());
    $display("T|B22 %0d", q.size() - 1 inside {-1});
    $display("T|B23 %0d", $countones(q.size() - 1));
    $display("T|B24 %0d", int'(q.size()) - 1);
    $display("T|B25 %0d", 64'(q.size() - 1));
    $display("T|B26 %0d", (q.size() - 1) == un);
    $display("T|B27 %0d", (q.size() - 1) < (d.size() - 1));
    $display("T|B28 %0d", q.size() + 1'b1 - 2);
    $display("T|B29 %0d", q.size() + 8'sd1 - 2);
    $display("T|B30 %0d", q.size() + 3'sb111);
  end
endmodule
"#,
    );
    assert_eq!(
        got,
        [
            "T|B1 00000000ffffffff",
            "T|B2 ffffffffffffffff",
            "T|B3 ffffffffffffffff",
            "T|B4 ffffffffffffffff",
            "T|B5 00000000ffffffff",
            "T|B6 ffffffffffffffff",
            "T|B7 ffffffffffffffff",
            "T|B8 ff",
            "T|B9 4294967295",
            "T|B10 -1",
            "T|B11 -1.000000",
            "T|B12 0",
            "T|B13 m1",
            "T|B14 0",
            "T|B15          -1|-2|ffffffff|11111111111111111111111111111111",
            "T|B16 15",
            "T|B17 -1",
            "T|B18 -1",
            "T|B19 4",
            "T|B20 -1",
            "T|B21 -1",
            "T|B22 1",
            "T|B23 32",
            "T|B24 -1",
            "T|B25 -1",
            "T|B26 1",
            "T|B27 0",
            "T|B28 4294967295",
            "T|B29 -1",
            "T|B30 -1",
        ],
    );
}

/// Nested and scoped shapes: an element of an array of queues or of an
/// associative array of dynamic arrays, a queue of queues, a static class
/// queue, an interface queue, and the paren-less `q.size` / `a.num`.
#[test]
fn nested_static_interface_and_paren_less() {
    let got = t_lines(
        r#"
class K;
  static int sq[$];
  int qa[3][$];
  int aq[string][$];
  function int m1(); return qa[1].size() - 1; endfunction
  function int m2(); return aq["x"].size() - 1; endfunction
endclass
interface ifc;
  int iq[$];
endinterface
module top;
  int aq[2][$]; int dq[$][$]; int ad[int][]; int q[$]; int a[int];
  K k;
  ifc i0();
  initial begin
    k = new;
    $display("T|D1 %0d", aq[0].size() - 1);
    $display("T|D2 %0d", ad[5].size() - 1);
    $display("T|D3 %0d", q.size - 1);
    $display("T|D4 %0d", a.num - 1);
    $display("T|D5 %0d", K::sq.size() - 1);
    $display("T|D6 %0d %0d", k.m1(), k.m2());
    $display("T|D7 %0d", i0.iq.size() - 1);
    dq.push_back(q);
    $display("T|D8 %0d", dq[0].size() - 1);
    $display("T|D9 %0d", k.qa[2].size() - 1 < 0);
    $display("T|D10 [%d]", aq[1].size);
  end
endmodule
"#,
    );
    assert_eq!(
        got,
        [
            "T|D1 -1",
            "T|D2 -1",
            "T|D3 -1",
            "T|D4 -1",
            "T|D5 -1",
            "T|D6 -1 -1",
            "T|D7 -1",
            "T|D8 -1",
            "T|D9 1",
            "T|D10 [          0]",
        ],
    );
}
