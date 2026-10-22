module top;
  typedef int sint_t;
  int n, errors;
  bit [2:0] i = 5;
  task check(string name, int got, int want);
    if(got!=want) begin errors++; $display("FAIL %s %0d %0d",name,got,want); end
    else $display("T|PASS %s %0d",name,got);
  endtask
  initial begin
    n=0; for(byte i=3;i>=0;i--) begin n++; if(n>8) break; #1; end
    check("byte",n,4); check("outer byte",i,5);
    n=0; for(shortint i=3;i>-1;i--) begin n++; if(n>8) break; #1; end
    check("shortint",n,4); check("outer shortint",i,5);
    n=0; for(integer i=3;i>=0;i--) begin n++; if(n>8) break; #1; end
    check("integer",n,4); check("outer integer",i,5);
    n=0; for(logic signed [5:0] i=3;i>=0;i--) begin n++; if(n>8) break; #1; end
    check("packed signed",n,4); check("outer packed",i,5);
    n=0; for(sint_t i=3;i>=0;i--) begin n++; if(n>8) break; #1; end
    check("typedef",n,4); check("outer typedef",i,5);
    $display("T|errors=%0d",errors); $finish;
  end
endmodule
