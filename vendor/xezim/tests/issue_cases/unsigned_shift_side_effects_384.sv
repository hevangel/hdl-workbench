// §11.8.2: type discovery must not evaluate calls or inactive conditional arms.
module top;
  logic [15:0] acc;
  int calls, untaken, errors;
  function automatic logic signed [7:0] sample(); calls++; return 8'shfc; endfunction
  function automatic logic [15:0] skipped(); untaken++; return 0; endfunction
  task automatic ck(string label, logic [15:0] got, logic [15:0] want);
    if (got === want) $display("PASS %s",label);
    else begin errors++; $display("FAIL %s %h %h",label,got,want); end
  endtask
  initial begin
    acc = 0;
    ck("function context", acc + (sample() >>> 1), 16'h007e);
    ck("one evaluation", calls, 1);
    ck("conditional function context", 1'b1 ? (sample() >>> 1) : skipped(), 16'h007e);
    ck("inactive call", untaken, 0);
    ck("conditional one evaluation", calls, 2);
    ck("right function context", (sample() >>> 1) + acc, 16'h007e);
    ck("third single evaluation", calls, 3);
    if (errors==0) $display("ALL PASS"); else $display("FAILED %0d",errors);
    $finish;
  end
endmodule
