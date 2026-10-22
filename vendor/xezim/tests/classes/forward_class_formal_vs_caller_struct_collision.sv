`timescale 1ns/1ps
// Pure-SystemVerilog regression for a CLASS-typed formal whose name collides
// with an unrelated same-named STRUCT local in a CALLER scope.
//
// `typedef class RIP;` forward-declares RIP (also registering it as a typedef
// placeholder). A callee method takes a RIP formal named `rw` and writes a
// member (`rw.status = 3`). A caller method declares a LOCAL struct ALSO named
// `rw` (`BUSOP rw;`) and then calls the callee, passing a real RIP object.
//
// `var_typedef_types` is a flat global map keyed by name, so the caller's
// `BUSOP rw` leaves `var_typedef_types["rw"] = "BUSOP"`. When the callee runs,
// `rw.status` was resolved through that stale struct layout: the READ
// (`rw.status == 0`) and the WRITE (`rw.status = 3`) were spliced as struct
// fields over the class handle, so the handle was clobbered and the member
// write never reached the heap. The class handle must ALWAYS be treated as an
// object — never struct-spliced — even when a same-named struct exists in an
// unrelated live caller frame.
//
// This is the RAL predictor scenario: `uvm_reg_predictor`'s local
// `uvm_reg_bus_op rw` vs `uvm_reg::do_predict`'s `uvm_reg_item rw` formal.
typedef class RIP;

typedef struct {
  int status;
} BUSOP;

class RIP;
  int status;
  RIP m_var;
  function new();
    status = 0;
    m_var = null;
  endfunction
  function void do_predict(RIP rw, int kind);
    if (rw.status == 0) rw.status = 3;   // class-handle member write via formal
    this.m_var = rw;                      // keep the handle alive
  endfunction
endclass

class PRED;
  function void write(RIP reg_item);
    BUSOP rw;          // caller-scope struct with the SAME name as the formal
    rw.status = 7;
    reg_item.do_predict(reg_item, 1);
  endfunction
endclass

module top;
  RIP r;
  PRED p;
  initial begin
    r = new();
    p = new();
    p.write(r);
    if (r.status == 3 && r.m_var == r) $display("TAG_PASS");
    else $display("TAG_FAIL r.status=%0d", r.status);
    $finish;
  end
endmodule
