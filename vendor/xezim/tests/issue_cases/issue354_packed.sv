module top;
  logic [3:0][7:0] arr [4];
  int errors = 0;
  task automatic check(input string what, input logic [31:0] got, input logic [31:0] want);
    if (got !== want) begin
      errors++;
      $display("FAIL %s: got %h, want %h", what, got, want);
    end else $display("PASS %s", what);
  endtask
  task automatic t;
    logic [3:0][7:0] la [4];
    logic [1:0][3:0] ld [];
    logic [1:0][3:0] lm [2][3];
    foreach (la[i]) la[i] = '0;
    la[1][2] = 8'hAB;
    la[1][2][3:0] = 4'h5;
    la[3][0][7] = 1'b1;
    check("local unpacked array", la[1], 32'h00A5_0000);
    check("local unpacked array, bit", la[3], 32'h0000_0080);
    check("local unpacked array, read", la[1][2][7:4], 4'hA);
    ld = new[2];
    ld[1] = '0;
    ld[1][1][2] = 1'b1;
    ld[1][0] = 4'h3;
    check("local dynamic array", ld[1], 8'h43);
    foreach (lm[i, j]) lm[i][j] = '0;
    lm[1][2][1] = 4'hC;
    lm[1][2][0][1] = 1'b1;
    check("local 2-D unpacked array", lm[1][2], 8'hC2);
  endtask
  initial begin
    foreach (arr[i]) arr[i] = '0;
    arr[1][2] = 8'hAB;
    arr[1][2][3:0] = 4'h5;
    check("module variable", arr[1], 32'h00A5_0000);
    check("module variable, read", arr[1][2][7:4], 4'hA);
    t;
    if (errors == 0) $display("ALL PASS");
    $finish;
  end
endmodule
