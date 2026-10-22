// Self-test for the casting-to-unpacked-queue of a STREAMING concat
// (IEEE 1800 §6.24.1 + §11.4.14), as uvm_reg_map::do_bus_access relies on
// to place a partial-register field's value onto the right byte lane.
//
// For a field at bit offset `lsb` inside a byte-addressed (n_bytes=1) map,
// UVM computes the per-byte bus write data with:
//     bits = {<< {byte_queue}};        // byte stream -> bit queue
//     repeat(bit_shift) bits.push_front(1'b0);
//     bytes = {<< 8 {bit_q_t'({<< {bits}})}};   // repack, 8 bits at a time
// A single field byte 0x3F at lsb=1 must come out as bus bytes {0x7E, 0x00}:
// the operand stream (the cast `bit_q_t'({<< {bits}})`) evaluates to an
// UNPACKED queue, and collapsing it to the element width (1) would truncate
// the shift to nothing and emit a data byte of 0x00 instead (the 3641 reg
// failure: field f0[6:1] write sent 0x00 instead of 0x7E).
module top;
  typedef bit bit_q_t[$];
  initial begin
    byte unsigned p[$];
    bit bits[$];
    p = '{8'h3F};                      // raw field byte
    bits = {<< {p}};                   // byte stream -> bit queue (8)
    bits.push_front(1'b0);             // byte-shift by lsb=1
    p = {<< 8 {bit_q_t'({<< {bits}}) }};
    if (p.size() == 2 && p[0] == 8'h7E && p[1] == 8'h00)
      $display("TAG_PASS");
    else
      $display("TAG_FAIL size=%0d p0=%02x p1=%02x", p.size(),
               (p.size() > 0) ? p[0] : 8'hxx,
               (p.size() > 1) ? p[1] : 8'hxx);
  end
endmodule