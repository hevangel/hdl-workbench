// Mini RISC-style pipeline exercising the ibex-hot construct cocktail:
// dense case decoder (jump table), wildcard casez (two-level dispatch),
// byte-masked RAM writes, CSR-style constant readback mux, and a
// tracer-style void decode helper with string formals + $sformatf.
module top;
  logic clk = 0;
  always #5 clk = ~clk;
  logic [31:0] insn, acc, csr_rdata, mem_rd, chk;
  logic [15:0] cinsn;
  logic [3:0] opcode;
  logic [2:0] cls;
  logic [31:0] mem [0:15];
  logic [3:0] wmask;
  logic [31:0] wdata;
  logic [3:0] addr;
  logic [11:0] csr_addr;
  string dec;
  logic [31:0] cyc;
  integer i;

  function automatic void trace_op(input string mnemonic, input logic [3:0] op);
    dec = $sformatf("%s\tr%0d,0x%0x", mnemonic, op, acc[15:0]);
  endfunction

  always @(posedge clk) begin
    // dense decoder: >=8 const arms, arbitrary bodies
    case (opcode)
      4'd0: begin acc <= acc + insn; trace_op("add", opcode); end
      4'd1: begin acc <= acc ^ insn; trace_op("xor", opcode); end
      4'd2: begin acc <= acc | insn; trace_op("or", opcode); end
      4'd3: begin acc <= acc & (insn | 32'h1); trace_op("and", opcode); end
      4'd4: begin acc <= acc << 1; trace_op("sll", opcode); end
      4'd5: begin acc <= acc >> 1; trace_op("srl", opcode); end
      4'd6: begin acc <= acc + {insn[15:0], 16'h0}; trace_op("lui", opcode); end
      4'd7: begin acc <= acc - insn; trace_op("sub", opcode); end
      default: begin acc <= acc + 32'd3; trace_op("nop", opcode); end
    endcase
    // wildcard compressed-decode: casez over masked patterns
    unique casez (cinsn)
      16'b000?_????_????_??00: cls <= 3'd1;
      16'b010?_????_????_??00: cls <= 3'd2;
      16'b100?_????_????_??00: cls <= 3'd3;
      16'b000?_????_????_??01: cls <= 3'd4;
      16'b010?_????_????_??01: cls <= 3'd5;
      16'b100?_????_????_??01: cls <= 3'd6;
      16'b110?_????_????_??10: cls <= 3'd7;
      16'b1111_????_????_??11: cls <= 3'd0;
      default:                 cls <= 3'd0;
    endcase
    // byte-masked RAM write, ibex prim_ram shape
    for (i = 0; i < 4; i = i + 1)
      if (wmask[i])
        mem[addr][8*i +: 8] <= wdata[8*i +: 8];
    mem_rd <= mem[addr ^ 4'h1];
    // CSR readback mux: const-result dense case
    case (csr_addr)
      12'h300: csr_rdata <= 32'h1800;
      12'h301: csr_rdata <= 32'h4014_1101;
      12'h304: csr_rdata <= 32'h888;
      12'h305: csr_rdata <= 32'h100;
      12'h340: csr_rdata <= 32'hdead_0001;
      12'h341: csr_rdata <= 32'hdead_0002;
      12'h342: csr_rdata <= 32'hdead_0003;
      12'h343: csr_rdata <= 32'hdead_0004;
      default: csr_rdata <= 32'h0;
    endcase
    // stimulus walk
    insn <= {insn[27:0], insn[31:28]} ^ 32'h9e37_79b9;
    cinsn <= cinsn + 16'h1357;
    opcode <= opcode + 4'd1;
    csr_addr <= (cyc[3]) ? 12'h300 + {8'h0, cyc[2:0], 1'b0} : 12'h7ff;
    wmask <= cyc[3:0];
    wdata <= wdata + 32'h0101_0101;
    addr <= addr + 4'd1;
    chk <= {chk[30:0], chk[31]} ^ csr_rdata ^ {29'h0, cls} ^ mem_rd ^ {24'h0, dec.getc(0)};
    cyc <= cyc + 1;
  end

  initial begin
    insn = 32'h0000_1111; chk = 0; cls = 0; csr_rdata = 0; mem_rd = 0; dec = "-"; cinsn = 16'h0; opcode = 0; acc = 0; cyc = 0;
    wmask = 0; wdata = 32'h0403_0201; addr = 0; csr_addr = 12'h300;
    for (i = 0; i < 16; i = i + 1) mem[i] = 32'h0;
    wait (cyc >= 32'd50000);
    $display("IBX acc=%h chk=%h dec=[%s]", acc, chk, dec);
    $finish;
  end
endmodule
