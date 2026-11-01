// top: t8_25pa
// §8.25/§6.20.2 (issue #423): class-scope ARRAY constants of a class — a
// header VALUE parameter with unpacked dimensions and a class localparam
// array, each read through the specialization qualifier and from static
// methods of the specialization — plus a static array property whose
// initializer must reach the per-element cells.
package p825pa;
  class wq_c #(
    int unsigned N = 4,
    int unsigned TBL [4] = '{1, 2, 3, 4}
  );
    localparam int unsigned LP [4] = '{N, N+1, N+2, N+3};
    static function int unsigned lp(int i);
      return LP[i];
    endfunction
    static function int unsigned tbl(int i);
      return TBL[i];
    endfunction
  endclass

  class sa_c;
    static int unsigned ARR [3] = '{10, 20, 30};
    static function int unsigned a(int i);
      return ARR[i];
    endfunction
  endclass
endpackage

module t8_25pa;
  initial begin
    $display("T|8.25|spec LP %0d %0d", p825pa::wq_c#(4, '{9, 8, 7, 6})::LP[0], p825pa::wq_c#(4, '{9, 8, 7, 6})::LP[3]);
    $display("T|8.25|spec fn %0d %0d", p825pa::wq_c#(4, '{9, 8, 7, 6})::lp(1), p825pa::wq_c#(4, '{9, 8, 7, 6})::tbl(3));
    $display("T|8.25|default %0d %0d", p825pa::wq_c#()::LP[3], p825pa::wq_c#()::tbl(3));
    $display("T|8.25|static arr %0d %0d %0d", p825pa::sa_c::ARR[1], p825pa::sa_c::a(2), $size(p825pa::sa_c::ARR));
  end
endmodule