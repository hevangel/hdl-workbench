// Pure-SystemVerilog self-test: a class CONSTRUCTOR's bare-name
// (unqualified) dynamic-array MEMBER write (`value = new[1]`) must not
// clobber a caller task's same-named `ref` formal array.
//
// UVM 4251: `uvm_reg_item::new` does `value = new[1]` on its `value[]`
// member, but `uvm_reg_item::set_value_array` later copies a caller's
// `ref uvm_reg_data_t value[]` in, and `is_enabled_for`...
//
// The exact trigger: a dynamic-array member of a class, allocated via
// `= new[n]` in the constructor, whose BARE name (`value`) collides with a
// `ref` dynamic-array formal of an inlined task (`burst_read(this ref
// big value[])`). The SV scoping rule (§8.9/§8.10) says an unqualified
// name in a method/constructor resolves to the CURRENT frame's own
// local/formal first, then to a class member of `this`, and only then to
// an ENCLOSING (inlined) frame's same-named local or `ref` formal.
//
// xezim routed the `value = new[1]` write target through the enclosing
// frame's rename, so it sized the CALLER's `read_block` (which had been
// `new[64]`) down to 1 instead of the new object's own member. The
// reference simulator keeps both: the caller's array stays 64 and the
// member is sized 1.
module top;
  typedef bit [63:0] big;

  class item;
    big value[];

    function new();
      value = new[1];
    endfunction
  endclass

  item rw;

  task automatic burst_read(ref big value[]);
    rw = new();
    // The `ref` formal is the caller's 64-element array; it must NOT be
    // shrunk by the constructor's `value = new[1]` above.
    if (value.size() !== 64) begin
      $display("TAG_FAIL ref formal shrunk to %0d", value.size());
      return;
    end
  endtask

  task automatic check_burst();
    big read_block[];
    read_block = new[64];
    burst_read(read_block);
    if (read_block.size() === 64 && rw.value.size() === 1)
      $display("TAG_PASS");
    else
      $display("TAG_FAIL caller=%0d member=%0d", read_block.size(), rw.value.size());
  endtask

  initial begin
    check_burst();
  end
endmodule