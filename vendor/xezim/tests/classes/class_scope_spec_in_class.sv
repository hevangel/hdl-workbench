// Group J: bare no-argument static function named inside the class chain
class plainc;
  static function int f(); return 41; endfunction
  function int g(); return f + 1; endfunction
  static function int sg(); return f + 2; endfunction
endclass
class holder #(type T = int, int N = 4);
  static function int nval(); return N; endfunction
  function int g(); return nval; endfunction
  static function int sg(); return nval; endfunction
endclass
class kid extends plainc;
  function int k(); return f; endfunction
endclass
module top;
  initial begin
    plainc p = new();
    holder #(int, 7) h = new();
    kid k = new();
    $display("T|J01 plain bare static in method:%0d", p.g());
    $display("T|J02 plain bare static in static:%0d", plainc::sg());
    $display("T|J03 spec bare static in method:%0d", h.g());
    $display("T|J04 spec bare static in static:%0d", holder #(int, 7)::sg());
    $display("T|J05 inherited plain bare static in method:%0d", k.k());
  end
endmodule
