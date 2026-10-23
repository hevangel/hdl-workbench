// Self-test for a BIT-SELECT write into a FRAME-LOCAL (block-scoped)
// struct member (IEEE 1800 §7.2, §11.4), the exact shape
// uvm_reg_map::do_bus_access uses to build a bus op's byte-enable:
//
//     foreach (adr[i]) begin
//       uvm_reg_bus_op rw_access;   // block-local struct
//       ...
//       for (int z=0; z<bus_width; z++)
//         rw_access.byte_en[z] = be[bus_width*i+z];  // indexed member write
//       accesses.push_back(rw_access);
//     end
//
// Xezim stored the block-local struct's members as leaves in the current
// call/local frame (`rw_access.byte_en`), but the indexed write minted a
// phantom `rw_access.byte_en[z]` element that no whole-member read
// consulted, so byte_en read back 0 (the 4168 reg bit-bash failure).
// A whole/member/whole-element write and the block-scoped scalar `bit`
// must still work alongside.
module top;
  typedef bit unsigned [7:0] byte_en_t;
  typedef bit unsigned [63:0] reg_data_t;
  typedef bit unsigned [63:0] reg_addr_t;
  typedef struct {
    reg_addr_t addr;
    reg_data_t data;
    int n_bits;
    byte_en_t byte_en;
    int status;
  } uvm_reg_bus_op;

  bit be[$];
  byte unsigned p[$];
  initial begin
    uvm_reg_bus_op accesses[$];
    reg_addr_t adr[$];
    int bus_width = 1;
    be.push_back(1'b1); be.push_back(1'b1);
    be.push_back(1'b1); be.push_back(1'b1);
    be.push_back(1'b0);
    p.push_back(8'h01);
    adr.push_back(0); adr.push_back(1); adr.push_back(2); adr.push_back(3);
    foreach (adr[i]) begin
      uvm_reg_bus_op rw_access;   // block-scoped struct
      reg_data_t data_i;
      for (int i0=0;i0<bus_width;i0++)
        data_i[i0*8+:8] = p[i*bus_width+i0];
      for (int z=0;z<bus_width;z++)
        rw_access.byte_en[z] = be[bus_width*i+z];   // indexed member write
      rw_access.addr = adr[i];
      rw_access.data = data_i;
      accesses.push_back(rw_access);
    end
    // be = {1,1,1,1,0}; for bus_width=1 each iteration writes byte_en[0]=be[?]
    // (bit 0 of the member) = 1, so byte_en must read back 8'b00000001.
    if (accesses.size() == 4
        && accesses[0].addr == 0 && accesses[0].byte_en == 8'b00000001
        && accesses[1].addr == 1 && accesses[1].byte_en == 8'b00000001
        && accesses[2].addr == 2 && accesses[2].byte_en == 8'b00000001
        && accesses[3].addr == 3 && accesses[3].byte_en == 8'b00000001)
      $display("TAG_PASS");
    else
      $display("TAG_FAIL n=%0d b0=%02b b1=%02b b2=%02b b3=%02b",
               accesses.size(),
               (accesses.size()>0)?accesses[0].byte_en:8'hxx,
               (accesses.size()>1)?accesses[1].byte_en:8'hxx,
               (accesses.size()>2)?accesses[2].byte_en:8'hxx,
               (accesses.size()>3)?accesses[3].byte_en:8'hxx);
  end
endmodule