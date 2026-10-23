// AES-shaped workload: 128-bit state, per-byte sbox substitution through a
// function, byte rotation, round-key evolution, driven by a task-structured
// FSM with argument-carrying tasks — the exact cocktail that used to drag
// whole always blocks onto the interpreter.
module top;
  logic clk = 0;
  always #5 clk = ~clk;
  logic [127:0] state, rkey;
  
  logic [3:0] round;
  logic [1:0] fsm;
  logic [31:0] blocks, done_blocks;
  logic [127:0] digest;
  integer i;

  function automatic logic [7:0] sub_byte(input logic [7:0] b);
    logic [7:0] s;
    case (b[3:0])
      4'h0: s = 8'h63;
      4'h1: s = 8'h7c;
      4'h2: s = 8'h77;
      4'h3: s = 8'h7b;
      4'h4: s = 8'hf2;
      4'h5: s = 8'h6b;
      4'h6: s = 8'h6f;
      4'h7: s = 8'hc5;
      4'h8: s = 8'h30;
      4'h9: s = 8'h01;
      4'ha: s = 8'h67;
      4'hb: s = 8'h2b;
      4'hc: s = 8'hfe;
      4'hd: s = 8'hd7;
      4'he: s = 8'hab;
      4'hf: s = 8'h76;
      default: s = 8'h00;
    endcase
    return s ^ {b[3:0], b[7:4]};
  endfunction

  task automatic apply_round(input logic [3:0] r);
    logic [127:0] t;
    t = {sub_byte(state[127:120] ^ rkey[127:120]),
           sub_byte(state[119:112] ^ rkey[119:112]),
           sub_byte(state[111:104] ^ rkey[111:104]),
           sub_byte(state[103:96] ^ rkey[103:96]),
           sub_byte(state[95:88] ^ rkey[95:88]),
           sub_byte(state[87:80] ^ rkey[87:80]),
           sub_byte(state[79:72] ^ rkey[79:72]),
           sub_byte(state[71:64] ^ rkey[71:64]),
           sub_byte(state[63:56] ^ rkey[63:56]),
           sub_byte(state[55:48] ^ rkey[55:48]),
           sub_byte(state[47:40] ^ rkey[47:40]),
           sub_byte(state[39:32] ^ rkey[39:32]),
           sub_byte(state[31:24] ^ rkey[31:24]),
           sub_byte(state[23:16] ^ rkey[23:16]),
           sub_byte(state[15:8] ^ rkey[15:8]),
           sub_byte(state[7:0] ^ rkey[7:0])};
    state <= {t[119:0], t[127:120]} ^ {124'h0, r};
    rkey <= {rkey[126:0], rkey[127] ^ rkey[97] ^ rkey[60]};
  endtask

  always @(posedge clk) begin
    case (fsm)
      2'd0: begin
        state <= {96'h00112233_44556677_8899aabb, blocks};
        rkey <= 128'h0f1e2d3c_4b5a6978_8796a5b4_c3d2e1f0;
        round <= 0;
        fsm <= 2'd1;
      end
      2'd1: begin
        apply_round(round);
        round <= round + 1;
        if (round == 4'd9) fsm <= 2'd2;
      end
      default: begin
        digest <= {digest[126:0], digest[127]} ^ state ^ {124'h0, fsm};
        blocks <= blocks + 1;
        done_blocks <= done_blocks + 1;
        fsm <= 2'd0;
      end
    endcase
  end

  initial begin
    state = 0; rkey = 0; round = 0; fsm = 0; blocks = 0; done_blocks = 0; digest = 0;
    wait (done_blocks >= 32'd2000);
    $display("AESM digest=%h blocks=%0d", digest, blocks);
    $finish;
  end
endmodule
