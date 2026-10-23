// top: c35
module sub;
  int k = 222;
endmodule
module c35;
  typedef struct { int a; byte b; real r; } st_t;
  import "DPI-C" function int add_i(input int a, input int b);
  import "DPI-C" function byte add_byte(input byte a);
  import "DPI-C" function longint add_ll(input longint a);
  import "DPI-C" function real mul_r(input real a, input real b);
  import "DPI-C" function bit ret_bit(input bit b);
  import "DPI-C" function void out_args(output int o, output real r, output bit bt);
  import "DPI-C" function void inout_i(inout int io);
  import "DPI-C" function string ret_str(input string s);
  import "DPI-C" function void str_out(output string s);
  import "DPI-C" function void bitvec(input bit [39:0] i, output bit [39:0] o);
  import "DPI-C" function void logvec(input logic [7:0] i, output logic [7:0] o);
  import "DPI-C" function int sum_open(input int a[]);
  import "DPI-C" function void fill_open(inout int a[]);
  import "DPI-C" function int sum_fixed(input int a[4]);
  import "DPI-C" function void struct_in(input st_t s, output st_t o);
  import "DPI-C" function chandle mk_handle(input int v);
  import "DPI-C" function int rd_handle(input chandle h);
  import "DPI-C" pure function int pure_sq(input int x);
  import "DPI-C" context function int call_export(input int x);
  import "DPI-C" context task c_task_waits(input int n);
  import "DPI-C" context function int scope_test();
  import "DPI-C" function int bit_sel(input bit [7:0] v);
  import "DPI-C" function void put_logic(output logic [3:0] v);
  import "DPI-C" add_i = function int cname_alias(input int a, input int b);
  export "DPI-C" function sv_double;
  export "DPI-C" task sv_wait;
  export "DPI-C" function sv_scoped;
  function int sv_scoped(); return $sformatf("%m") == "c35.sv_scoped" ? 111 : 333; endfunction
  function int sv_double(int x); return 2 * x; endfunction
  task sv_wait(int n); #(n); $display("T|35.9|export task resumed t=%0t", $time); endtask
  sub sub();
  int o, io; real r; bit bt; string s; bit [39:0] bv; logic [7:0] lv, lo; int da[], fa[4]; int sa[2:5]; st_t si, so; chandle h; logic [3:0] l4;
  initial begin
    $display("T|35.5|int %0d byte %0d ll %0d real %f bit %b", add_i(3, 4), add_byte(-128), add_ll(64'h1_0000_0000), mul_r(1.5, 4.0), ret_bit(0));
    out_args(o, r, bt); $display("T|35.5.4|out %0d %f %b", o, r, bt);
    io = 5; inout_i(io); $display("T|35.5.4|inout %0d", io);
    $display("T|35.5.6|string %s", ret_str("hello")); str_out(s); $display("T|35.5.6|str out %s", s);
    bitvec(40'hF0_1234_5678, bv); $display("T|35.5.6|bitvec %h", bv);
    lv = 8'b10xz_01zx; logvec(lv, lo); $display("T|35.5.6|logvec %b", lo);
    da = '{1, 2, 3}; $display("T|35.5.6.1|open dyn %0d", sum_open(da));
    fill_open(sa); $display("T|35.5.6.1|open fill ranged %p", sa);
    fa = '{1, 2, 3, 4}; $display("T|35.5.6|fixed %0d", sum_fixed(fa));
    si = '{5, 8'd9, 1.25}; struct_in(si, so); $display("T|35.5.6|struct %0d %0d %f", so.a, so.b, so.r);
    h = mk_handle(42); $display("T|35.5.6|chandle %0d null=%0d", rd_handle(h), rd_handle(null));
    $display("T|35.5.2|pure %0d", pure_sq(9));
    $display("T|35.7|export call %0d", call_export(20));
    $display("T|35.5.4|alias %0d", cname_alias(1, 1));
    $display("T|35.5.6|bitsel %0d", bit_sel(8'b0000_1001));
    put_logic(l4); $display("T|35.5.6|put logic %b", l4);
    $display("T|35.11|scope %0d", scope_test());
    #5 c_task_waits(10); $display("T|35.9|import task returned t=%0t", $time);
  end
endmodule
