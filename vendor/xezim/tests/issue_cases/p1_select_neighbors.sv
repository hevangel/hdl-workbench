module top;
  logic [7:0] x=8'ha5;
  int errors;
  task check(string name, logic [7:0] got, logic [7:0] want);
    if (got!==want) begin errors++; $display("FAIL %s %h %h",name,got,want); end
    else $display("T|PASS %s %h",name,got);
  endtask
  function automatic logic [7:0] f(input logic [7:0] x);
    x[7:4]=3; x[0]=0; return x;
  endfunction
  task automatic t(output logic [7:0] out);
    logic [7:0] x; x=0; x[3:0]=5; x[7]=1; out=x;
  endtask
  class C;
    logic [7:0] x=8'h40;
    function void wr(); x[0]=1; x[7:4]=9; endfunction
  endclass
  initial begin
    logic [7:0] out; C obj;
    check("input select",f(8'hff),8'h3e);
    t(out); check("local selects",out,8'h85);
    obj=new; obj.wr(); check("property selects",obj.x,8'h91);
    begin logic [7:0] x; x=8'h10; #1; x[6+:2]=3; check("indexed up",x,8'hd0); end
    begin logic [7:0] x; x=8'h10; #1; x[5-:2]=3; check("indexed down",x,8'h30); end
    check("module unchanged",x,8'ha5);
    $display("T|errors=%0d",errors); $finish;
  end
endmodule
