//! §8 / §23.9: a class declared inside a module is a scope of each instance
//! of that module, so its methods name the items of the instance holding the
//! object — variables, parameters, collections, subroutines, events,
//! interface instances — not the top module's same-named ones nor the first
//! instance's (the method bodies are one AST shared by every instance).

use xezim::simulate;

fn t_lines(src: &str) -> Vec<String> {
    simulate(src, 1_000_000)
        .expect("simulate failed")
        .output
        .iter()
        .map(|o| o.message.trim_end().to_string())
        .filter(|m| m.starts_with("T|"))
        .collect()
}

/// Plain writes and `std::randomize` of the instance's dynamic, fixed and
/// scalar variables, two instances, the top declaring the same names.
#[test]
fn module_class_binds_its_instance() {
    let src = r#"// A class declared inside a module reads and writes the enclosing instance's
// variables (§8: lexical scope), including through std::randomize.
module kid #(parameter int B = 0);
  int dq [];
  int fa [3];
  int x;
  class helper;
    function void plain();
      dq = new[2];
      dq[1] = B + 1;
      fa[2] = B + 2;
      x = B + 3;
    endfunction
    function int rnd_dyn();
      return std::randomize(dq) with { dq.size() == 3; foreach (dq[i]) dq[i] == B + 7 * i; };
    endfunction
    function int rnd_fixed();
      return std::randomize(fa) with { foreach (fa[i]) fa[i] == B + 10 + i; };
    endfunction
    function int rnd_scalar();
      return std::randomize(x) with { x == B + 99; };
    endfunction
  endclass
  initial begin
    helper h = new;
    int ok;
    h.plain();
    $display("T|%m plain dsize=%0d d1=%0d f2=%0d x=%0d", dq.size(), dq[1], fa[2], x);
    ok = h.rnd_dyn();
    $display("T|%m rdyn ok=%0d dsize=%0d d2=%0d", ok, dq.size(), dq[2]);
    ok = h.rnd_fixed();
    $display("T|%m rfix ok=%0d f2=%0d", ok, fa[2]);
    ok = h.rnd_scalar();
    $display("T|%m rsc ok=%0d x=%0d", ok, x);
  end
endmodule
module top;
  int dq [];
  int fa [3];
  int x;
  kid #(1000) a();
  kid #(2000) b();
  initial begin
    #1;
    $display("T|top dsize=%0d f2=%0d x=%0d", dq.size(), fa[2], x);
  end
endmodule
"#;
    // Expected values from the reference simulator.
    let expected = [
        "T|top.a plain dsize=2 d1=1001 f2=1002 x=1003",
        "T|top.a rdyn ok=1 dsize=3 d2=1014",
        "T|top.a rfix ok=1 f2=1012",
        "T|top.a rsc ok=1 x=1099",
        "T|top.b plain dsize=2 d1=2001 f2=2002 x=2003",
        "T|top.b rdyn ok=1 dsize=3 d2=2014",
        "T|top.b rfix ok=1 f2=2012",
        "T|top.b rsc ok=1 x=2099",
        "T|top dsize=0 f2=0 x=0",
    ];
    assert_eq!(t_lines(src), expected);
}

/// Collection methods (`size`, `push_back`, `num`, `exists`, `sum`) and
/// element reads through the instance's queue, dynamic and associative arrays.
#[test]
fn module_class_collection_methods() {
    let src = r#"module kid #(parameter int B = 0);
  int dq [];
  int q [$];
  int aa [int];
  class helper;
    function void plain();
      dq = new[2];
      q.push_back(B);
      aa[B] = 1;
      $display("T|in %m dsize=%0d qsize=%0d q0=%0d anum=%0d ex=%0d", dq.size(), q.size(), q[0], aa.num(), aa.exists(B));
      foreach (dq[i]) dq[i] = B + i;
      $display("T|in %m d1=%0d sum=%0d", dq[1], dq.sum());
    endfunction
  endclass
  helper h;
  initial begin
    h = new;
    #(B/1000);
    h.plain();
    $display("T|%m dsize=%0d qsize=%0d anum=%0d d1=%0d", dq.size(), q.size(), aa.num(), dq[1]);
  end
endmodule
module top;
  int dq [];
  int q [$];
  int aa [int];
  kid #(1000) a();
  kid #(2000) b();
  initial begin
    #5;
    $display("T|top dsize=%0d qsize=%0d anum=%0d", dq.size(), q.size(), aa.num());
  end
endmodule
"#;
    // Expected values from the reference simulator.
    let expected = [
        "T|in top.a.helper.plain dsize=2 qsize=1 q0=1000 anum=1 ex=1",
        "T|in top.a.helper.plain d1=1001 sum=2001",
        "T|top.a dsize=2 qsize=1 anum=1 d1=1001",
        "T|in top.b.helper.plain dsize=2 qsize=1 q0=2000 anum=1 ex=1",
        "T|in top.b.helper.plain d1=2001 sum=4001",
        "T|top.b dsize=2 qsize=1 anum=1 d1=2001",
        "T|top dsize=0 qsize=0 anum=0",
    ];
    assert_eq!(t_lines(src), expected);
}

/// A class task waiting on the module's event wakes on its own instance's
/// trigger.
#[test]
fn module_class_waits_on_its_instance_event() {
    let src = r#"module kid #(parameter int B = 0);
  event ev;
  int cnt;
  class helper;
    task waiter();
      @(ev);
      cnt += 100;
    endtask
    function void fire();
      ->ev;
    endfunction
  endclass
  helper h;
  initial begin
    h = new;
    fork h.waiter(); join_none
    #(B/1000);
    h.fire();
    #1 $display("T|%m cnt=%0d", cnt);
  end
endmodule
module top;
  kid #(1000) a();
  kid #(2000) b();
endmodule
"#;
    // Expected values from the reference simulator.
    let expected = ["T|top.a cnt=100", "T|top.b cnt=100"];
    assert_eq!(t_lines(src), expected);
}

/// A module task called from a class method runs (it was dropped).
#[test]
fn module_class_calls_module_task() {
    let src = r#"// A module task / function called from a method of a class declared in the module.
module kid;
  int cnt;
  task mt(int k);
    $display("T|in mt %m k=%0d", k);
    cnt = cnt + k;
  endtask
  function void mf(int k);
    $display("T|in mf %m k=%0d", k);
    cnt = cnt + k;
  endfunction
  class helper;
    task trun();
      mt(1);
      mf(2);
    endtask
  endclass
  helper h;
  initial begin
    h = new;
    #1;
    h.trun();
    $display("T|%m cnt=%0d", cnt);
  end
endmodule
module top;
  kid a();
endmodule
"#;
    // Expected values from the reference simulator.
    let expected = [
        "T|in mt top.a.mt k=1",
        "T|in mf top.a.mf k=2",
        "T|top.a cnt=3",
    ];
    assert_eq!(t_lines(src), expected);
}

/// Sibling audit: module functions and tasks, localparams, parameterized
/// widths, nets, assoc arrays, events, interface instances, upward and
/// `$root` references, two instances with different parameters.
#[test]
fn module_class_reaches_module_scope() {
    let src = r#"// What a class declared inside a module reaches.
interface bus_if;
  logic [7:0] d;
endinterface
module kid #(parameter int B = 0, parameter int W = 4);
  localparam int LP = B * 2;
  int x;
  int fa [3];
  int q [$];
  int aa [string];
  event ev;
  wire [7:0] nw;
  logic [W-1:0] narrow;
  int cnt;
  bus_if bif();
  typedef struct { int a; int b; } pair_t;
  pair_t pr;
  assign nw = B / 100;
  function int mf(int k);
    return B + k;
  endfunction
  task automatic mt(int k);
    cnt += k;
  endtask
  class helper;
    int own;
    static int scount;
    function void run();
      own = mf(5);                 // S1 module function
      mt(B);                       // S2 module task
      x = LP;                      // S3 localparam
      narrow = '1;                 // S4 parameterized width
      q.push_back(nw);             // S5 net read
      aa["k"] = B;                 // S6 assoc string key
      ->ev;                        // S7 event trigger
      bif.d = B / 1000;            // S8 interface instance member
      pr.a = B + 11;               // S9 unpacked struct member
      scount++;                    // S10 static class member (shared)
    endfunction
    function int peek();
      return $bits(narrow);        // S11 $bits of param-sized var
    endfunction
    task waiter();
      @(ev);                       // S12 wait on module event
      cnt += 100;
    endtask
    function int up();
      return top.tv + B;           // S13 upward/absolute reference
    endfunction
    function int rt();
      return $root.top.tv + 1;     // S14 $root reference
    endfunction
  endclass
  helper h;
  initial begin
    h = new;
    fork h.waiter(); join_none
    #(B/1000);
    h.run();
    #1;
    $display("T|%m S1 own=%0d S2 cnt=%0d S3 x=%0d S4 nar=%0d S5 q0=%0d S6 aa=%0d", h.own, cnt, x, narrow, q[0], aa["k"]);
    $display("T|%m S8 bif=%0d S11 bits=%0d S13 up=%0d S14 rt=%0d", bif.d, h.peek(), h.up(), h.rt());
  end
endmodule
module top;
  int tv = 7;
  int x;
  int q [$];
  int cnt;
  kid #(1000, 4) a();
  kid #(2000, 8) b();
  initial begin
    #10;
    $display("T|top x=%0d qn=%0d cnt=%0d", x, q.size(), cnt);
  end
endmodule
"#;
    // Expected values from the reference simulator.
    let expected = [
        "T|top.a S1 own=1005 S2 cnt=1100 S3 x=2000 S4 nar=15 S5 q0=10 S6 aa=1000",
        "T|top.a S8 bif=1 S11 bits=4 S13 up=1007 S14 rt=8",
        "T|top.b S1 own=2005 S2 cnt=2100 S3 x=4000 S4 nar=255 S5 q0=20 S6 aa=2000",
        "T|top.b S8 bif=2 S11 bits=8 S13 up=2007 S14 rt=8",
        "T|top x=0 qn=0 cnt=0",
    ];
    assert_eq!(t_lines(src), expected);
}

/// Each instance's class is its own type, so a static member is
/// per instance.
#[test]
fn module_class_static_member_per_instance() {
    let src = r#"module kid;
  class helper;
    static int scount;
    function void run(); scount++; endfunction
  endclass
  helper h;
  initial begin
    h = new;
    #1 h.run();
    #1 $display("T|%m sc=%0d", helper::scount);
  end
endmodule
module top;
  kid a();
  kid b();
endmodule
"#;
    // Expected values from the reference simulator.
    let expected = ["T|top.a sc=1", "T|top.b sc=1"];
    assert_eq!(t_lines(src), expected);
}

/// A member write of the instance's unpacked struct from a method.
#[test]
fn module_class_writes_instance_struct_member() {
    let src = r#"module kid;
  typedef struct { int a; int b; } pair_t;
  pair_t pr;
  class helper;
    function int rd(); return pr.a; endfunction
    function void wr(); pr.b = 77; endfunction
  endclass
  helper h;
  initial begin
    h = new;
    pr.a = 5;
    $display("T|rd=%0d", h.rd());
    h.wr();
    $display("T|b=%0d", pr.b);
  end
endmodule
module top;
  kid a();
endmodule
"#;
    // Expected values from the reference simulator.
    let expected = ["T|rd=5", "T|b=77"];
    assert_eq!(t_lines(src), expected);
}

/// Siblings: a static with an initializer, a static method, a static read
/// through a handle, and a parameterized class declared in a module — each
/// instance has its own statics.
#[test]
fn module_class_statics_per_instance_siblings() {
    let src = r#"module kid;
  class helper;
    static int scount;
    static int sinit = 5;
    int inst_n;
    function void run(); scount++; sinit++; endfunction
    static function int get(); return scount; endfunction
  endclass
  helper h;
  initial begin
    h = new;
    #1 h.run();
    #1 $display("T|%m sc=%0d si=%0d get=%0d hsc=%0d", helper::scount, helper::sinit, helper::get(), h.scount);
  end
endmodule
module kid2;
  class pc #(int P = 1);
    static int c;
  endclass
  initial begin
    #1 pc#(2)::c++;
    #2 $display("T|%m pc=%0d", pc#(2)::c);
  end
endmodule
module top;
  kid a();
  kid b();
  kid2 x();
  kid2 y();
endmodule
"#;
    // Time-0 order across instances is not specified (§4.7): compare sorted.
    let mut got = t_lines(src);
    got.sort();
    // Expected values from the reference simulator.
    let expected = [
        "T|top.a sc=1 si=6 get=1 hsc=1",
        "T|top.b sc=1 si=6 get=1 hsc=1",
        "T|top.x pc=1",
        "T|top.y pc=1",
    ];
    assert_eq!(got, expected);
}

/// Siblings of the struct member write: a nested struct member, a member of
/// an element of an array of structs, a plain variable, a task, and a whole
/// struct assignment pattern, from two instances.
#[test]
fn module_class_writes_instance_items() {
    let src = r#"module kid;
  typedef struct { int a; int b; } pair_t;
  typedef struct { pair_t p; int c; } nest_t;
  pair_t pr;
  nest_t ns;
  pair_t arr[2];
  int plain;
  class helper;
    function int rd(); return pr.a; endfunction
    function void wr(); pr.b = 77; endfunction
    function void wr2(); ns.p.a = 5; ns.c = 6; endfunction
    function void wr3(); arr[1].b = 9; plain = 4; endfunction
    function void wr4(); pr = '{a:1, b:2}; endfunction
    task twr(); pr.a = 33; endtask
  endclass
  helper h;
  initial begin
    h = new;
    pr.a = 5;
    $display("T|%m rd=%0d", h.rd());
    h.wr();
    $display("T|%m b=%0d", pr.b);
    h.wr2(); h.wr3();
    $display("T|%m ns=%0d %0d arr=%0d plain=%0d", ns.p.a, ns.c, arr[1].b, plain);
    h.twr();
    $display("T|%m twr=%0d", pr.a);
    h.wr4();
    $display("T|%m wr4=%0d %0d", pr.a, pr.b);
  end
endmodule
module top;
  kid a();
  kid b();
endmodule
"#;
    let mut got = t_lines(src);
    got.sort();
    // Expected values from the reference simulator.
    let expected = [
        "T|top.a b=77",
        "T|top.a ns=5 6 arr=9 plain=4",
        "T|top.a rd=5",
        "T|top.a twr=33",
        "T|top.a wr4=1 2",
        "T|top.b b=77",
        "T|top.b ns=5 6 arr=9 plain=4",
        "T|top.b rd=5",
        "T|top.b twr=33",
        "T|top.b wr4=1 2",
    ];
    assert_eq!(got, expected);
}
