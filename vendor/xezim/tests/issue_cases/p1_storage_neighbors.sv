module worker(output int seen);
  initial begin begin int n; n = 9; #3; n++; seen = n; end end
endmodule
module top;
  int errors, a, b, c, d, inst;
  event kick;
  worker u(inst);
  task check(string name, int got, int want);
    if (got != want) begin errors++; $display("FAIL %s %0d %0d", name, got, want); end
    else $display("T|PASS %s %0d", name, got);
  endtask
  always @(kick) begin int n; n++; a = n; end
  always @(kick) begin static int v = 20; v++; b = v; end
  initial begin
    begin : outer
      begin : inner
        int n; static int v = 100;
        n = 50; v++; #2; c = n; d = v;
      end
    end
  end
  initial begin
    begin int n; n = 80; #1; n++; end
    -> kick; #1; -> kick; #2;
    check("always nested collision",a,2);
    check("static nested collision",b,22);
    check("named nesting",c,50);
    check("named static",d,101);
    check("instance nesting",inst,10);
    $display("T|errors=%0d",errors); $finish;
  end
endmodule
