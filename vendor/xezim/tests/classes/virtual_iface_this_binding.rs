//! §25.9 / §25.8 — binding and using a virtual interface through an explicit
//! `this.`, and a PARAMETERIZED virtual interface as a subroutine formal.
//!
//! Two independent gaps, both hit by the standard UVM-style driver idiom
//! `function new(virtual bus_if #(D, A) t); this.vif = t; ... endfunction`:
//!
//! 1. `this.vif = bus;` silently dropped the binding. `try_bind_virtual_iface`
//!    resolved the object of a `MemberAccess` lvalue only when it was an
//!    `Ident`, so an explicit `this` yielded handle 0 and the
//!    `virtual_iface_bindings` entry was never recorded. Every later
//!    `vif.<member>` access — including BARE ones in other methods — then
//!    missed the §25.8 redirect and read/wrote a phantom signal, so the real
//!    interface members stayed x. Only the bare `vif = bus;` spelling worked.
//!
//! 2. A parameterized virtual interface as a FORMAL argument
//!    (`function new(virtual bus_if #(D, A) t)`) failed to PARSE.
//!    `parse_function_ports` handles `virtual <iface>[.modport] <name>` inline
//!    and never consumed a `#(...)` list, though the class-PROPERTY form
//!    already accepted it via `parse_data_type`.

use xezim::simulate;

/// `this.vif = bus` must bind exactly like `vif = bus`, for reads, writes and
/// nonblocking writes, including from a different method than the binder.
const THIS_BINDING: &str = r#"
interface bus_if (input bit clk);
  logic [7:0] addr;
  logic [7:0] data;
endinterface
module tb;
  bit clk;
  always #5 clk = ~clk;
  bus_if i_bare (.clk(clk));
  bus_if i_this (.clk(clk));
  class drv;
    virtual bus_if vif;
    function new(virtual bus_if t, int use_this);
      if (use_this) this.vif = t;
      else          vif = t;
    endfunction
    task go();
      @(posedge vif.clk);
      vif.addr <= 8'hA5;
      this.vif.data <= 8'h5A;   // write through the explicit-this spelling too
    endtask
  endclass
  int bare_addr, bare_data, this_addr, this_data;
  initial begin
    drv d_bare, d_this;
    d_bare = new(i_bare, 0);
    d_this = new(i_this, 1);
    fork d_bare.go(); d_this.go(); join_none
    repeat(3) @(posedge clk);
    bare_addr = i_bare.addr; bare_data = i_bare.data;
    this_addr = i_this.addr; this_data = i_this.data;
  end
endmodule
"#;

/// A PARAMETERIZED virtual interface passed as a constructor formal, bound via
/// `this.`, driven through a polymorphic base handle — the full UVM driver
/// shape. Two specializations coexist so a mis-sized truncation shows up.
const PARAM_VIF_FORMAL: &str = r#"
interface bus_if #(parameter int DW = 8, parameter int AW = 4) (input bit clk);
  logic [AW-1:0] addr;
  logic [DW-1:0] data;
  logic          valid;
endinterface
virtual class base_drv;
  pure virtual task go(bit [63:0] ad, bit [63:0] da);
  pure virtual function int dw();
endclass
class drv #(parameter int D = 32, parameter int A = 4) extends base_drv;
  virtual bus_if #(D, A) vif;
  protected bit [D-1:0] shadow;
  function new(virtual bus_if #(D, A) t);
    this.vif = t;
    this.vif.addr  <= '0;
    this.vif.valid <= 0;
  endfunction
  virtual task go(bit [63:0] ad, bit [63:0] da);
    @(posedge vif.clk);
    vif.addr  <= ad[A-1:0];
    vif.data  <= da[D-1:0];
    shadow     = da[D-1:0];
  endtask
  virtual function int dw(); return D; endfunction
endclass
module tb;
  bit clk;
  always #5 clk = ~clk;
  bus_if #(.DW(32), .AW(4))  ifa (.clk(clk));
  bus_if #(.DW(8),  .AW(16)) ifb (.clk(clk));
  base_drv pool[2];
  int a_addr, a_data, b_addr, b_data, a_dw, b_dw;
  initial begin
    drv #(.D(32), .A(4))  da;
    drv #(.D(8),  .A(16)) db;
    da = new(ifa);
    db = new(ifb);
    pool[0] = da; pool[1] = db;
    repeat(2) @(posedge clk);
    fork pool[0].go(64'hFFFF_FFFF_FFFF_FFF5, 64'hAAAA_BBBB_CCCC_DDDD); join_none
    repeat(2) @(posedge clk);
    a_addr = ifa.addr; a_data = ifa.data;
    fork pool[1].go(64'h0000_0000_0000_1234, 64'h1111_2222_3333_4455); join_none
    repeat(2) @(posedge clk);
    b_addr = ifb.addr; b_data = ifb.data;
    a_dw = pool[0].dw(); b_dw = pool[1].dw();
  end
endmodule
"#;

fn u(sim: &xezim::compiler::Simulator, n: &str) -> u64 {
    sim.get_signal(n)
        .or_else(|| sim.get_signal(&format!("tb.{}", n)))
        .unwrap_or_else(|| panic!("signal not found: {}", n))
        .to_u64()
        .unwrap_or_else(|| panic!("{} not u64-able (x/z?)", n))
}

#[test]
fn this_prefixed_virtual_interface_binding_works() {
    let sim = simulate(THIS_BINDING, 200).expect("simulate failed");
    assert_eq!(
        u(&sim, "bare_addr"),
        0xA5,
        "bare `vif = bus` binding (control)"
    );
    assert_eq!(
        u(&sim, "bare_data"),
        0x5A,
        "bare-bound: this.vif write lands"
    );
    assert_eq!(
        u(&sim, "this_addr"),
        0xA5,
        "`this.vif = bus` must bind identically"
    );
    assert_eq!(
        u(&sim, "this_data"),
        0x5A,
        "this-bound: this.vif write lands"
    );
}

#[test]
fn parameterized_virtual_interface_formal_drives_correctly() {
    let sim = simulate(PARAM_VIF_FORMAL, 200).expect("simulate failed");
    assert_eq!(
        u(&sim, "a_dw"),
        32,
        "specialization reaches the polymorphic call"
    );
    assert_eq!(u(&sim, "b_dw"), 8, "second specialization is distinct");
    assert_eq!(
        u(&sim, "a_addr"),
        0x5,
        "64-bit arg truncated to the 4-bit addr"
    );
    assert_eq!(
        u(&sim, "a_data"),
        0xCCCC_DDDD,
        "truncated to the 32-bit data"
    );
    assert_eq!(
        u(&sim, "b_addr"),
        0x1234,
        "16-bit addr on the other variant"
    );
    assert_eq!(u(&sim, "b_data"), 0x55, "truncated to the 8-bit data");
}
