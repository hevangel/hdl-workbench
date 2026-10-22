package pk;
  typedef int unsigned addr_t;
  typedef addr_t aq_t [$];
  typedef int ida_t [];
  typedef struct { aq_t al; bit [6:0] at; } ent_t;
  typedef struct { aq_t al; bit [6:0] at; } map_t [string];
  typedef struct { ida_t d; int k; } dent_t;
  map_t regs = '{ "A" : '{ al: {'h10}, at: 7'h1 }, "B" : '{ al: {'h20, 'h21, 'h22}, at: 7'h2 } };
  ent_t eaa [string] = '{ "A" : '{ al: {'h30, 'h31}, at: 7'h3 } };
  aq_t all = { regs["A"].al, regs["B"].al };
  aq_t pq = {1, 2, 3};
  map_t empty_map;
endpackage

module tb;
  import pk::*;
  map_t mregs = '{ "A" : '{ al: {'h40, 'h41}, at: 7'h4 } };
  ent_t maa [string];
  ent_t mint [int];
  ent_t mq [$];
  ent_t fa [2];
  dent_t daa [string];
  aq_t lq;
  initial begin
    ent_t e;
    $display("T|01 pkg regs num=%0d A=%0d B=%0d", regs.num(), regs["A"].al.size(), regs["B"].al.size());
    $display("T|02 pkg all size=%0d %p", all.size(), all);
    $display("T|03 pkg pq %p", pq);
    $display("T|04 pkg eaa A=%0d %p", eaa["A"].al.size(), eaa["A"].al);
    $display("T|05 mod mregs A=%0d at=%0d", mregs["A"].al.size(), mregs["A"].at);
    maa["x"].al.push_back(5); maa["x"].al.push_back(6);
    $display("T|06 maa x size=%0d %p num=%0d", maa["x"].al.size(), maa["x"].al, maa.num());
    mint[3].al = {7, 8, 9};
    $display("T|07 mint size=%0d al[1]=%0d", mint[3].al.size(), mint[3].al[1]);
    e.al = {1, 2}; e.at = 3;
    mq.push_back(e);
    $display("T|08 mq al size=%0d %p", mq[0].al.size(), mq[0].al);
    fa[1].al.push_back(4);
    $display("T|09 fa al size=%0d", fa[1].al.size());
    daa["d"].d = new[3];
    daa["d"].d[2] = 5;
    $display("T|10 daa d size=%0d %p", daa["d"].d.size(), daa["d"].d);
    lq = {regs["A"].al, regs["B"].al, mregs["A"].al};
    $display("T|11 concat %p", lq);
    $display("T|12 regs %p", regs);
    foreach (regs[k]) $display("T|13 key=%s size=%0d", k, regs[k].al.size());
    foreach (regs["B"].al[i]) $display("T|14 B[%0d]=%0d", i, regs["B"].al[i]);
    regs["B"].al.push_back('h23);
    $display("T|15 B push size=%0d last=%0d", regs["B"].al.size(), regs["B"].al[$]);
    $display("T|16 empty num=%0d", empty_map.num());
    e = maa["x"];
    $display("T|17 copy-out size=%0d", e.al.size());
    maa["y"] = e;
    $display("T|18 copy-in size=%0d %p", maa["y"].al.size(), maa["y"].al);
    $display("T|19 find=%p", regs["B"].al.find with (item > 'h20));
    $display("T|20 eq=%0d", regs["A"].al == mq[0].al);
    $finish;
  end
endmodule
