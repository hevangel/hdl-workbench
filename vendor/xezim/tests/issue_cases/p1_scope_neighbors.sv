module top;
  int errors;
  task check(string name, int got, int want);
    if (got != want) begin errors++; $display("FAIL %s %0d %0d", name, got, want); end
    else $display("T|PASS %s %0d", name, got);
  endtask
  bit [2:0] i = 5;
  int ra, rb, rc, rd, re, rf;
  task automatic a(output int r); int v; v = 1000; #5; v++; r = v; endtask
  task automatic b(output int r); bit [3:0] v; v = 3; #10; v += 15; r = v; endtask
  task automatic c(output int q); #15; q = -1000; endtask
  task automatic d(); bit [3:0] q; q = 3; #20; endtask
  task automatic e(output int r);
    int w; w = -1;
    begin bit [3:0] w; w = 15; end
    w--; r = w;
  endtask
  task automatic f(output int r); int v; v = 77; #8; v += 2; r = v; endtask
  task automatic g(); string v; v = "word"; #18; endtask
  initial a(ra);
  initial b(rb);
  initial c(rc);
  initial d();
  initial e(rd);
  initial f(re);
  initial g();
  initial begin
    int n; n = 0;
    for (int i = 3; i >= 0; i--) begin n++; if(n>8) break; #1; end
    check("loop signed", n, 4);
    check("loop outer", i, 5);
    begin
      shortint j; j = -2;
      begin bit [1:0] j; j = 3; end
      check("short outer", j, -2);
    end
    begin
      byte j; j = -8;
      begin int unsigned j; j = 99; end
      check("byte outer", j, -8);
    end
    #30;
    check("concurrent int",ra,1001);
    check("concurrent bits",rb,2);
    check("output formal",rc,-1000);
    check("nested signed",rd,-2);
    check("string collision",re,79);
    $display("T|errors=%0d",errors);
    $finish;
  end
endmodule
