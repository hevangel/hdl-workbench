// Same-named variables of different declarations must not share a type or
// storage (IEEE 1800-2017 6.21, 12.7.1, 13.3).
module t;
  int a = 0, b = 0, r1, r2, r3, r4, r5, r6;

  // 1. for-loop variables in two fork branches
  initial fork
    for (int i = 0; i < 20; i++) begin #10; a++; end
    begin #5; for (bit [2:0] i = 0; i < 5; i++) begin #1; b++; end end
  join

  // 2. locals of two automatic tasks running at once
  task automatic ta(output int r);
    int v; v = 1000; #10; v = v + 1; r = v;
  endtask
  task automatic tb(output int r);
    bit [3:0] v; #5; v = 3; #10; v = v + 4'd15; r = v;
  endtask
  initial ta(r1);
  initial tb(r2);

  // 3. a task formal while another process holds a same-named local
  task automatic tc(output int q);
    #10; q = -1000;
  endtask
  task automatic td();
    bit [3:0] q; #5; q = 3; #20;
  endtask
  initial tc(r3);
  initial td();

  // 4. a nested block's local shadowing a local of the same task
  task automatic te(output int r);
    int w; w = -1;
    begin bit [3:0] w; w = 4'hF; end
    w = w - 1; r = w;
  endtask
  initial te(r4);

  // 5. nested block locals of two initial blocks
  initial begin begin int y; y = 1000; #10; y = y + 1; r5 = y; end end
  initial begin begin bit [3:0] y; #5; y = 3; #10; y = y + 4'd15; r6 = y; end end

  initial #300 begin
    $display("%s 1: int for loop ran %0d times (expect 20)", a == 20 ? "PASS" : "FAIL", a);
    $display("%s 2: int task local = %0d (expect 1001), bit [3:0] local = %0d (expect 2)",
             r1 == 1001 && r2 == 2 ? "PASS" : "FAIL", r1, r2);
    $display("%s 3: int formal = %0d (expect -1000)", r3 == -1000 ? "PASS" : "FAIL", r3);
    $display("%s 4: int local after block = %0d (expect -2)", r4 == -2 ? "PASS" : "FAIL", r4);
    $display("%s 5: int block local = %0d (expect 1001), bit [3:0] block local = %0d (expect 2)",
             r5 == 1001 && r6 == 2 ? "PASS" : "FAIL", r5, r6);
    $finish;
  end
endmodule
