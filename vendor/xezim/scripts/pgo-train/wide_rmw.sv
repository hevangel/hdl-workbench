module tb;
  logic clk = 0; always #5 clk = ~clk;
  logic [335:0] acc; logic [43:0] tag; int cyc = 0;
  // A blocking read-modify-write of a 336-bit register: the block reads
  // `acc` and then overwrites it, so a bail after the write would re-run
  // against the new value unless the old one is saved first.
  always @(posedge clk) begin
    acc = {acc[291:0], acc[335:292] ^ tag};
    tag = tag + 44'd7;
    cyc <= cyc + 1;
  end
  initial begin
    acc = {8{42'h2_5a5a_5a5a_5}}; tag = 44'h1;
    repeat (20000) @(posedge clk);
    #1 $display("RMW %h %h %0d", acc[63:0], acc[335:272], cyc);
    $finish;
  end
endmodule
