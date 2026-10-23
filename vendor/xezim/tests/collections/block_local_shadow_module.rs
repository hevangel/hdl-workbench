//! §6.21 / §9.3.4: a variable declared in a block is a new variable that
//! shadows a same-named one of the enclosing scopes. Collection locals
//! (fixed, queue, dynamic, associative) of nested blocks, fork blocks and
//! module tasks / functions were stored under their bare name — the top
//! module's same-named item — and every local of an instance's process was
//! shared by all instances of the module.

use xezim::simulate;

fn t_lines(src: &str) -> Vec<String> {
    simulate(src, 100)
        .expect("simulate failed")
        .output
        .iter()
        .map(|o| o.message.trim_end().to_string())
        .filter(|m| m.starts_with("T|"))
        .collect()
}

/// Plain assignment and `std::randomize` of a block-local array named like a
/// module array, in the top and in an instance.
#[test]
fn block_local_array_shadows_module_array() {
    let src = r#"// A block-local array that shadows a same-named module array: randomizing
// or assigning the local must leave the module array alone.
module kid;
  bit [15:0] arr [4];
  initial begin
    begin
      bit [15:0] arr [2];
      int ok;
      ok = std::randomize(arr) with { foreach (arr[i]) arr[i] == 900 + i; };
      $display("T|%m local-rand ok=%0d l1=%0d", ok, arr[1]);
    end
    $display("T|%m kid arr0=%0d", arr[0]);
  end
endmodule
module top;
  bit [15:0] arr [4];
  bit [15:0] brr [4];
  kid u();
  initial begin
    begin
      bit [15:0] brr [2];
      int ok;
      ok = std::randomize(brr) with { foreach (brr[i]) brr[i] == 700 + i; };
      $display("T|top local-rand ok=%0d l1=%0d", ok, brr[1]);
      brr[0] = 5;
    end
    #1;
    $display("T|top arr0=%0d brr0=%0d brr1=%0d", arr[0], brr[0], brr[1]);
  end
endmodule
"#;
    // Expected values from the reference simulator.
    let expected = [
        "T|top.u local-rand ok=1 l1=901",
        "T|top.u kid arr0=0",
        "T|top local-rand ok=1 l1=701",
        "T|top arr0=0 brr0=0 brr1=0",
    ];
    assert_eq!(t_lines(src), expected);
}

/// Sibling audit: fixed, queue, dynamic, associative, struct, class handle,
/// event, scalar and packed locals; named, nested and fork blocks, a for
/// loop, a static task and an automatic function, `std::randomize`.
#[test]
fn block_local_kinds_shadow_module_items() {
    let src = r#"// Block locals that shadow a module item of the same name.
class obj; int v; function new(int x); v = x; endfunction endclass
module kid;
  int fa [3];
  int q [$];
  initial begin
    begin : blk
      int fa [3];
      int q [$];
      fa[1] = 31; q.push_back(32);
      $display("T|K1 %m local fa1=%0d qn=%0d", fa[1], q.size());
    end
    #2 $display("T|K2 %m module fa1=%0d qn=%0d", fa[1], q.size());
  end
endmodule
module top;
  typedef struct { int a; } st_t;
  int fa [3];
  int q [$];
  int da [];
  int aa [int];
  st_t st;
  obj h;
  event ev;
  int sc;
  bit [7:0] pk;
  int i;
  int nest [2];
  int tl [2];
  int fl [2];
  int fk [2];
  int rr [2];
  kid u();
  task tsk();
    int tl [2];
    tl[0] = 41;
  endtask
  function automatic int fn();
    int fl [2];
    fl[0] = 42;
    return fl[0];
  endfunction
  initial begin
    int r;
    h = new(1);
    begin : named
      int fa [3];
      fa[0] = 1;
    end
    begin
      int q [$];
      q.push_back(5);
    end
    begin
      int da [];
      da = new[4];
      da[0] = 7;
    end
    begin
      int aa [int];
      aa[3] = 9;
    end
    begin
      st_t st;
      st.a = 11;
    end
    begin
      obj h;
      h = new(12);
    end
    begin
      event ev;
      ->ev;
    end
    begin
      int sc;
      sc = 13;
    end
    begin
      bit [7:0] pk;
      pk = 14;
    end
    for (int i = 0; i < 3; i++) begin end
    begin
      int nest [2];
      nest[0] = 15;
      begin
        int nest [2];
        nest[0] = 16;
      end
      $display("T|N1 outer-local nest0=%0d", nest[0]);
    end
    tsk();
    r = fn();
    fork
      begin
        int fk [2];
        fk[1] = 17;
      end
    join
    begin
      int rr [2];
      r = std::randomize(rr) with { foreach (rr[i]) rr[i] == 50 + i; };
    end
    #5;
    $display("T|S1 fixed fa0=%0d", fa[0]);
    $display("T|S2 queue qn=%0d", q.size());
    $display("T|S3 dyn dn=%0d", da.size());
    $display("T|S4 assoc an=%0d", aa.num());
    $display("T|S5 struct a=%0d", st.a);
    $display("T|S6 handle v=%0d", h.v);
    $display("T|S8 scalar sc=%0d", sc);
    $display("T|S9 packed pk=%0d", pk);
    $display("T|S10 forvar i=%0d", i);
    $display("T|S11 nested nest0=%0d", nest[0]);
    $display("T|S12 task-local tl0=%0d", tl[0]);
    $display("T|S13 func-local fl0=%0d", fl[0]);
    $display("T|S14 fork-local fk1=%0d", fk[1]);
    $display("T|S15 rand-local rr1=%0d", rr[1]);
  end
  initial begin
    @(ev);
    $display("T|S7 event fired (should not)");
  end
endmodule
"#;
    // Expected values from the reference simulator.
    let expected = [
        "T|K1 top.u.blk local fa1=31 qn=1",
        "T|N1 outer-local nest0=15",
        "T|K2 top.u module fa1=0 qn=0",
        "T|S1 fixed fa0=0",
        "T|S2 queue qn=0",
        "T|S3 dyn dn=0",
        "T|S4 assoc an=0",
        "T|S5 struct a=0",
        "T|S6 handle v=1",
        "T|S8 scalar sc=0",
        "T|S9 packed pk=0",
        "T|S10 forvar i=0",
        "T|S11 nested nest0=0",
        "T|S12 task-local tl0=0",
        "T|S13 func-local fl0=0",
        "T|S14 fork-local fk1=0",
        "T|S15 rand-local rr1=0",
    ];
    assert_eq!(t_lines(src), expected);
}

/// Process-level locals of two instances of one module.
#[test]
fn instance_process_locals_are_per_instance() {
    let src = r#"// Scalar/handle locals of two instances' processes.
module kid #(parameter int B = 0);
  initial begin
    int v;
    v = B;
    #(B/1000);
    $display("T|%m v=%0d", v);
    begin
      int w;
      w = B + 1;
      #5;
      $display("T|%m w=%0d", w);
    end
  end
endmodule
module top;
  kid #(1000) a();
  kid #(2000) b();
endmodule
"#;
    // Expected values from the reference simulator.
    let expected = [
        "T|top.a v=1000",
        "T|top.b v=2000",
        "T|top.a w=1001",
        "T|top.b w=2001",
    ];
    assert_eq!(t_lines(src), expected);
}

/// A queue local of an instance's block, named like a top queue.
#[test]
fn instance_block_queue_shadows_top_queue() {
    let src = r#"module kid;
  int q [$];
  initial begin
    begin
      int q [$];
      q.push_back(32);
    end
  end
endmodule
module top;
  int q [$];
  kid u();
  initial begin
    #1 $display("T|out qn=%0d", q.size());
  end
endmodule
"#;
    // Expected values from the reference simulator.
    let expected = ["T|out qn=0"];
    assert_eq!(t_lines(src), expected);
}

/// A queue local at the top of an instance's process.
#[test]
fn instance_process_queue_shadows_top_queue() {
    let src = r#"module kid;
  initial begin
    int q [$];
    q.push_back(32);
  end
endmodule
module top;
  int q [$];
  kid u();
  initial begin
    #1 $display("T|out qn=%0d", q.size());
  end
endmodule
"#;
    // Expected values from the reference simulator.
    let expected = ["T|out qn=0"];
    assert_eq!(t_lines(src), expected);
}

/// A renamed local as a `for` loop variable: the step (an assignment
/// expression) names it too (the loop used to run forever).
#[test]
fn renamed_local_as_loop_variable() {
    let src = r#"// A renamed block local used as a `for` loop variable (the step is an
// assignment expression).
module kid;
  initial begin
    begin : blk
      integer i;
      for (i = 0; i < 2; i = i + 1) #1 $display("T|kid loop %0d", i);
    end
    $display("T|kid done");
  end
endmodule
module top;
  integer i;
  kid u();
  initial begin
    integer i;
    for (i = 0; i < 2; i += 1) #1 $display("T|top loop %0d", i);
    $display("T|top done i=%0d", i);
  end
  initial #10 $display("T|module i=%0d", i);
endmodule
"#;
    // Expected values from the reference simulator.
    let expected = [
        "T|kid loop 0",
        "T|top loop 0",
        "T|kid loop 1",
        "T|kid done",
        "T|top loop 1",
        "T|top done i=2",
        "T|module i=x",
    ];
    assert_eq!(t_lines(src), expected);
}

/// A static local of an `always` block keeps its value between
/// activations, per instance (§6.21).
#[test]
fn always_block_local_keeps_its_value() {
    let src = r#"// Locals of an instance's always block / task, two instances.
module kid #(parameter int B = 0);
  logic clk = 0;
  int seen;
  always #1 clk = ~clk;
  always @(posedge clk) begin
    int t;
    t = t + B;
    seen = t;
  end
  task automatic tk(output int o);
    int arr [2];
    arr[0] = B;
    #1;
    o = arr[0];
  endtask
  initial begin
    int r;
    #6;
    tk(r);
    $display("T|%m seen=%0d r=%0d", seen, r);
  end
endmodule
module top;
  kid #(1000) a();
  kid #(2000) b();
endmodule
"#;
    // Expected values from the reference simulator.
    let expected = ["T|top.a seen=3000 r=1000", "T|top.b seen=6000 r=2000"];
    assert_eq!(t_lines(src), expected);
}

/// §6.21/§23.6: static locals of process blocks are created and initialized
/// once, before any process starts: in loop bodies, fork arms, named blocks
/// (read hierarchically, also before the block first runs), always_ff and
/// generate copies, per instance.
#[test]
fn static_block_locals_across_shapes() {
    let src = r#"// Static locals of process blocks: loops, forks, named blocks read
// hierarchically (also before their block first runs), generate copies.
module kid #(parameter int B = 0);
  logic clk = 0, idle = 0;
  int m = 9;
  int seen [2];
  int r1, r2;
  always #1 clk = ~clk;
  always @(posedge idle) begin : nb
    static int u = 5;
    static int v = m + 1;
    u++;
  end
  for (genvar g = 0; g < 2; g++) begin : gl
    always @(posedge clk) begin
      int c;
      c = c + g + 1 + B;
      seen[g] = c;
    end
  end
  always_ff @(posedge clk) begin
    int ff;
    ff = ff + 1;
    r1 <= ff;
  end
  always @(posedge clk) begin
    for (int i = 0; i < 2; i++) begin
      int z;
      z = z + 1;
      r2 = z;
    end
  end
  initial begin
    #1 $display("T|%m nb.u=%0d nb.v=%0d", nb.u, nb.v);
    #5 $display("T|%m seen=%0d,%0d r1=%0d r2=%0d", seen[0], seen[1], r1, r2);
  end
endmodule
module top;
  kid #(10) a();
  kid #(20) b();
  int q;
  initial begin
    for (int i = 0; i < 3; i++) begin
      int x;
      static int y = 10;
      x = x + 1; y = y + 1;
      q = x * 100 + y;
    end
    $display("T|loop q=%0d", q);
    for (int i = 0; i < 2; i++) begin
      int w;
      fork
        begin w = w + 1; q = w; end
      join
    end
    $display("T|fork w=%0d", q);
    repeat (2) begin
      static int sx = 4;
      automatic int ax = 4;
      sx++; ax++;
      q = sx * 100 + ax;
    end
    $display("T|mix q=%0d", q);
    #8 $display("T|top a.nb.u=%0d b.nb.v=%0d", a.nb.u, b.nb.v);
  end
endmodule
"#;
    // Expected values from the reference simulator.
    let expected = [
        "T|loop q=313",
        "T|fork w=2",
        "T|mix q=605",
        "T|top.a nb.u=5 nb.v=10",
        "T|top.b nb.u=5 nb.v=10",
        "T|top.a seen=33,36 r1=3 r2=6",
        "T|top.b seen=63,66 r1=3 r2=6",
        "T|top a.nb.u=5 b.nb.v=10",
    ];
    assert_eq!(t_lines(src), expected);
}

/// §6.21/§13.3/§13.4: the variables of a static task or function keep their
/// values between calls; an automatic one's (an automatic module's
/// included) start afresh.
#[test]
fn static_subroutine_locals_keep_their_values() {
    let src = r#"// Variables of static tasks and functions keep their values between calls;
// those of automatic ones (and of an automatic module's) start afresh.
module helper;
  task tk(output int o);
    int c;
    c = c + 1;
    o = c;
  endtask
  function int fn();
    int c;
    c = c + 3;
    return c;
  endfunction
endmodule
module automatic amod;
  task tk(output int o);
    int c;
    c = c + 1;
    o = c;
  endtask
  initial begin
    int q;
    #1 tk(q); tk(q);
    $display("T|automatic module task c=%0d", q);
  end
endmodule
module top;
  helper h();
  amod am();
  task static tks(output int o);
    int c;
    c = c + 1;
    o = c;
  endtask
  task automatic tka(output int o);
    int c;
    static int s;
    c = c + 1; s = s + 1;
    o = c * 10 + s;
  endtask
  function automatic int fna();
    int c;
    c = c + 3;
    return c;
  endfunction
  function int fns(int x);
    static int acc = 100;
    acc = acc + x;
    return acc;
  endfunction
  function int fnw(int x);
    int t;
    t = x * 2;
    return t;
  endfunction
  initial begin
    int q;
    tks(q); tks(q); $display("T|task static c=%0d", q);
    h.tk(q); h.tk(q); $display("T|default task c=%0d", q);
    q = h.fn(); q = h.fn(); $display("T|default function c=%0d", q);
    tka(q); tka(q); $display("T|automatic task q=%0d", q);
    q = fna(); q = fna(); $display("T|automatic function c=%0d", q);
    q = fns(1); q = fns(2); $display("T|static acc=%0d", q);
    q = fnw(3) + fnw(4); $display("T|written-first t=%0d", q);
  end
endmodule
"#;
    // Expected values from the reference simulator.
    let expected = [
        "T|task static c=2",
        "T|default task c=2",
        "T|default function c=6",
        "T|automatic task q=12",
        "T|automatic function c=3",
        "T|static acc=103",
        "T|written-first t=14",
        "T|automatic module task c=1",
    ];
    assert_eq!(t_lines(src), expected);
}

/// A scalar loop variable / local in a call frame or scalar class property
/// shadows a same-named package or block-local collection across queue
/// `push_back`, direct/concat/slice property assignments, queue concatenation,
/// type casts (`int'(c1.bits)`), equality comparisons (`==`/`!=`), and `inside`
/// set membership.
#[test]
fn scalar_local_and_class_member_shadow_same_named_collection() {
    let src = r#"package p;
  int map[int];
  int fa[2] = '{100, 200};
  class item_map;
    string name;
    function new(string n = "m"); name = n; endfunction
  endclass
  class item_holder;
    protected bit m_maps[item_map];
    bit [6:0] bits;
    int fa;
    int map;
    function new(bit [6:0] b = 0, int f = 0, int m = 0);
      bits = b; fa = f; map = m;
    endfunction
    function void add_map(item_map map);
      m_maps[map] = 1;
    endfunction
    virtual function void get_maps(ref item_map maps[$]);
      foreach (m_maps[map]) begin
        maps.push_back(map);
      end
    endfunction
    function int check_self(int map, int bits);
      int q[$];
      q = {q, map, this.fa};
      return (this.fa == fa) * 1000 + (map == bits) * 100 + q[0] * 10 + q[1];
    endfunction
  endclass
endpackage

module top;
  import p::*;
  initial begin
    item_holder h1 = new(7'h09, 11, 22);
    item_holder h2 = new(7'h05, 33, 44);
    item_map m = new("my_map");
    bit [15:0] vec = 16'h005a;
    int qi[$];
    int cmp_bits, cmp_fa, in1, in2, self_res;
    h2.bits = h1.bits;
    $display("T|direct bits=%0h/%0h", h1.bits, h2.bits);
    h1.add_map(m);
    begin
      bit bits[];
      int fa[2];
      item_map q[$];
      bits = new[3];
      fa = '{7, 8};
      h1.get_maps(q);
      $display("T|queue size=%0d same=%0d name=%s dyn=%0d bits=%0h",
               q.size(), q[0] == m, q[0].name, bits.size(), h2.bits);

      h2.bits = {4'h2, 3'h3};
      $display("T|concat c2.bits=%0h bits.size=%0d", h2.bits, bits.size());
      h2.bits = vec[6:0];
      $display("T|slice  c2.bits=%0h bits.size=%0d", h2.bits, bits.size());

      qi = {qi, int'(h1.bits), h1.fa};
      $display("T|qconcat sz=%0d q0=%0d q1=%0d", qi.size(), qi[0], qi[1]);

      cmp_bits = (h1.bits == h2.bits);
      cmp_fa   = (h1.fa == h2.fa);
      $display("T|cmp bits_eq=%0d fa_eq=%0d", cmp_bits, cmp_fa);

      in1 = (h1.bits inside {7'h09, 7'h01});
      in2 = (7'h09 inside {h1.bits});
      $display("T|inside in1=%0d in2=%0d", in1, in2);

      self_res = h1.check_self(3, 4);
      $display("T|self res=%0d", self_res);
    end
  end
endmodule
"#;
    // Expected values from the reference simulator.
    let expected = [
        "T|direct bits=9/9",
        "T|queue size=1 same=1 name=my_map dyn=3 bits=9",
        "T|concat c2.bits=13 bits.size=3",
        "T|slice  c2.bits=5a bits.size=3",
        "T|qconcat sz=2 q0=9 q1=11",
        "T|cmp bits_eq=0 fa_eq=0",
        "T|inside in1=1 in2=1",
        "T|self res=1041",
    ];
    assert_eq!(t_lines(src), expected);
}
