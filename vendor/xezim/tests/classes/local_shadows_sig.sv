// Pure-SystemVerilog self-test: a subroutine-local CLASS HANDLE that
// shadows a same-named module signal must keep its own (32-bit) lvalue
// width — IEEE 1800 §6.19 / §23.6, a block-local declaration shadows a
// design-level name.
//
// This is the multi-top regression. The multi-top wrapper (`-s top -s dut`)
// synthesizes `module __xezim_multi_top; top top(); dut dut();` and registers
// a FLAT width-1 instance signal per requested top, so "top" becomes a
// width-1 design signal. UVM's `uvm_root::m_uvm_get_root` then declares a
// function-local `uvm_root top`; if lvalue-width inference gives the module
// signal precedence, `top = new()` resizes the constructed handle to 1 bit
// and truncates it (e.g. handle 4 -> 0). `top != m_inst` then fired
// `UVM/BAD_TOP` and recursed forever — observed in the register-map
// memory-access and concurrent-update regressions.
//
// Here `sig` plays the local; `bit sig;` at module scope is the width-1
// shadowed signal; `junk[3]` forces the next allocation's handle above 1 bit
// so a truncation is observable.
module top;
  bit sig;                     // shadowing module signal, width 1
  class C;
    int id;
    function int get(); return id; endfunction
    function new(int i); id = i; endfunction
  endclass
  C junk[3];                   // handle of the next allocation is > 1 bit
  function automatic int test();
    C sig;                     // local shadows module signal `sig`
    sig = new(5);
    if (sig == null) return 0;
    return sig.get();
  endfunction
  initial begin
    foreach(junk[i]) junk[i] = new(i);
    if (test() == 5)
      $display("TAG_PASS");
    else
      $display("TAG_FAIL got=%0d", test());
  end
endmodule