// Group H: class-scope typedef reached through a specialization
class holder #(type T = int, int P = 4);
  typedef T elem_t;
  typedef logic [P-1:0] vec_t;
endclass
typedef holder #(byte, 6) h_t;
class via #(type R = int);
  typedef R::elem_t et_t;
  function int w1(); et_t e; return $bits(e); endfunction
  typedef R::vec_t vt_t;
  function int w2(); vt_t v; return $bits(v); endfunction
  function int s1(); et_t e = -1; return e; endfunction
endclass
class user;
  function int w3(); h_t::elem_t e; return $bits(e); endfunction
  function int w4(); holder #(shortint)::elem_t e; return $bits(e); endfunction
  function int w5(); h_t::vec_t v; return $bits(v); endfunction
endclass
module top;
  h_t::elem_t e1;
  holder #(shortint)::elem_t e2;
  h_t::vec_t v1;
  holder #(int, 11)::vec_t v2;
  initial begin
    via #(h_t) a = new();
    via #(holder #(longint, 3)) b = new();
    user u = new();
    $display("T|H01 module td::elem_t:%0d", $bits(e1));
    $display("T|H02 module spec::elem_t:%0d", $bits(e2));
    $display("T|H03 module td::vec_t:%0d", $bits(v1));
    $display("T|H04 module spec::vec_t:%0d", $bits(v2));
    $display("T|H05 class typedef R::elem_t (td):%0d", a.w1());
    $display("T|H06 class typedef R::vec_t (td):%0d", a.w2());
    $display("T|H07 class typedef R::elem_t (spec):%0d", b.w1());
    $display("T|H08 class typedef R::vec_t (spec):%0d", b.w2());
    $display("T|H09 method local td::elem_t:%0d", u.w3());
    $display("T|H10 method local spec::elem_t:%0d", u.w4());
    $display("T|H11 method local td::vec_t:%0d", u.w5());
    $display("T|H12 class typedef R::elem_t signed (td):%0d", a.s1());
    $display("T|H13 class typedef R::elem_t signed (spec):%0d", b.s1());
  end
endmodule
