module top;
  class C;
    static logic [7:0] z = 8'h40;
    function void inst_sel();
      z[0] = 1'b1;
    endfunction
    static function void stat_sel();
      z[7:6] = 2'b11;
    endfunction
  endclass
  initial begin
    C o;
    o = new;
    o.inst_sel();
    if (C::z === 8'h41) $display("PASS method bit-select"); else $display("FAIL method bit-select: C::z = %h", C::z);
    C::stat_sel();
    if (C::z === 8'hC1) $display("PASS static method part-select"); else $display("FAIL static method part-select: C::z = %h", C::z);
    C::z[1] = 1'b1;
    if (C::z === 8'hC3) $display("PASS scoped bit-select"); else $display("FAIL scoped bit-select: C::z = %h", C::z);
  end
endmodule
