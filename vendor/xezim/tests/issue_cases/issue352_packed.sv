module top;
  typedef struct packed { logic [3:0] a; logic [3:0] b; } s_t;
  typedef struct packed { logic [8:1] f; logic [0:3] g; } e_t;
  int errors = 0;
  task automatic check(input string what, input logic [31:0] got, input logic [31:0] want);
    if (got !== want) begin
      errors++;
      $display("FAIL %s: got %h, want %h", what, got, want);
    end else $display("PASS %s", what);
  endtask
  task automatic t;
    logic [1:0][7:0] p;
    logic [1:0][2:0][3:0] q;
    s_t [2:0] a;
    e_t e;
    e_t [0:1] b;
    int i;
    p = '0;
    q = '0;
    a = '0;
    e = '0;
    b = '0;
    i = 1;
    p[1][3:0] = 4'h9;
    p[0][7] = 1'b1;
    check("2-D", p, 16'h0980);
    p[i][7 -: 2] = 2'b11;
    check("2-D, -: after an element select", p, 16'hC980);
    q[1][2][3] = 1'b1;
    q[0][1][2:1] = 2'b11;
    q[1][1:0] = 8'hAB;
    check("3-D", q, 24'h8AB_060);
    a[i].a[3] = 1'b1;
    a[2].b[1:0] = 2'b10;
    check("packed array of structs", a, 24'h02_80_00);
    e = 12'hABC;
    check("member declared [8:1], read", e.f[4:1], 4'hB);
    e = '0;
    e.f[4:1] = 4'hB;
    e.g[0:1] = 2'b11;
    check("members declared [8:1] and [0:3]", e, 12'h0BC);
    b = 24'hABC_123;
    check("ascending array of structs, read", b[1].g, 4'h3);
    b = '0;
    b[1].g = 4'h3;
    check("ascending array of structs", b, 24'h000_003);
  endtask
  task automatic out_arg(output logic [1:0][7:0] o);
    o = '0;
    o[1][3:0] = 4'h9;
  endtask
  function automatic logic [1:0][7:0] ret();
    ret = '0;
    ret[1][3:0] = 4'h9;
  endfunction
  initial begin
    logic [1:0][7:0] o;
    t;
    out_arg(o);
    check("output formal", o, 16'h0900);
    check("return value", ret(), 16'h0900);
    if (errors == 0) $display("ALL PASS");
    $finish;
  end
endmodule
