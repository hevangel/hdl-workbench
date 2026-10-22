// RAL field ACCESS POLICIES (IEEE 1800.2 §18.5.2, uvm_reg_field::XpredictX).
// One 8-bit field for each of the 25 policies UVM defines (NOACCESS is not
// one of them: get_access() derives it from map rights, which
// uvm_ral_map_rights.sv covers). All reset to 8'hA5. After reset() each field's
// mirror is predicted through a write of 8'h0F, a second write of 8'hF0 and a
// read returning 8'h3C, and the mirror is logged after every step:
//   T|<policy>|<after reset>|<after write 1>|<after write 2>|<after read>
// The second write is what separates the write-ONCE policies (W1, WO1) from
// the rest. No bus is involved: predict() drives the mirror directly, so this
// exercises the library's policy table itself.
import uvm_pkg::*;
`include "uvm_macros.svh"

class acc_reg extends uvm_reg;
  `uvm_object_utils(acc_reg)
  uvm_reg_field F;
  string acc = "RW";
  function new(string name = "acc_reg"); super.new(name, 8, UVM_NO_COVERAGE); endfunction
  virtual function void build();
    F = uvm_reg_field::type_id::create("F");
    //           parent size lsb access volatile reset has_reset is_rand ind_acc
    F.configure(this,  8,   0,  acc,   0,       8'hA5, 1,       0,      1);
  endfunction
endclass

class acc_block extends uvm_reg_block;
  `uvm_object_utils(acc_block)
  acc_reg R[$];
  function new(string name = "acc_block"); super.new(name, UVM_NO_COVERAGE); endfunction
  virtual function void build();
    string pol[$] = '{"RO","RW","RC","RS","WC","WS","WRC","WRS","WSRC","WCRS",
                      "W1C","W1S","W1T","W0C","W0S","W0T","W1SRC","W1CRS",
                      "W0SRC","W0CRS","WO","WOC","WOS","W1","WO1"};
    default_map = create_map("map", 0, 1, UVM_LITTLE_ENDIAN);
    foreach (pol[i]) begin
      acc_reg r = acc_reg::type_id::create(pol[i]);
      r.acc = pol[i];
      r.configure(this, null, "");
      r.build();
      default_map.add_reg(r, i, "RW");
      R.push_back(r);
    end
    lock_model();
  endfunction
endclass

class ral_acc_test extends uvm_test;
  `uvm_component_utils(ral_acc_test)
  function new(string name, uvm_component parent); super.new(name, parent); endfunction
  task run_phase(uvm_phase phase);
    acc_block blk = acc_block::type_id::create("blk");
    phase.raise_objection(this);
    blk.build();
    blk.reset();
    foreach (blk.R[i]) begin
      uvm_reg_field f = blk.R[i].F;
      bit [7:0] m0, m1, m2, m3;
      m0 = f.get_mirrored_value();
      void'(f.predict(8'h0F, .kind(UVM_PREDICT_WRITE))); m1 = f.get_mirrored_value();
      void'(f.predict(8'hF0, .kind(UVM_PREDICT_WRITE))); m2 = f.get_mirrored_value();
      void'(f.predict(8'h3C, .kind(UVM_PREDICT_READ)));  m3 = f.get_mirrored_value();
      $display("T|%s|%02h|%02h|%02h|%02h", f.get_access(), m0, m1, m2, m3);
    end
    phase.drop_objection(this);
  endtask
endclass

module top;
  initial run_test("ral_acc_test");
endmodule
