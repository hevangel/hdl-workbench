module top;
  int errors = 0;
  task check(input string what, input logic [7:0] got, input logic [7:0] want);
    if (got !== want) begin
      errors++;
      $display("FAIL %s: got %h, want %h", what, got, want);
    end else $display("PASS %s", what);
  endtask

  // Module variables named like the locals below.
  logic [7:0] a = 8'h11, k = 8'h12, y = 8'h13, t = 8'h14, d = 8'h15;
  logic clk = 0;
  logic [7:0] r;

  // A task local.
  task automatic t_local(output logic [7:0] o);
    logic [7:0] a;
    a = 8'h00;
    a[3:0] = 4'h5;
    o = a;
  endtask

  // A function input.
  function automatic logic [7:0] f_formal(input logic [7:0] k);
    k[7:4] = 4'h3;
    k[0] = 1'b0;
    return k;
  endfunction

  // A class property, in a method.
  class C;
    logic [7:0] y = 8'h81;
    function void set();
      y[7:4] = 4'h9;
    endfunction
  endclass

  // A block variable of an always block: static, so it keeps its value.
  always @(posedge clk) begin : blk
    logic [7:0] t;
    if (t === 8'bx) t = 8'h00;
    t = t + 1;
    t[7] = 1'b1;
    r = t;
  end

  initial begin
    C o;
    logic [7:0] v;
    repeat (3) begin
      #1 clk = 1;
      #1 clk = 0;
    end
    check("always block variable", r, 8'h83);
    t_local(v);
    check("task local", v, 8'h05);
    check("function input", f_formal(8'hFF), 8'h3E);
    o = new;
    o.set();
    check("class property", o.y, 8'h91);
    // A block variable of a block that waits.
    begin
      logic [7:0] d;
      d = 8'h00;
      #1;
      d[1:0] = 2'b11;
      check("block variable", d, 8'h03);
    end
    check("module a", a, 8'h11);
    check("module k", k, 8'h12);
    check("module y", y, 8'h13);
    check("module t", t, 8'h14);
    check("module d", d, 8'h15);
    if (errors == 0) $display("ALL PASS");
  end
endmodule
