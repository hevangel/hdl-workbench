// Neighbours of issues #303-#305: randomize() with and array-method with
// clauses in a task and a function of two instances of one module.
module rs_sub;
  int arr [4];
  int lim;
  int q [$];
  task automatic go(int base);
    int ok;
    ok = std::randomize(arr) with { foreach (arr[i]) arr[i] == base + i + lim; };
    $display("T|%m task ok=%0d a0=%0d a3=%0d", ok, arr[0], arr[3]);
  endtask
  function automatic int total();
    return arr.sum() with (item - lim);
  endfunction
  initial begin
    lim = 100;
    go(5);
    $display("T|%m total=%0d", total());
    q = {1, 2, 3};
    void'(std::randomize(q) with { q.size() == 2; foreach (q[i]) q[i] == lim + i; });
    $display("T|%m q=%0d %0d %0d", q.size(), q[0], q[1]);
  end
endmodule
module top;
  int arr [4];
  int lim = 7;
  rs_sub k1();
  rs_sub k2();
  initial #1 $display("T|top a0=%0d lim=%0d", arr[0], lim);
endmodule
