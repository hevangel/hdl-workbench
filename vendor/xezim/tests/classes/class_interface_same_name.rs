//! §3.13: a class and an interface may share a name — they live in different
//! name spaces. Inside the class's package a bare type argument naming it
//! (`registry#(mon)`) is the class; only `virtual mon` names the interface.
//! The type-parameter branch of the virtual-interface type test took any
//! argument that matched an interface definition for a virtual interface, so
//! `T obj; obj = new(...)` inside the specialization became an alias
//! assignment and left the handle null (the UVM factory's `create` returned
//! null for a component whose class shared its name with a protocol-monitor
//! interface). Expectations cross-checked against the reference simulator.

use xezim::simulate;

fn out(src: &str) -> Vec<String> {
    let sim = simulate(src, 1_000_000).expect("simulate failed");
    sim.output.iter().map(|o| o.message.clone()).collect()
}

#[test]
fn type_param_bound_to_class_named_like_interface() {
    let o = out(r#"
package pk;
  class base;
    string nm;
    function new(string n = ""); nm = n; endfunction
  endclass
  class registry #(type T = base);
    static function T make(string n);
      T obj;
      obj = new(n);
      return obj;
    endfunction
  endclass
  class mon extends base;
    int v;
    function new(string n = "mon"); super.new(n); v = 7; endfunction
  endclass
  function automatic void go();
    mon m;
    m = registry#(mon)::make("m");
    if (m == null) $display("NULL"); else $display("got %s v=%0d", m.nm, m.v);
  endfunction
endpackage

interface mon #(int N = 2) (input logic clk);
  logic [N-1:0] q;
endinterface

module tb;
  logic clk = 0;
  mon #(.N(3)) u_mon(.clk(clk));
  initial pk::go();
endmodule
"#);
    assert_eq!(o, ["got m v=7"]);
}

#[test]
fn factory_style_registry_with_same_named_interface() {
    let o = out(r#"
package rp;
  class base;
    string nm;
    function new(string n = ""); nm = n; endfunction
  endclass
  virtual class obj_w; pure virtual function base create_c(string n); endclass
  class registry #(type T = base, string Tname = "<unknown>") extends obj_w;
    typedef registry #(T, Tname) this_type;
    local static this_type me = get();
    static function this_type get();
      if (me == null) me = new;
      return me;
    endfunction
    virtual function base create_c(string n);
      T obj;
      obj = new(n);
      return obj;
    endfunction
    static function T create(string n);
      base o;
      o = get().create_c(n);
      if (!$cast(create, o)) $display("cast failed");
    endfunction
  endclass
endpackage
package pk;
  import rp::*;
  class mon extends base;
    typedef registry#(mon, "mon") type_id;
    int v;
    extern function new(string n = "mon");
  endclass
  function mon::new(string n = "mon"); super.new(n); v = 7; endfunction
  class agent;
    mon m_mon;
    function void build();
      m_mon = mon::type_id::create("m");
      if (m_mon == null) $display("NULL"); else $display("got %s v=%0d", m_mon.nm, m_mon.v);
    endfunction
  endclass
endpackage
package tp;
  import pk::*;
  function automatic void go(); agent a = new(); a.build(); endfunction
endpackage

interface mon #(int N = 2) (input logic clk);
  logic [N-1:0] q;
  initial $display("interface mon N=%0d", N);
endinterface

module tb;
  import tp::*;
  logic clk = 0;
  mon #(.N(3)) u_mon(.clk(clk));
  initial go();
endmodule
"#);
    assert_eq!(o, ["interface mon N=3", "got m v=7"]);
}
