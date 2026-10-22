// top: fun
module fun;
  logic [3:0] sel; int r;
  initial begin
    sel = 3;
    unique if (sel == 3) r = 1; else if (sel > 2) r = 2;
    sel = 9;
    priority if (sel == 3) r = 5; else if (sel == 4) r = 6;
    unique case (sel) 5: r = 5; 6: r = 6; endcase
    $display("T|12.4.2|done r=%0d", r);
  end
endmodule
