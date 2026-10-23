//! §19.3 — covergroups declared outside the top module: in a module
//! instance, an interface, each top of a multi-top design, at file scope,
//! in a package and in a class declared inside a module. They used to be
//! parsed and dropped (file scope, package) or never registered (instances,
//! module-scope classes), so `new()` built nothing and every query read 0.
//!
//! Every expected number was cross-checked against the reference
//! simulator. A covergroup declared in a module is a type of each module
//! instance: `get_coverage()` averages the instances in that module
//! instance only.

use xezim::simulate;

fn lines(src: &str) -> Vec<String> {
    let sim = simulate(src, 100_000).expect("simulate failed");
    sim.output
        .iter()
        .map(|o| o.message.trim().to_string())
        .collect()
}

fn assert_line(lines: &[String], want: &str) {
    assert!(
        lines.iter().any(|l| l == want),
        "expected `{want}`, got {lines:?}"
    );
}

/// A covergroup in an instantiated module, sampled on the instance's own
/// clock port, and one sampled explicitly; queried from a `final` block of
/// the instance and hierarchically from the top.
#[test]
fn covergroup_in_module_instance() {
    let l = lines(
        r#"
module unit(input logic clk, input logic [1:0] v);
  covergroup v_cg @(posedge clk);
    cp_v : coverpoint v;
  endgroup
  v_cg cg = new();
  final $display("in unit: %0.2f", cg.get_inst_coverage());
endmodule
module leaf;
  bit [1:0] v;
  covergroup v_cg;
    cp_v : coverpoint v;
  endgroup
  v_cg cg = new();
  initial begin
    v = 1; cg.sample();
    $display("in leaf: %0.2f", cg.get_inst_coverage());
  end
endmodule
module top;
  logic clk = 0;
  logic [1:0] a = 0;
  always #5 clk = ~clk;
  unit u0 (.clk(clk), .v(a));
  leaf l0 ();
  initial begin
    #12 a = 1;
    #10 $finish;
  end
endmodule
"#,
    );
    assert_line(&l, "in leaf: 25.00");
    assert_line(&l, "in unit: 50.00");
}

/// Two instances of one module: each has its own covergroup type, so the
/// type coverage of `u0.cg` covers u0's instance only.
#[test]
fn covergroup_type_per_module_instance() {
    let l = lines(
        r#"
module unit(input logic clk, input logic [1:0] v);
  covergroup v_cg @(posedge clk);
    cp_v : coverpoint v;
  endgroup
  v_cg cg = new();
endmodule
module top;
  logic clk = 0;
  logic [1:0] a = 0, b = 3;
  always #5 clk = ~clk;
  unit u0 (.clk(clk), .v(a));
  unit u1 (.clk(clk), .v(b));
  initial begin
    #12 a = 1;
    #10 $display("u0 %0.2f  u1 %0.2f  type %0.2f", u0.cg.get_inst_coverage(),
                 u1.cg.get_inst_coverage(), u0.cg.get_coverage());
    $finish;
  end
endmodule
"#,
    );
    assert_line(&l, "u0 50.00  u1 25.00  type 50.00");
}

/// A covergroup in an interface, sampled through the interface instance.
#[test]
fn covergroup_in_interface() {
    let l = lines(
        r#"
interface bus_if;
  bit [1:0] v;
  covergroup v_cg;
    cp_v : coverpoint v;
  endgroup
  v_cg cg = new();
endinterface
module top;
  bus_if bi ();
  initial begin
    bi.v = 1; bi.cg.sample();
    $display("intf: %0.2f", bi.cg.get_inst_coverage());
  end
endmodule
"#,
    );
    assert_line(&l, "intf: 25.00");
}

/// Each top of a two-top design gets its covergroup.
#[test]
fn covergroups_in_two_tops() {
    let l = lines(
        r#"
module a_top;
  bit [1:0] v;
  covergroup v_cg; cp_v : coverpoint v; endgroup
  v_cg cg = new();
  initial begin v = 1; cg.sample(); $display("a_top: %0.2f", cg.get_inst_coverage()); end
endmodule
module b_top;
  bit [1:0] w;
  covergroup w_cg; cp_w : coverpoint w; endgroup
  w_cg cg = new();
  initial begin w = 2; cg.sample(); w = 3; cg.sample(); $display("b_top: %0.2f", cg.get_inst_coverage()); end
endmodule
"#,
    );
    assert_line(&l, "a_top: 25.00");
    assert_line(&l, "b_top: 50.00");
}

/// File-scope and package covergroups, created in the top and in a child
/// (the package one also by its scoped name).
#[test]
fn covergroups_at_file_scope_and_in_package() {
    let l = lines(
        r#"
covergroup file_cg with function sample (bit [1:0] v);
  cp_v : coverpoint v;
endgroup
package p;
  covergroup pkg_cg with function sample (bit [1:0] v);
    cp_v : coverpoint v;
  endgroup
endpackage
module unit;
  file_cg c1 = new();
  p::pkg_cg c2 = new();
  initial begin
    c1.sample(1); c2.sample(2);
    $display("sub: file_cg %0.2f pkg_cg %0.2f", c1.get_inst_coverage(), c2.get_inst_coverage());
  end
endmodule
module top;
  import p::*;
  unit u ();
  file_cg c1 = new();
  pkg_cg c2 = new();
  initial begin
    c1.sample(1); c2.sample(2); c2.sample(3);
    $display("top: file_cg %0.2f pkg_cg %0.2f", c1.get_inst_coverage(), c2.get_inst_coverage());
  end
endmodule
"#,
    );
    assert_line(&l, "sub: file_cg 25.00 pkg_cg 25.00");
    assert_line(&l, "top: file_cg 25.00 pkg_cg 50.00");
}

/// A covergroup embedded in a class that is declared inside a module.
#[test]
fn covergroup_in_class_declared_in_module() {
    let l = lines(
        r#"
module tb;
  class watcher;
    bit [1:0] op;
    covergroup op_cg;
      cp_op : coverpoint op;
    endgroup
    function new(); op_cg = new(); endfunction
  endclass
  watcher w;
  initial begin
    w = new();
    w.op = 0; w.op_cg.sample();
    w.op = 1; w.op_cg.sample();
    $display("op_cg %0.2f", w.op_cg.get_inst_coverage());
  end
endmodule
"#,
    );
    assert_line(&l, "op_cg 50.00");
}

/// Bin bounds taken from the instance's module parameters.
#[test]
fn instance_covergroup_bins_use_instance_parameters() {
    let l = lines(
        r#"
module unit #(parameter int N = 3) (input logic clk, input logic [3:0] v);
  covergroup v_cg @(posedge clk);
    cp_v : coverpoint v { bins lo = {[0:N]}; bins hi = {[N+1:15]}; }
  endgroup
  v_cg cg;
  initial cg = new();
endmodule
module top;
  logic clk = 0;
  logic [3:0] a = 2, b = 9;
  always #5 clk = ~clk;
  unit #(.N(1)) u0 (.clk(clk), .v(a));
  unit #(.N(9)) u1 (.clk(clk), .v(b));
  initial begin
    #12 $display("u0 %0.2f  u1 %0.2f", u0.cg.get_inst_coverage(), u1.cg.get_inst_coverage());
    $finish;
  end
endmodule
"#,
    );
    assert_line(&l, "u0 50.00  u1 50.00");
}
