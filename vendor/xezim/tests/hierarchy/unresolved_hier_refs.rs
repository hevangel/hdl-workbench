//! IEEE 1800-2023 §23.6, §23.8: a hierarchical name that does not resolve is
//! an elaboration error. xezim used to run such a design with no message: a
//! write to a guessed internal name (`dut.mem_data[3] = ...`) did nothing and
//! a read returned x. Verilator 5.038 rejects every negative case below at
//! elaboration ("Can't find definition of 'mem_data' in dotted
//! variable/method: 'dut.mem_data'").
use std::path::PathBuf;
use std::process::Command;

fn dir(name: &str) -> PathBuf {
    let d = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
        .join("unresolved_hier_refs")
        .join(name);
    std::fs::create_dir_all(&d).unwrap();
    d
}

/// Run xezim on `src` (written as `<name>/t.sv`) with `args`; the exit
/// status and the combined output.
fn run(name: &str, src: &str, args: &[&str]) -> (bool, String) {
    let path = dir(name).join("t.sv");
    std::fs::write(&path, src).unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_xezim"))
        .args(args)
        .arg(&path)
        .output()
        .unwrap();
    let mut text = String::from_utf8_lossy(&output.stdout).to_string();
    text.push_str(&String::from_utf8_lossy(&output.stderr));
    (output.status.success(), text)
}

/// A design under `tb` with a `dut` instance holding a memory, an instance
/// array, a for-generate block and an interface instance; `$BODY` is
/// replaced by the module items under test.
const SHELL: &str = r#"
module leaf(input logic clk);
  logic [3:0] r;
endmodule
module dut(input logic clk);
  logic [3:0] ram_data [0:7];
  leaf u_arr [1:0] (.clk(clk));
  for (genvar i = 0; i < 2; i++) begin : g
    logic [3:0] gv;
  end
endmodule
interface bus_if; logic [7:0] data; endinterface
module tb;
  logic clk = 0;
  logic [3:0] x;
  wire [3:0] w;
  bus_if bus();
  dut dut(.clk(clk));
$BODY
  initial #20 $finish;
endmodule
"#;

/// A testbench that injects faults by writing to memory names the design
/// does not have. The run must stop at elaboration, with the
/// file and line, the name, the scope it fails in and the scope it was
/// searched from.
#[test]
fn write_to_a_missing_internal_name_is_an_elaboration_error() {
    let src = SHELL.replace(
        "$BODY",
        "  initial begin\n    #1;\n    dut.mem_data[3] = dut.mem_data[3] ^ 4'h1;\n  end",
    );
    let (ok, text) = run("write", &src, &[]);
    assert!(!ok, "accepted:\n{text}");
    assert!(
        text.contains(
            "t.sv:21:5: error: hierarchical name 'dut.mem_data[3]' does not resolve: \
             'mem_data' is not declared in 'tb.dut' (searched from 'tb'; IEEE 1800-2023 §23.8)"
        ),
        "{text}"
    );
    assert!(
        !text.contains("Simulation finished"),
        "the design ran:\n{text}"
    );
}

/// Reads, writes, continuous assignments, event controls, assertions,
/// system task arguments and uncalled subroutines are all checked, and so
/// is each kind of scope a name can fail in.
#[test]
fn every_unresolved_reference_is_an_error() {
    let cases: &[(&str, &str, &str)] = &[
        (
            "read",
            "  initial x = dut.missing;",
            "'missing' is not declared in 'tb.dut'",
        ),
        (
            "assign",
            "  assign w = dut.missing;",
            "'missing' is not declared in 'tb.dut'",
        ),
        (
            "assign_lhs",
            "  assign dut.missing = 1'b1;",
            "'missing' is not declared in 'tb.dut'",
        ),
        (
            "event",
            "  initial @(posedge dut.noclk) x = 0;",
            "'noclk' is not declared in 'tb.dut'",
        ),
        (
            "assertion",
            "  always #5 clk = ~clk;\n  a: assert property (@(posedge clk) dut.missing == 0);",
            "'missing' is not declared in 'tb.dut'",
        ),
        (
            "systask",
            "  initial $display(\"%b\", dut.missing);",
            "'missing' is not declared in 'tb.dut'",
        ),
        (
            "uncalled_task",
            "  task flip; dut.mem_data[0] = 1; endtask",
            "'mem_data' is not declared in 'tb.dut'",
        ),
        (
            "uncalled_function",
            "  function automatic int f(); return dut.missing; endfunction",
            "'missing' is not declared in 'tb.dut'",
        ),
        (
            "dead_branch",
            "  initial if (0) dut.missing = 1;",
            "'missing' is not declared in 'tb.dut'",
        ),
        (
            "intermediate",
            "  initial x = dut.nosub.r;",
            "'nosub' is not declared in 'tb.dut'",
        ),
        (
            "instance_array",
            "  initial x = dut.u_arr[1].missing;",
            "'missing' is not declared in 'tb.dut.u_arr[1]'",
        ),
        (
            "generate",
            "  initial x = dut.g[0].missing;",
            "'missing' is not declared in 'tb.dut.g[0]'",
        ),
        (
            "interface",
            "  initial bus.nosuch = 1;",
            "'nosuch' is not declared in 'tb.bus'",
        ),
        (
            "absolute",
            "  initial x = tb.dut.missing;",
            "'missing' is not declared in 'tb.dut'",
        ),
        (
            "root",
            "  initial x = $root.tb.dut.missing;",
            "'missing' is not declared in 'tb.dut'",
        ),
        (
            "inner_block_local",
            "  initial begin\n    begin int dut; dut = 1; end\n    dut.missing = 1;\n  end",
            "'missing' is not declared in 'tb.dut'",
        ),
    ];
    for (name, body, want) in cases {
        let (ok, text) = run(name, &SHELL.replace("$BODY", body), &[]);
        assert!(!ok, "{name}: accepted:\n{text}");
        assert!(
            text.contains("does not resolve") && text.contains(want),
            "{name}: want {want:?}:\n{text}"
        );
    }
}

/// Names written in an instance resolve from that instance: downward into
/// its children, and upward by an enclosing module's or instance's name. A
/// package or modport declared elsewhere is not a member of an instance.
#[test]
fn unresolved_references_in_instances_are_errors() {
    let cases: &[(&str, &str, &str)] = &[
        (
            "upward_module",
            r#"
module leaf(input logic clk);
  logic [3:0] r;
  initial #1 r = mid.nosuch;
endmodule
module mid(input logic clk);
  leaf u(.clk(clk));
endmodule
module tb;
  logic clk = 0;
  mid m(.clk(clk));
endmodule
"#,
            "hierarchical name 'mid.nosuch' does not resolve: 'nosuch' is not declared in \
             'tb.m' (searched from 'tb.m.u'",
        ),
        (
            "upward_top",
            r#"
module leaf(input logic clk);
  logic [3:0] r;
  always @(posedge clk) r <= tb.nosuch;
endmodule
module mid(input logic clk);
  leaf u(.clk(clk));
endmodule
module tb;
  logic clk = 0;
  mid m(.clk(clk));
endmodule
"#,
            "'nosuch' is not declared in 'tb' (searched from 'tb.m.u'",
        ),
        (
            "downward",
            r#"
module leaf(input logic clk);
  logic [3:0] r;
endmodule
module mid(input logic clk);
  leaf u(.clk(clk));
  logic [3:0] y;
  assign y = u.nope;
endmodule
module tb;
  logic clk = 0;
  mid m(.clk(clk));
endmodule
"#,
            "'nope' is not declared in 'tb.m.u' (searched from 'tb.m'",
        ),
        (
            "declared_elsewhere",
            r#"
package helper;
  int x;
endpackage
interface bus_if;
  logic d;
  modport mp(input d);
endinterface
module leaf;
  logic q;
endmodule
module tb;
  leaf dut();
  bus_if bus();
  initial begin
    dut.helper = 1;
    dut.mp = 1;
  end
endmodule
"#,
            "'helper' is not declared in 'tb.dut'",
        ),
    ];
    for (name, src, want) in cases {
        let (ok, text) = run(name, src, &[]);
        assert!(!ok, "{name}: accepted:\n{text}");
        assert!(text.contains(want), "{name}: want {want:?}:\n{text}");
    }
}

/// `--no-strict` keeps the old leniency: the name is not checked.
#[test]
fn no_strict_accepts_an_unresolved_reference() {
    let src = SHELL.replace(
        "$BODY",
        "  initial begin #1 dut.mem_data[3] = 4'h1; $display(\"T|ran\"); end",
    );
    let (ok, text) = run("lenient", &src, &["--no-strict"]);
    assert!(ok, "rejected:\n{text}");
    assert!(text.contains("T|ran"), "{text}");
}

/// Every legal kind of dotted name still elaborates: class handles, struct
/// members (of a block-local variable that hides an instance, in its own
/// initializer too), interface
/// and modport members, instance arrays, generate blocks
/// (named, implicit and a branch a constant rules out), `$root`, absolute
/// and upward names, subroutines, events, parameters, `defparam`, `bind`,
/// named blocks, package and class scopes, and scope arguments.
#[test]
fn legal_hierarchical_references_still_elaborate() {
    let src = r#"
interface bus_if(input logic clk);
  logic [7:0] data;
  modport mp(input data);
endinterface
package pkg;
  int pv = 5;
endpackage
module chk(input logic a);
  logic seen_a;
  always @* seen_a = a;
endmodule
module leaf #(parameter W = 4, parameter USE = 0) (input logic clk);
  logic [W-1:0] r;
  logic [3:0] mem [0:3];
  typedef struct packed { logic [3:0] a; logic [3:0] b; } s_t;
  s_t s;
  event ev;
  function automatic int f(int x); return x + W; endfunction
  task automatic t(output int y); y = 7; endtask
  if (USE) begin : gopt
    leaf2 u_opt();
  end
  // Elaborated only where USE is set, as `gopt` is: in the other
  // instances neither the reference nor its target exists.
  if (USE) begin : gref
    wire [3:0] t = gopt.u_opt.q;
  end
  if (1) begin
    logic anon = 1'b1;
  end
  initial begin : named
    int v;
    v = 3;
  end
endmodule
module leaf2;
  logic [3:0] q = 4'h6;
endmodule
module mid(input logic clk, bus_if.mp p);
  leaf #(4, 1) u_leaf(.clk(clk));
  leaf #(2) u_arr [1:0] (.clk(clk));
  for (genvar i = 0; i < 2; i++) begin : g
    logic [3:0] gv;
    leaf u_g(.clk(clk));
  end
  logic [7:0] seen;
  always @(posedge clk) seen <= p.data;
  logic up1, up2, local_m;
  always @(posedge clk) begin up1 <= tb.flag; up2 <= dut.local_m; end
  task automatic ping(output int k); k = tb.helper(2); endtask
endmodule
class C;
  int field;
  C inner;
  function int get(); return field; endfunction
endclass
module tb;
  logic clk = 0;
  logic flag = 1;
  bus_if bus(clk);
  mid dut(.clk(clk), .p(bus));
  defparam dut.u_arr[0].W = 2;
  bind leaf2 chk u_chk(.a(q[1]));
  C obj;
  int y, z, k;
  typedef struct { int a; int b; } us_t;
  us_t us;
  function automatic int helper(int v); return v * 3; endfunction
  always #5 clk = ~clk;
  wire [3:0] w1 = dut.u_leaf.r;
  assign dut.g[0].gv = 4'h3;
  a: assert property (@(posedge clk) dut.u_leaf.r !== 4'hf);
  initial begin : shadow
    us_t dut;  // hides the instance `dut` in this block: `dut.a` is a member
    dut.a = 5;
    if (dut.a != 5) $display("FAIL block-local shadow");
  end
  initial begin : self_init
    automatic us_t dut = '{1, dut.a};  // its own initializer sees it
    if (dut.a != 1) $display("FAIL self initializer");
  end
  initial begin
    $dumpvars(0, tb.dut);
    obj = new; obj.inner = new;
    obj.inner.field = 4;
    us.a = 1; us.b = 2;
    bus.data = 8'h5a;
    #1;
    dut.u_leaf.mem[2] = 4'h9;
    dut.u_leaf.s.a = 4'h1;
    dut.u_arr[1].r = 2'b10;
    dut.g[1].u_g.r = 4'h7;
    $root.tb.dut.u_leaf.r = 4'h2;
    y = dut.u_leaf.f(1);
    dut.u_leaf.t(z);
    dut.ping(k);
    -> dut.u_leaf.ev;
    #1;
    if (w1 !== 4'h2) $display("FAIL w1=%h", w1);
    if (dut.u_leaf.mem[2] !== 4'h9) $display("FAIL mem");
    if (obj.inner.get() !== 4) $display("FAIL obj");
    if (y != 5 || z != 7 || k != 6) $display("FAIL f/t/ping y=%0d z=%0d k=%0d", y, z, k);
    if (pkg::pv != 5) $display("FAIL pkg");
    if (dut.g[0].gv !== 4'h3) $display("FAIL gen");
    if (us.a + us.b != 3) $display("FAIL struct");
    if (tb.dut.u_arr[1].r !== 2'b10) $display("FAIL arr");
    if (dut.u_leaf.gref.t !== 4'h6) $display("FAIL gref");
    if (dut.u_leaf.gopt.u_opt.u_chk.seen_a !== 1'b1) $display("FAIL bind");
    if (dut.u_leaf.named.v !== 3) $display("FAIL named block");
    if (dut.u_leaf.genblk3.anon !== 1'b1) $display("FAIL implicit generate block");
    if (dut.u_leaf.W != 4 || dut.u_arr[0].W != 2) $display("FAIL parameters");
    #20;
    if (dut.seen !== 8'h5a) $display("FAIL modport seen=%h", dut.seen);
    $display("T|PASS");
    $finish;
  end
endmodule
"#;
    let (ok, text) = run("legal", src, &[]);
    assert!(ok, "rejected:\n{text}");
    assert!(text.contains("T|PASS"), "{text}");
    assert!(!text.contains("FAIL"), "{text}");
}

/// §23.8 / §27.6: scope placeholders do not hide missing declarations.
#[test]
fn missing_generate_members_are_checked() {
    for (index, declaration, target) in [
        (0, "if (1) begin : g logic a; end", "g.missing"),
        (
            1,
            "for(genvar i=0;i<2;i++) begin : g logic a; end",
            "g[1].missing",
        ),
        (
            2,
            "if (1) begin : g if(1) begin : nested logic a; end end",
            "g.nested.missing",
        ),
        (3, "case(1) 1: begin : g logic a; end endcase", "g.missing"),
        (4, "if (1) begin : g end", "g.missing"),
    ] {
        let source = format!("module top; {declaration} initial {target} = 1; endmodule");
        let (ok, text) = run(&format!("gen_missing_{index}"), &source, &["--no-cache"]);
        assert!(
            !ok && text.contains("missing") && text.contains("does not resolve"),
            "{text}"
        );
    }
}

/// §23.8: a farther scope cannot rescue a missing member of a nearer match.
#[test]
fn nearest_hierarchical_head_is_not_replaced_by_an_outer_match() {
    for (index, statement) in [
        "initial child.valid = 1;",
        "initial $display(child.valid);",
        "wire x = child.valid;",
        "always @(child.valid) ;",
        "function int f(); return child.valid; endfunction",
    ]
    .iter()
    .enumerate()
    {
        let source = format!(
            "module good; logic valid; endmodule module bad; logic other; endmodule module outer; bad child(); {statement} endmodule module top; good child(); outer u(); endmodule"
        );
        let (ok, text) = run(&format!("nearest_{index}"), &source, &["--no-cache"]);
        assert!(
            !ok && text.contains("valid") && text.contains("does not resolve"),
            "{text}"
        );
    }
}
