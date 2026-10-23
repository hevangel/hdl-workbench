module tb;
  int miq [3] [$];
  int mdq [2] [];
  int maq [2] [string];
  int r [$];
  int src [$] = '{4, 5, 6};
  initial begin
    for (int i = 0; i < 8; i++) miq[1].push_back(i);
    $display("T|01 size=%0d p=%p", miq[1].size(), miq[1]);
    miq[0] = miq[1];
    $display("T|02 copy size=%0d p=%p", miq[0].size(), miq[0]);
    miq[2] = src;
    $display("T|03 from var size=%0d", miq[2].size());
    r = miq[1];
    $display("T|04 to var size=%0d", r.size());
    r = miq[1].find with (item > 4);
    $display("T|05 find to var %p", r);
    miq[0] = miq[1].find with (item > 5);
    $display("T|06 find to elem %p", miq[0]);
    miq[2] = {miq[1][0], 77};
    $display("T|07 concat %p", miq[2]);
    miq[2] = '{1, 2, 3};
    $display("T|08 pattern %p", miq[2]);
    miq[1].sort() with (-item);
    $display("T|09 sort %p", miq[1]);
    miq[1].reverse();
    $display("T|10 reverse %p", miq[1]);
    $display("T|11 sum=%0d min=%p max=%p", miq[1].sum(), miq[1].min(), miq[1].max());
    $display("T|12 all %p", miq);
    begin int n = 0; foreach (miq[i, j]) n++; $display("T|13 foreach2 n=%0d", n); end
    begin int n = 0; foreach (miq[1][j]) n += miq[1][j]; $display("T|14 foreach elem sum=%0d", n); end
    $display("T|15 inside=%0d %0d", 3 inside {miq[1]}, 99 inside {miq[1]});
    miq[1].insert(0, 100); miq[1].delete(1);
    $display("T|16 ins/del %p", miq[1]);
    $display("T|17 pop=%0d %0d size=%0d", miq[1].pop_front(), miq[1].pop_back(), miq[1].size());
    mdq[1] = new[3];
    mdq[1][2] = 9;
    mdq[0] = mdq[1];
    $display("T|18 dyn %p %p", mdq[0], mdq[1]);
    maq[1]["a"] = 1; maq[1]["b"] = 2;
    maq[0] = maq[1];
    $display("T|19 aa num=%0d %p", maq[0].num(), maq[0]);
    $display("T|20 eq=%0d", miq[0] == miq[0]);
    miq[0].delete();
    $display("T|21 del size=%0d", miq[0].size());
    $display("T|22 uniq=%p", miq[2].unique());
    $finish;
  end
endmodule
