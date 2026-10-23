class db #(type T = int);
  static int cnt;
endclass
typedef int intq_t[$];
typedef int intq2_t[$];
typedef intq_t intq3_t;
typedef int intd_t[];
typedef int intd2_t[];
typedef int f3a_t[3];
typedef int f3b_t[0:2];
typedef int f3c_t[2:0];
typedef int aa_t[string];
typedef int aa2_t[string];
typedef byte byq_t[$];
typedef bit [31:0] bq_t[$];
typedef int signed isq_t[$];
class mon #(type T = int);
  typedef T tq_t[$];
  typedef T td_t[];
  function int n(); return db#(tq_t)::cnt; endfunction
  function int d(); return db#(td_t)::cnt; endfunction
endclass
package pk; typedef int pq_t[$]; endpackage
module tb;
  initial begin
    mon#(int) m = new;
    mon#(byte) mb = new;
    db#(intq_t)::cnt++;
    $display("T|q2|%0d", db#(intq2_t)::cnt);
    $display("T|q3|%0d", db#(intq3_t)::cnt);
    $display("T|pkg|%0d", db#(pk::pq_t)::cnt);
    $display("T|isq|%0d", db#(isq_t)::cnt);
    $display("T|bq|%0d", db#(bq_t)::cnt);
    $display("T|byq|%0d", db#(byq_t)::cnt);
    $display("T|cls|%0d", m.n());
    $display("T|cls_b|%0d", mb.n());
    db#(intd_t)::cnt += 5;
    $display("T|dyn|%0d %0d", db#(intd2_t)::cnt, m.d());
    db#(f3a_t)::cnt += 7;
    $display("T|fix|%0d %0d", db#(f3b_t)::cnt, db#(f3c_t)::cnt);
    db#(aa_t)::cnt += 9;
    $display("T|aa|%0d", db#(aa2_t)::cnt);
    $finish;
  end
endmodule
