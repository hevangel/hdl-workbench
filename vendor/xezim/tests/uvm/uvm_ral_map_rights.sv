// Map RIGHTS rewrite a field's effective access (uvm_reg_field::get_access).
// A register added to a map with rights "RO" or "WO" restricts its fields:
//   rights RO: write-ish policies read as RO, RC or RS; write-only ones as
//              NOACCESS
//   rights WO: the rule CHANGED between versions. UVM 1.2 keeps RW and WO
//              (as WO) and makes everything else NOACCESS. 1800.2-2020 keeps
//              the write-side effect and strips only the read side (W1CRS
//              becomes W1C, WSRC becomes WS, W1C stays W1C, ...), making only
//              the read-only RO/RC/RS NOACCESS.
//   rights RW: no change
// A NOACCESS field must then ignore predicted writes AND reads, keeping its
// mirror at the reset value 8'hA5.
//   T|<rights>|<declared policy>|<effective policy>|<mirror after W,W,R>
import uvm_pkg::*;
`include "uvm_macros.svh"

class one_reg extends uvm_reg;
  `uvm_object_utils(one_reg)
  uvm_reg_field F;
  string acc = "RW";
  function new(string name = "one_reg"); super.new(name, 8, UVM_NO_COVERAGE); endfunction
  virtual function void build();
    F = uvm_reg_field::type_id::create("F");
    F.configure(this, 8, 0, acc, 0, 8'hA5, 1, 0, 1);
  endfunction
endclass

class rights_block extends uvm_reg_block;
  `uvm_object_utils(rights_block)
  one_reg R[$];
  string rights[$];
  function new(string name = "rights_block"); super.new(name, UVM_NO_COVERAGE); endfunction
  function void add(string rgt, string pol);
    one_reg r = one_reg::type_id::create($sformatf("%s_%s", rgt, pol));
    r.acc = pol;
    r.configure(this, null, "");
    r.build();
    default_map.add_reg(r, R.size(), rgt);
    R.push_back(r);
    rights.push_back(rgt);
  endfunction
  virtual function void build();
    default_map = create_map("map", 0, 1, UVM_LITTLE_ENDIAN);
    add("RO", "RW");   add("RO", "RO");   add("RO", "W1C");  add("RO", "W1");
    add("RO", "RC");   add("RO", "WRC");  add("RO", "WSRC");
    add("RO", "RS");   add("RO", "WRS");  add("RO", "WCRS");
    add("RO", "WO");   add("RO", "WOC");  add("RO", "WO1");
    add("WO", "RW");   add("WO", "WO");   add("WO", "RO");   add("WO", "W1C");
    add("WO", "RC");   add("WO", "RS");   add("WO", "WRC");  add("WO", "WRS");
    add("WO", "W1SRC");add("WO", "W1CRS");add("WO", "WCRS"); add("WO", "WSRC");
    add("WO", "W1");   add("WO", "WO1");
    add("RW", "W1C");
    lock_model();
  endfunction
endclass

class ral_rights_test extends uvm_test;
  `uvm_component_utils(ral_rights_test)
  function new(string name, uvm_component parent); super.new(name, parent); endfunction
  task run_phase(uvm_phase phase);
    rights_block blk = rights_block::type_id::create("blk");
    phase.raise_objection(this);
    blk.build();
    blk.reset();
    foreach (blk.R[i]) begin
      uvm_reg_field f = blk.R[i].F;
      bit [7:0] m;
      void'(f.predict(8'h0F, .kind(UVM_PREDICT_WRITE), .map(blk.default_map)));
      void'(f.predict(8'hF0, .kind(UVM_PREDICT_WRITE), .map(blk.default_map)));
      void'(f.predict(8'h3C, .kind(UVM_PREDICT_READ),  .map(blk.default_map)));
      m = f.get_mirrored_value();
      $display("T|%s|%s|%s|%02h", blk.rights[i], blk.R[i].acc,
               f.get_access(blk.default_map), m);
    end
    phase.drop_objection(this);
  endtask
endclass

module top;
  initial run_test("ral_rights_test");
endmodule
