//! §25.9: a virtual interface handed back through an `output` formal into a
//! property of ANOTHER object named from inside a method (`get(c.sline)`,
//! the `uvm_config_db::get(this, "", "TX", cfg.vif)` shape) must bind the
//! property. The owner `c` is a property of `this`, which the binding path
//! never looked up, so the handle stayed null and every `@(posedge
//! sline.clk)` returned at once. Cross-checked against the reference
//! simulator.

use xezim::simulate;

#[test]
fn vif_bound_through_output_actual_of_this_property() {
    let src = r#"
interface serial_if;
  logic sdata;
  logic clk;
endinterface
package pk;
  class rsrc #(type T = int);
    T val;
    function void write(T t); val = t; endfunction
    function T read(); return val; endfunction
  endclass
  class cfgdb #(type T = int);
    static rsrc#(T) r;
    static function void set(T v); r = new(); r.write(v); endfunction
    static function bit get(output T value); value = r.read(); return 1; endfunction
  endclass
  class acfg;
    virtual serial_if sline;
  endclass
  typedef class mon;
  class tst;
    acfg c;
    mon m;
    function void build();
      c = new();
      m = new();
      void'(cfgdb#(virtual serial_if)::get(c.sline));
      m.sline = c.sline;
    endfunction
  endclass
  class mon;
    virtual serial_if sline;
    task run_phase();
      repeat(3) @(posedge sline.clk);
      $display("%0t after 3 clks", $time);
    endtask
  endclass
endpackage
module tb;
  import pk::*;
  logic pclk = 0;
  serial_if TX();
  assign TX.clk = pclk;
  always #1 pclk = ~pclk;
  initial begin
    automatic tst t = new();
    automatic mon m;
    cfgdb#(virtual serial_if)::set(TX);
    t.build();
    m = t.m;
    $display("m.sline.clk=%b pclk=%b", m.sline.clk, pclk);
    m.run_phase();
    $finish;
  end
endmodule
"#;
    let sim = simulate(src, 1_000).expect("simulate failed");
    let out: Vec<String> = sim.output.iter().map(|o| o.message.clone()).collect();
    assert_eq!(out, ["m.sline.clk=0 pclk=0", "5 after 3 clks"]);
}
