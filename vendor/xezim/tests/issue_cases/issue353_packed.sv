module top;
  typedef struct packed { logic [8:1] f; logic [0:3] g; } e_t;
  logic [1:0][2:0][3:0] m3;
  logic [1:0][1:0][1:0][3:0] m4;
  e_t me;
  e_t [1:0] ma;
  int errors = 0;
  task automatic check(input string what, input logic [31:0] got, input logic [31:0] want);
    if (got !== want) begin
      errors++;
      $display("FAIL %s: got %h, want %h", what, got, want);
    end else $display("PASS %s", what);
  endtask
  task automatic from_task;
    me = '0;
    me.f[4:1] = 4'hB;
    me.g[0:1] = 2'b11;
    check("members declared [8:1] and [0:3], written in a task", me, 12'h0BC);
    ma = 24'hABC_123;
    check("member of an array element, read in a task", ma[1].f[4:1], 4'hB);
    ma = '0;
    ma[1].f[4:1] = 4'hB;
    check("member of an array element, written in a task", ma, 24'h0B0_000);
  endtask
  initial begin
    m3 = '0;
    m3[0][1][2:1] = 2'b11;
    m3[0][2][1 +: 2] = 2'b10;
    check("3-D part-selects", m3, 24'h000_460);
    m3[1][2:1] = 8'hAB;
    check("element range of the middle dimension", m3, 24'hAB0_460);
    check("element range of the middle dimension, read", m3[1][2:1], 8'hAB);
    m4 = '0;
    m4[0][1][0][3:1] = 3'b101;
    check("4-D part-select", m4, 32'h0000_0A00);
    begin : blk
      logic [1:0][2:0][3:0] b3;
      b3 = '0;
      b3[0][1][2:1] = 2'b11;
      check("block variable", b3, 24'h000_060);
    end
    from_task;
    if (errors == 0) $display("ALL PASS");
    $finish;
  end
endmodule
