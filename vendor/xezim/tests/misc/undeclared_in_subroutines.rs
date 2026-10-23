//! §13.3/§13.4, §8.6: an identifier that nothing declares is an error inside
//! a task or function body too — module, interface and program subroutines,
//! package subroutines and class methods. No validator walked subroutine
//! bodies, so `task t; undeclared_x = 1; endtask` elaborated and ran
//! silently. Each case below is rejected by the reference simulator at the
//! same line (cross-checked against it), and the legal testbench — every
//! name shape a subroutine may use without declaring it locally — runs to
//! the reference simulator's output.

use std::path::PathBuf;
use std::process::Command;

fn case_dir(name: &str) -> PathBuf {
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
        .join("undeclared_in_subroutines")
        .join(name);
    std::fs::create_dir_all(&dir).expect("mkdir");
    dir
}

/// Run `xezim t.sv` inside a fresh directory; returns (exit code, stdout,
/// stderr).
fn run(name: &str, src: &str) -> (i32, String, String) {
    let dir = case_dir(name);
    std::fs::write(dir.join("t.sv"), src).unwrap();
    let out = Command::new(env!("CARGO_BIN_EXE_xezim"))
        .current_dir(&dir)
        .args(["--no-cache", "t.sv"])
        .output()
        .expect("run xezim");
    (
        out.status.code().unwrap_or(-1),
        String::from_utf8_lossy(&out.stdout).into_owned(),
        String::from_utf8_lossy(&out.stderr).into_owned(),
    )
}

/// `src` must fail to elaborate with the undeclared-identifier diagnostic for
/// `name` at `t.sv:<line>:<col>`.
fn rejects(case: &str, src: &str, name: &str, line_col: &str) {
    let (code, stdout, stderr) = run(case, src);
    let want = format!("t.sv:{}: error: Undeclared identifier '{}'", line_col, name);
    assert_eq!(code, 1, "{case}: accepted:\n{stdout}{stderr}");
    assert!(
        stderr.contains(&want),
        "{case}: expected `{want}`:\n{stdout}{stderr}"
    );
    assert!(
        !stdout.contains("done"),
        "{case}: simulation ran:\n{stdout}"
    );
}

#[test]
fn write_in_module_task() {
    rejects(
        "module_task",
        "module tb;\n  task t;\n    undeclared_x = 1;\n  endtask\n  initial begin\n    t();\n    $display(\"done\");\n  end\nendmodule\n",
        "undeclared_x",
        "3:5",
    );
}

#[test]
fn read_in_module_task() {
    rejects(
        "module_task_read",
        "module tb;\n  int r;\n  task t;\n    r = undeclared_rd + 1;\n  endtask\n  initial begin\n    t();\n    $display(\"done %0d\", r);\n  end\nendmodule\n",
        "undeclared_rd",
        "4:9",
    );
}

#[test]
fn write_in_module_function() {
    rejects(
        "module_function",
        "module tb;\n  function int f(input int a);\n    undeclared_f = a;\n    return a;\n  endfunction\n  initial $display(\"done %0d\", f(3));\nendmodule\n",
        "undeclared_f",
        "3:5",
    );
}

#[test]
fn write_in_sub_instance_task() {
    rejects(
        "sub_instance_task",
        "module sub;\n  task t;\n    undeclared_s = 1;\n  endtask\n  initial t();\nendmodule\nmodule tb;\n  sub u();\n  initial #1 $display(\"done\");\nendmodule\n",
        "undeclared_s",
        "3:5",
    );
}

#[test]
fn write_in_class_method() {
    rejects(
        "class_method",
        "module tb;\n  class C;\n    int a;\n    task t;\n      undeclared_c = 1;\n    endtask\n  endclass\n  initial begin\n    C c;\n    c = new;\n    c.t();\n    $display(\"done\");\n  end\nendmodule\n",
        "undeclared_c",
        "5:7",
    );
}

#[test]
fn write_in_package_task() {
    rejects(
        "package_task",
        "package p;\n  task t;\n    undeclared_p = 1;\n  endtask\nendpackage\nmodule tb;\n  import p::*;\n  initial begin\n    t();\n    $display(\"done\");\n  end\nendmodule\n",
        "undeclared_p",
        "3:5",
    );
}

#[test]
fn write_in_interface_task() {
    rejects(
        "interface_task",
        "interface ifc;\n  logic s;\n  task t;\n    undeclared_if = 1;\n  endtask\nendinterface\nmodule tb;\n  ifc i();\n  initial begin\n    i.t();\n    $display(\"done\");\n  end\nendmodule\n",
        "undeclared_if",
        "4:5",
    );
}

#[test]
fn write_in_program_task() {
    rejects(
        "program_task",
        "program pg;\n  task t;\n    undeclared_pg = 1;\n  endtask\n  initial begin\n    t();\n    $display(\"done\");\n  end\nendprogram\nmodule tb;\n  pg p();\nendmodule\n",
        "undeclared_pg",
        "3:5",
    );
}

/// Every shape a subroutine body may legally name without a local
/// declaration: formals, static and automatic locals, an inline enum's
/// members, `with` iterators, loop variables, class members and `this`,
/// inherited members (including a built-in base class), class and value
/// parameters, package imports and `pkg::name`, `$unit` declarations,
/// interface/modport ports and methods, a clocking block, upward and
/// downward hierarchical references, a function declared later, generate
/// scope names, a legacy implicit-return function, and a bound module.
const LEGAL: &str = r#"`timescale 1ns/1ns
int unit_var = 7;
function automatic int unit_fn(int a); return a + unit_var; endfunction
package pk;
  typedef enum {RED, GREEN, BLUE} color_t;
  int pk_var = 3;
  function automatic int pk_fn(int a); return a * 2; endfunction
  task automatic pk_task(output int r);
    color_t c = GREEN;
    r = pk_var + int'(c);
  endtask
  class PkBase;
    int base_val = 5;
    function int get(); return base_val; endfunction
  endclass
  class PkDerived extends PkBase;
    rand bit [3:0] f;
    constraint c_f { f < 4; }
    function int twice(); c_f.constraint_mode(0); return get() + this.base_val + base_val; endfunction
  endclass
endpackage
interface bus_if(input logic clk);
  logic [7:0] data;
  logic valid;
  clocking cb @(posedge clk);
    output data, valid;
  endclocking
  modport mst(output data, valid, import drive);
  task automatic drive(input logic [7:0] d);
    data = d; valid = 1;
  endtask
endinterface
module leaf(input logic clk, bus_if.mst bp);
  int leaf_cnt;
  int step = 2;
  task automatic bump();
    leaf_cnt = leaf_cnt + later_fn(step);
    top.top_cnt = top.top_cnt + 1;
  endtask
  function automatic int later_fn(int a); return a; endfunction
  task automatic send(input logic [7:0] v);
    bp.drive(v);
  endtask
  always @(posedge clk) bump();
endmodule
module mon(input logic [7:0] d);
  int seen;
  task automatic note(); seen = seen + d; endtask
  always @(d) note();
endmodule
module top;
  import pk::*;
  logic clk = 0;
  int top_cnt;
  bus_if bi(clk);
  leaf u_leaf(.clk(clk), .bp(bi.mst));
  class MyMb extends mailbox #(int);
    function new(); super.new(); endfunction
    task push1(); put(1); endtask
  endclass
  class Local #(type T = int, int W = 4);
    T val;
    static int cnt;
    function int width(); return W + $bits(T); endfunction
    task automatic tick(); cnt++; val = val + 1; endtask
  endclass
  generate if (1) begin : g
    int gen_v = 9;
    function automatic int gfn(int a); return a + gen_v; endfunction
  end endgenerate
  function automatic int down();
    return u_leaf.leaf_cnt + g.gen_v;
  endfunction
  function int legacy;
    input [7:0] a;
    reg [7:0] tmp;
    begin
      tmp = a;
      legacy = tmp + 1;
    end
  endfunction
  task automatic cb_drive();
    bi.cb.data <= 8'h5;
    @(bi.cb);
  endtask
  task automatic counters(output int s);
    static int st_count = 0;
    int q[$] = '{1, 5, 3};
    int r[$];
    enum {IDLE, BUSY} state;
    st_count++;
    if (st_count > 5) state = BUSY;
    r = q.find(x) with (x > 2);
    s = st_count + r.size();
    s = s + q.sum() with (item * 2);
    foreach (q[i]) s += q[i];
    for (int j = 0; j < 2; j++) s += j;
  endtask
  always #5 clk = ~clk;
  initial begin
    int r1, r2;
    color_t col;
    Local #(byte, 2) l;
    PkDerived pd;
    MyMb m;
    col = pk::BLUE;
    l = new;
    pd = new;
    m = new;
    pk_task(r1);
    counters(r2);
    l.tick();
    m.push1();
    cb_drive();
    u_leaf.send(8'h3);
    $display("r1=%0d r2=%0d w=%0d tw=%0d pk=%0d un=%0d $unit=%0d leg=%0d g=%0d col=%s",
             r1, r2, l.width(), pd.twice(), pk::pk_fn(2), unit_fn(1), $unit::unit_var,
             legacy(8'd4), g.gfn(1), col.name());
    #20;
    $display("down=%0d top_cnt=%0d", down(), top_cnt);
    $finish;
  end
endmodule
bind top mon u_mon(.d(bi.data));
"#;

#[test]
fn legal_subroutine_names_still_elaborate() {
    let (code, stdout, stderr) = run("legal", LEGAL);
    assert_eq!(code, 0, "legal testbench rejected:\n{stdout}{stderr}");
    assert!(
        stdout.contains(
            "r1=4 r2=31 w=10 tw=15 pk=4 un=8 $unit=7 leg=5 g=10 col=BLUE\ndown=13 top_cnt=2\n"
        ),
        "wrong output:\n{stdout}{stderr}"
    );
}

/// §8/§23.9: a class member is visible bare only inside its own class
/// hierarchy — a member of an UNRELATED class does not declare the name for
/// another class's method.
#[test]
fn member_of_unrelated_class_in_class_method() {
    rejects(
        "unrelated_class_member",
        "class A;\n  int secret;\nendclass\nclass B;\n  function int get();\n    return secret;\n  endfunction\nendclass\nmodule tb;\n  initial begin\n    B b;\n    b = new;\n    $display(\"done %0d\", b.get());\n  end\nendmodule\n",
        "secret",
        "6:12",
    );
}

/// UVM 1.2 removed the global `factory`; code still naming it compiled
/// because `uvm_default_coreservice_t` has a (local) `factory` member, and
/// `factory.set_inst_override_by_name(...)` then silently did nothing. The
/// reference simulator rejects the name. Reduced shape of that case.
#[test]
fn removed_global_used_as_method_receiver() {
    rejects(
        "removed_global_receiver",
        "class fac;\n  function void set_inst_override_by_name(string a, string b, string c);\n  endfunction\nendclass\nclass coreservice;\n  local fac factory;\nendclass\nclass test;\n  function void build();\n    factory.set_inst_override_by_name(\"a\", \"b\", \"c\");\n  endfunction\nendclass\nmodule tb;\n  initial begin\n    test t;\n    t = new;\n    t.build();\n    $display(\"done\");\n  end\nendmodule\n",
        "factory",
        "10:5",
    );
}

/// Names a class method may use bare: members of its own class and of its
/// ancestors (a base-class enum member, static, protected member and
/// localparam), the enclosing class's static and typedef from a nested
/// class, the enclosing module's variable, and an implemented interface
/// class. Output cross-checked against the reference simulator.
#[test]
fn class_scope_names_still_elaborate() {
    let src = r#"package cp;
  class Base;
    typedef enum {LO, HI} lvl_t;
    static int count;
    protected int prot = 3;
    localparam int DEPTH = 2;
    function int base_f(); return 1; endfunction
  endclass
  class Derived extends Base;
    function int f();
      lvl_t l = HI;
      count++;
      return prot + DEPTH + int'(l) + base_f() + count;
    endfunction
  endclass
  class Outer;
    static int shared = 5;
    typedef int my_int;
    class Inner;
      function int g();
        my_int x = shared;
        return x;
      endfunction
    endclass
  endclass
  interface class Shape;
    pure virtual function int area();
  endclass
  class Sq implements Shape;
    int side = 4;
    virtual function int area(); return side * side; endfunction
  endclass
endpackage
module tb;
  import cp::*;
  int modvar = 11;
  class InMod;
    function int h(); return modvar; endfunction
  endclass
  initial begin
    Derived d;
    Outer::Inner i;
    InMod m;
    Sq s;
    d = new;
    i = new;
    m = new;
    s = new;
    $display("f=%0d g=%0d h=%0d area=%0d", d.f(), i.g(), m.h(), s.area());
  end
endmodule
"#;
    let (code, stdout, stderr) = run("class_scope_legal", src);
    assert_eq!(code, 0, "legal class scopes rejected:\n{stdout}{stderr}");
    assert!(
        stdout.contains("f=8 g=5 h=11 area=16\n"),
        "wrong output:\n{stdout}{stderr}"
    );
}
