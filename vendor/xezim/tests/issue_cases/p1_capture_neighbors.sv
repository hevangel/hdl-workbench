module top;
  event go;
  int errors, seen, finished;
  string result;
  task automatic a(output string r);
    string s;
    s = "old";
    #5;
    begin int s; s = 22; end
    s = "abcdefgh";
    s[0] = "Z";
    r = s;
  endtask
  task automatic b;
    int s;
    s = 1;
    #12;
  endtask
  initial a(result);
  initial b;
  initial begin
    for(int i=0;i<1;i++) begin
      #1;
      fork
        begin
          repeat(2) begin
            @go;
            begin int n; n++; seen=n; end
          end
          finished=1;
        end
      join_none
    end
    #1 ->go; #1 ->go; #1;
    $display("T|capture %0d %0d",seen,finished);
    if(seen!=2 || finished!=1) errors++;
    #12;
    $display("T|string %s",result);
    if(result!="Zbcdefgh") errors++;
    $display("T|errors=%0d",errors); $finish;
  end
endmodule
