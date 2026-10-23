//! IEEE 1800-2023 §6.8 / §6.3.1 / §6.11.3 / §5.10 / §7.4 / §25.9 / §7.3.2:
//! a variable declared in a procedural scope (function, task, class method,
//! named or unnamed block, fork branch, loop body, `always` block) has its
//! full declared type — 2-state-ness, signedness, struct layout, string,
//! virtual interface, tagged union, packed dimensions — exactly like a
//! module-scope variable. Expected values come from the reference simulator.

use xezim::simulate;

fn t_lines(sim: &xezim::compiler::Simulator) -> Vec<String> {
    sim.output
        .iter()
        .map(|o| o.message.trim_end().to_string())
        .filter(|l| l.starts_with("T|"))
        .collect()
}

#[test]
fn blocklocal_2state_vector() {
    const SRC: &str = r#"
module rbs;
  function automatic int f1(); bit [3:0] b; b = 4'b1x0z; return b; endfunction
  function automatic int f2(); byte unsigned u; u = 200; return u; endfunction
  task automatic t1(output int o); int unsigned u; u = -1; o = (u > 0); endtask
  bit [3:0] mb;
  initial begin : named
    bit [3:0] nb;
    int o;
    nb = 4'b1x0z; mb = 4'b1x0z;
    $display("T|r1|func-local bit=%0d byte-unsigned=%0d", f1(), f2());
    t1(o); $display("T|r2|task-local int unsigned > 0 = %0d", o);
    $display("T|r3|named-block bit=%b module bit=%b", nb, mb);
    fork begin bit [3:0] fb; fb = 4'b1x0z; $display("T|r4|fork-local bit=%b", fb); end join
    for (int i = 0; i < 1; i++) begin bit [3:0] lb; lb = 4'b1x0z; $display("T|r5|loop-local bit=%b", lb); end
  end
  always @(mb) begin bit [3:0] ab; ab = 4'b01xz; $display("T|r6|always-local bit=%b", ab); end
endmodule"#;
    let sim = simulate(SRC, 1000).expect("sim");
    assert_eq!(
        t_lines(&sim),
        [
            "T|r1|func-local bit=8 byte-unsigned=200",
            "T|r2|task-local int unsigned > 0 = 1",
            "T|r3|named-block bit=1000 module bit=1000",
            "T|r4|fork-local bit=1000",
            "T|r5|loop-local bit=1000",
            "T|r6|always-local bit=0100",
        ],
    );
}

#[test]
fn blocklocal_init_converts() {
    const SRC: &str = r#"
module rbi;
  bit [3:0] m_b4 = 4'b1x0z;
  byte unsigned m_bu = 200;
  int m_i = 'x;
  initial begin
    bit [3:0] l_b4 = 4'b1x0z;
    byte unsigned l_bu = 200;
    int l_i = 'x;
    automatic bit [3:0] a_b4 = 4'b1x0z;
    automatic byte unsigned a_bu = 200;
    bit [3:0] p_b4;
    byte unsigned p_bu;
    p_b4 = 4'b1x0z; p_bu = 200;
    $display("T|r1|module %b %0d %0d", m_b4, m_bu, m_i);
    $display("T|r2|static-local %b %0d %0d", l_b4, l_bu, l_i);
    $display("T|r3|auto-local %b %0d", a_b4, a_bu);
    $display("T|r4|assigned %b %0d", p_b4, p_bu);
  end
endmodule"#;
    let sim = simulate(SRC, 1000).expect("sim");
    assert_eq!(
        t_lines(&sim),
        [
            "T|r1|module 1000 200 0",
            "T|r2|static-local 1000 200 0",
            "T|r3|auto-local 1000 200",
            "T|r4|assigned 1000 200",
        ],
    );
}

#[test]
fn blocklocal_string_init() {
    const SRC: &str = r#"
module rsr2;
  initial begin
    string a = "abc";
    string r;
    r = {3{a}};
    $display("T|r1|[%s] %0d", r, r.len());
  end
endmodule"#;
    let sim = simulate(SRC, 1000).expect("sim");
    assert_eq!(t_lines(&sim), ["T|r1|[abcabcabc] 9",],);
}

#[test]
fn blocklocal_struct_literal_init() {
    const SRC: &str = r#"
module rsi;
  typedef struct {int a; byte b;} sab_t;
  struct {int a; byte b;} m1 = '{a:5, b:6};
  sab_t m2 = '{a:5, b:6};
  sab_t m3 = '{5, 6};
  initial begin
    struct {int a; byte b;} l1 = '{a:5, b:6};
    sab_t l2 = '{a:5, b:6};
    sab_t l3 = '{5, 6};
    sab_t l4;
    struct {byte b; int a;} l5 = '{b:6, a:5};
    struct {int a; int b;} l6 = '{a:5, b:6};
    l4 = '{a:5, b:6};
    $display("T|r1|%0d %0d %0d", m1.a, m2.a, m3.a);
    $display("T|r2|%0d %0d %0d %0d", l1.a, l2.a, l3.a, l4.a);
    $display("T|r3|%0d %0d %0d", l5.a, l5.b, l6.a);
    $display("T|r4|%p", l2);
  end
endmodule"#;
    let sim = simulate(SRC, 1000).expect("sim");
    assert_eq!(
        t_lines(&sim),
        [
            "T|r1|5 5 5",
            "T|r2|5 5 5 5",
            "T|r3|5 6 5",
            "T|r4|'{a:5, b:6}",
        ],
    );
}

#[test]
fn blocklocal_unsigned_init() {
    const SRC: &str = r#"
module run;
  int unsigned m1 = -1;
  int unsigned m2;
  initial begin
    int unsigned l1 = -1;
    int unsigned l2;
    longint unsigned l3 = -1;
    l2 = -1; m2 = -1;
    $display("T|r1|%0d %0d %0d %0d %0d", m1, m2, l1, l2, l3);
    $display("T|r2|%0d", l1 > 0);
    $display("T|r3|%0d %d", m1, m1);
  end
endmodule"#;
    let sim = simulate(SRC, 1000).expect("sim");
    assert_eq!(
        t_lines(&sim),
        [
            "T|r1|4294967295 4294967295 4294967295 4294967295 18446744073709551615",
            "T|r2|1",
            "T|r3|4294967295 4294967295",
        ],
    );
}

#[test]
fn subroutine_locals_keep_type() {
    const SRC: &str = r#"
module p1;
  typedef struct packed {logic [3:0] hi; bit [3:0] lo;} ps_t;
  typedef enum bit [1:0] {A0, A1, A2} e_t;
  typedef struct {int a; byte b; logic [3:0] c;} us_t;
  function automatic int f_bit(); bit [3:0] b; b = 4'b1x0z; return b; endfunction
  function automatic int f_init(); bit [3:0] b = 4'b1x0z; return b; endfunction
  function automatic int f_un(); byte unsigned u = 200; return u; endfunction
  function automatic int f_neg(); int unsigned u = -1; return u > 0; endfunction
  function int f_static_bit(); bit [7:0] b; b = 8'bxxxx1111; return b; endfunction
  function automatic logic [31:0] f_ps(); ps_t p; p = 8'bx1z01x0z; return p; endfunction
  function automatic int f_us(); us_t s = '{a:5, b:6, c:7}; return s.a + s.b + s.c; endfunction
  function automatic int f_short(); shortint unsigned s = -1; return s; endfunction
  function automatic int f_lu(); longint unsigned l = -1; return l > 0; endfunction
  function automatic int f_int2(); int i; i = 'x; return i; endfunction
  function automatic int f_byte_sx(); byte b = 8'hff; return b; endfunction
  function automatic int f_bvec_signed(); bit signed [7:0] b = 8'hfe; return b; endfunction
  function automatic int f_pmd(); bit [1:0][3:0] m; m = 8'h5a; return m[1]; endfunction
  function automatic int f_pmd_bits(); logic [7:0][3:0] pk; return $bits(pk) * 100 + $size(pk) ; endfunction
  function automatic int f_enum(); e_t e = A2; return e; endfunction
  task automatic t_out(output int o); bit [3:0] b; b = 4'bzzz1; o = b; endtask
  initial begin
    int o;
    $display("T|f1|%0d %0d %0d %0d", f_bit(), f_init(), f_un(), f_neg());
    $display("T|f2|%0d %h %0d %0d", f_static_bit(), f_ps(), f_us(), f_short());
    $display("T|f3|%0d %0d %0d %0d", f_lu(), f_int2(), f_byte_sx(), f_bvec_signed());
    $display("T|f4|%0d %0d %0d", f_pmd(), f_pmd_bits(), f_enum());
    t_out(o); $display("T|t1|%0d", o);
  end
endmodule"#;
    let sim = simulate(SRC, 1000).expect("sim");
    assert_eq!(
        t_lines(&sim),
        [
            "T|f1|8 8 200 1",
            "T|f2|15 000000XX 18 65535",
            "T|f3|1 0 -1 -2",
            "T|f4|5 3208 2",
            "T|t1|1",
        ],
    );
}

#[test]
fn block_locals_keep_type() {
    const SRC: &str = r#"
interface bus_if(input bit clk); logic [7:0] data; endinterface
module lb;
  bit clk;
  bus_if b(clk);
  typedef struct packed {logic [3:0] hi; bit [3:0] lo;} ps_t;
  typedef union tagged { void A; bit [3:0] B; } tu_t;
  typedef struct {int a; byte b;} sab_t;
  typedef enum {E0, E1, E2} e_t;
  logic [3:0] s4 = 4'b10xz;
  logic [3:0] r1;
  initial begin
    b.data = 8'h3c;
    #1;
    begin
      virtual bus_if vc = b;
      tu_t tu = tagged B (4'd9);
      string s = "abc";
      string r;
      e_t e = E2;
      logic [7:0][3:0] pk [2][3];
      bit signed [7:0] sb = 8'hfe;
      shortint unsigned su = -1;
      sab_t st = '{5, 6};
      r = {2{s}};
      $display("T|b1|%h", vc.data);
      if (tu matches tagged B .v) $display("T|b2|B %0d", v); else $display("T|b2|noB");
      $display("T|b3|%s %0d", r, r.len());
      $display("T|b4|%s %0d", e.name(), e);
      $display("T|b6|%0d %0d %0d %0d", $bits(pk), $size(pk,3), $dimensions(pk), $size(pk,4));
      $display("T|b7|%0d %0d %0d %p", sb, su, sb < 0, st);
    end
  end
  initial begin
    #2;
    fork
      begin bit [3:0] fb = 4'b1x0z; byte unsigned fu = 200; $display("T|k1|%b %0d", fb, fu); end
    join
    for (int i = 0; i < 2; i++) begin automatic bit [3:0] lb = 4'bxx11; automatic int unsigned lu = -1; $display("T|k2|%b %0d %0d", lb, lu, lu > 0); lb = 4'bz0z0; $display("T|k3|%b", lb); end
    repeat (1) begin int ri; ri = 'z; $display("T|k4|%0d", ri); end
  end
  always @(posedge clk) begin automatic bit [3:0] ab = 4'bx1x1; automatic byte unsigned au = 255; $display("T|k5|%b %0d", ab, au); end
  always @(posedge clk) begin
    automatic bit [3:0] eb = s4;
    r1 <= eb;
    $display("T|k6|%b", eb);
    eb = 4'b0x1z;
    $display("T|k7|%b", eb);
  end
  initial begin #3 clk = 1; #1 $display("T|k8|%b", r1); end
endmodule"#;
    let sim = simulate(SRC, 1000).expect("sim");
    assert_eq!(
        t_lines(&sim),
        [
            "T|b1|3c",
            "T|b2|B 9",
            "T|b3|abcabc 6",
            "T|b4|E2 2",
            "T|b6|192 8 4 4",
            "T|b7|-2 65535 1 '{a:5, b:6}",
            "T|k1|1000 200",
            "T|k2|0011 4294967295 1",
            "T|k3|0000",
            "T|k2|0011 4294967295 1",
            "T|k3|0000",
            "T|k4|0",
            "T|k5|0101 255",
            "T|k6|1000",
            "T|k7|0010",
            "T|k8|1000",
        ],
    );
}

#[test]
fn method_and_edge_block_locals() {
    const SRC: &str = r#"
module p8;
  class C;
    logic [3:0] src = 4'b1x0z;
    function int m1(); bit [3:0] b; b = src; return b; endfunction
    function int m2(); bit [3:0] b = src; return b; endfunction
    function int m3(); byte unsigned u = 200; int unsigned v = -1; return u + (v > 0); endfunction
    function int m4(); int i; i = src; return i; endfunction
    function int m5(); bit [7:0] b; b = 0; b[3:0] = src; return b; endfunction
    task t1(output int o); bit [3:0] b; b = src; o = b; endtask
  endclass
  logic [3:0] s4 = 4'b10xz;
  bit clk;
  int r1, r2;
  always @(posedge clk) begin
    automatic bit [3:0] ab;
    automatic int ai;
    ab = s4; ai = s4;
    r1 <= ab; r2 <= ai;
  end
  initial begin
    C c = new; int o;
    $display("T|c1|%0d %0d %0d %0d %0d", c.m1(), c.m2(), c.m3(), c.m4(), c.m5());
    c.t1(o); $display("T|c2|%0d", o);
    #1 clk = 1; #1 $display("T|e1|%0d %0d", r1, r2);
  end
endmodule"#;
    let sim = simulate(SRC, 1000).expect("sim");
    assert_eq!(t_lines(&sim), ["T|c1|8 8 201 8 8", "T|c2|8", "T|e1|8 8",],);
}

#[test]
fn local_array_packed_dims() {
    const SRC: &str = r#"
module lp;
  initial begin
    logic [7:0][3:0] pk1 [2];
    logic [7:0][3:0] pk2 [2][3];
    pk1[1] = 32'h12345678;
    pk2[1][2][7] = 4'hc;
    $display("T|a1|%h %h %h", pk1[1], pk1[1][2], pk1[1][3]);
    $display("T|a2|%h %h", pk2[1][2], pk2[1][2][7]);
    $display("T|a3|%0d %0d %0d %0d", $size(pk1,2), $dimensions(pk1), $left(pk1,3), $size(pk2,4));
  end
endmodule"#;
    let sim = simulate(SRC, 1000).expect("sim");
    assert_eq!(
        t_lines(&sim),
        ["T|a1|12345678 6 5", "T|a2|cxxxxxxx c", "T|a3|8 3 3 4",],
    );
}

#[test]
fn locals_reusing_a_name_keep_their_own_type() {
    const SRC: &str = r#"module lr;
  logic [3:0] v;
  class K;
    logic [3:0] d;
    function void set(); d = 4'b1x0z; endfunction
  endclass
  function automatic logic [3:0] f4(); logic [3:0] d; d = 4'b1x0z; return d; endfunction
  function automatic bit [3:0] f2(); bit [3:0] d; d = 4'b1x0z; return d; endfunction
  initial begin
    bit [3:0] d;
    K k = new;
    d = 4'b01xz;
    v = 4'bx1z0;
    $display("T|n1|%b %b", d, v);
    $display("T|n2|%b %b", f4(), f2());
    d = 4'bzz11;
    k.set();
    $display("T|n3|%b %b", d, k.d);
    begin logic [3:0] d; d = 4'b1x1x; $display("T|n4|%b", d); end
  end
endmodule"#;
    let sim = simulate(SRC, 1000).expect("sim");
    assert_eq!(
        t_lines(&sim),
        [
            "T|n1|0100 x1z0",
            "T|n2|1x0z 1000",
            "T|n3|0011 1x0z",
            "T|n4|1x1x",
        ],
    );
}
