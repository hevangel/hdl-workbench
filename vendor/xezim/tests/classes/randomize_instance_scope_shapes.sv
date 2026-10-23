// Neighbours of issues #303-#305: randomize() with inline constraints and
// array-method `with` clauses inside module instances.
class rs_obj;
  rand int x;
  rand int y;
  rand int carr [3];
endclass

module rs_kid #(parameter int LO = 100, parameter int HI = 200);
  bit [15:0] fa [6];
  int da [];
  int lim;
  initial begin
    int ok;
    lim = HI;
    ok = std::randomize(fa) with { foreach (fa[i]) { fa[i] >= LO; fa[i] <= lim; } };
    $display("T|%m fixed ok=%0d inrange=%0d", ok,
             fa[0] >= LO && fa[0] <= HI && fa[5] >= LO && fa[5] <= HI);
    ok = std::randomize(da) with { da.size() == 3 + LO / 100; foreach (da[i]) da[i] inside {[LO:lim]}; };
    $display("T|%m dyn ok=%0d size=%0d inrange=%0d", ok, da.size(),
             da.size() > 0 && da[0] >= LO && da[0] <= HI);
  end
endmodule

module rs_gen;
  for (genvar g = 0; g < 2; g++) begin : gb
    int arr [4];
    int q [$];
    int base;
    initial begin
      int ok;
      base = 10 * (g + 1);
      ok = std::randomize(arr) with { foreach (arr[i]) arr[i] inside {[base:base+5]}; };
      $display("T|%m g=%0d ok=%0d inr=%0d", g, ok,
               arr[0] >= base && arr[0] <= base + 5 && arr[3] >= base && arr[3] <= base + 5);
      ok = std::randomize(q) with { q.size() == g + 2; foreach (q[i]) q[i] == base + i; };
      $display("T|%m g=%0d qok=%0d qsize=%0d q0=%0d qlast=%0d", g, ok, q.size(), q[0], q[q.size()-1]);
    end
  end
  if (1) begin : ib
    bit [7:0] arr [3];
    initial begin
      int ok;
      ok = std::randomize(arr) with { foreach (arr[i]) arr[i] == 7 + i; };
      $display("T|%m ok=%0d a=%0d %0d %0d", ok, arr[0], arr[1], arr[2]);
    end
  end
endmodule

module rs_cls;
  int x;
  int lim;
  int marr [3];
  int mq [$];
  rs_obj c;
  initial begin
    int ok;
    #2;
    x = 99;
    lim = 50;
    marr = '{11, 22, 33};
    mq = {5, 6};
    c = new;
    ok = c.randomize() with { x > 0; x < lim; y == x + 1; };
    $display("T|%m c1 ok=%0d in=%0d y=%0d modx=%0d", ok, c.x > 0 && c.x < 50, c.y == c.x + 1, x);
    ok = c.randomize() with { x == marr[1]; foreach (carr[i]) carr[i] == marr[i] + mq.size(); };
    $display("T|%m c2 ok=%0d x=%0d a=%0d %0d %0d", ok, c.x, c.carr[0], c.carr[1], c.carr[2]);
    ok = c.randomize() with { x inside {marr}; x != marr[0]; x != marr[2]; };
    $display("T|%m c3 ok=%0d x=%0d", ok, c.x);
    ok = c.randomize() with { x == local::x + mq.sum() with (item * 2); };
    $display("T|%m c4 ok=%0d x=%0d", ok, c.x);
  end
endmodule

module rs_coll;
  int q [$];
  byte dq [];
  int n;
  initial begin
    int ok;
    #3;
    n = 5;
    ok = std::randomize(q) with { q.size() == n; foreach (q[i]) q[i] inside {[1:9]}; q.sum() == 20; };
    $display("T|%m q ok=%0d size=%0d sum=%0d", ok, q.size(), q.sum());
    dq = new[4];
    ok = std::randomize(dq) with { foreach (dq[i]) dq[i] == i - n; };
    $display("T|%m dq ok=%0d size=%0d d0=%0d d3=%0d", ok, dq.size(), dq[0], dq[3]);
  end
endmodule

module rs_vars;
  int lo, hi, v, w;
  int arr [5];
  int found [$];
  initial begin
    int ok;
    #4;
    lo = 40; hi = 45;
    ok = std::randomize(v) with { v >= lo; v <= hi; };
    $display("T|%m v ok=%0d in=%0d", ok, v >= 40 && v <= 45);
    ok = std::randomize(v, w) with { v == lo + 1; w == v + hi; };
    $display("T|%m vw ok=%0d v=%0d w=%0d", ok, v, w);
    foreach (arr[i]) arr[i] = i + lo;
    $display("T|%m sumwith=%0d", arr.sum() with (item - lo));
    found = arr.find(e) with (e > lo + 2);
    $display("T|%m find=%0d", found.size());
    ok = std::randomize(v) with { v == arr.sum() with (item - lo); };
    $display("T|%m sumv ok=%0d v=%0d", ok, v);
  end
endmodule

module top;
  bit [15:0] fa [6];
  int da [];
  int lim = 5;
  int arr [4];
  int x = 3;
  int marr [3] = '{1, 2, 3};
  int mq [$];
  int q [$];
  int lo = 1000, hi = 1001, v, w;
  rs_kid #(.LO(100), .HI(200)) a();
  rs_kid #(.LO(1000), .HI(2000)) b();
  rs_gen k();
  rs_cls c();
  rs_coll l();
  rs_vars r();
  initial begin
    #10;
    $display("T|top fa0=%0d da=%0d lim=%0d arr0=%0d x=%0d q=%0d v=%0d w=%0d lo=%0d",
             fa[0], da.size(), lim, arr[0], x, q.size(), v, w, lo);
  end
endmodule
