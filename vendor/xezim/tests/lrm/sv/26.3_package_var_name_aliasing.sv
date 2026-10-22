// top: rpv
package pa; int cnt = 1; function int inc(); cnt++; return cnt; endfunction endpackage
package pc; int cnt = 50; endpackage
module rpv;
  initial begin
    #1;
    $display("T|r1|pa=%0d pc=%0d", pa::cnt, pc::cnt);
    void'(pa::inc());
    $display("T|r2|pa=%0d pc=%0d", pa::cnt, pc::cnt);
    pc::cnt = 7;
    $display("T|r3|pa=%0d pc=%0d", pa::cnt, pc::cnt);
  end
endmodule
