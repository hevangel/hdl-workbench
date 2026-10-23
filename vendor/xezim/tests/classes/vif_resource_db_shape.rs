//! §25.9: a virtual interface stored in and read back from a
//! `uvm_resource_db`-shaped structure (1800.2-2020.3's): a type-parameter
//! property written through `do_write` (which first compares `val == t`),
//! and read back through TWO nested `inout T val` formals — a static
//! wrapper calling a virtual method of an implementation object. Both
//! levels name the formal `val`. Reference-validated: the class property
//! ends up bound, and waiting on its clock sees the first toggle.
//!
//! Two defects lost the binding. An `inout` formal whose actual was null
//! on entry never propagated the binding it ended with. And the vif keys
//! of formals are flat by name, so the caller's `val` formal leaked into
//! the callee's same-named PROPERTY: `val == t` compared equal and
//! `do_write` returned without storing.

use xezim::simulate;

fn outs(sim: &xezim::compiler::Simulator) -> Vec<String> {
    sim.output.iter().map(|o| o.message.clone()).collect()
}

#[test]
fn vif_through_nested_inout_formals_and_type_param_property() {
    let src = r#"
interface adder_if(output bit clk);
endinterface
class rsrc #(type T = int);
  T val;
  function void write(T t); do_write(t); endfunction
  virtual function void do_write(T t);
    if (val == t) return;
    val = t;
  endfunction
  function T read(); return do_read(); endfunction
  virtual function T do_read(); return val; endfunction
endclass
virtual class impl_base #(type T = int);
  pure virtual function bit read_by_name(string s, inout T val);
endclass
class impl #(type T = int) extends impl_base #(T);
  static rsrc #(T) r;
  virtual function bit read_by_name(string s, inout T val);
    val = r.read();
    return 1;
  endfunction
  function void set(string s, T val);
    r = new;
    r.write(val);
  endfunction
endclass
class db #(type T = int);
  static impl #(T) imp;
  static function void set(string s, T val);
    if (imp == null) imp = new;
    imp.set(s, val);
  endfunction
  static function bit read_by_name(string s, inout T val);
    return imp.read_by_name(s, val);
  endfunction
endclass
class env;
  virtual adder_if m_if;
  function void conn();
    bit ok;
    ok = db#(virtual adder_if)::read_by_name("x", m_if);
    $display("ok=%0d null=%0d", ok, m_if == null);
  endfunction
  task run();
    @(m_if.clk);
    $display("t=%0t edge", $time);
  endtask
endclass
module top;
  adder_if dif();
  env e;
  initial begin
    db#(virtual adder_if)::set("env", dif);
    e = new;
    e.conn();
    e.run();
    $finish;
  end
  initial forever #50 dif.clk = ~dif.clk;
endmodule
"#;
    let sim = simulate(src, 1000).expect("simulate failed");
    let out = outs(&sim);
    assert!(out.iter().any(|l| l == "ok=1 null=0"), "{:?}", out);
    assert!(out.iter().any(|l| l == "t=50 edge"), "{:?}", out);
}
