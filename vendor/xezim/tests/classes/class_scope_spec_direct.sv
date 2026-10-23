// Group D: same members reached DIRECTLY (no type parameter), for comparison
typedef enum {RED, GREEN, BLUE} color_e;
class holder #(type T = int, string Tname = "<unknown>", int P = 4);
  localparam int LP = P * 2;
  typedef T elem_t;
  static int count = 11;
  static function string type_name(); return Tname; endfunction
  static function int num(); return P + 100; endfunction
  static function color_e col(); return BLUE; endfunction
  static task bump(); count++; endtask
endclass
typedef holder #(byte, "dt", 6) h_t;
module top;
  h_t::elem_t e;
  initial begin
    $display("T|D01 spec bare string:%s", holder #(byte, "dd", 6)::type_name);
    $display("T|D02 spec bare int:%0d", holder #(byte, "dd", 6)::num);
    $display("T|D03 spec static prop:%0d", holder #(byte, "dd", 6)::count);
    $display("T|D04 spec param P:%0d", holder #(byte, "dd", 6)::P);
    $display("T|D05 spec localparam LP:%0d", holder #(byte, "dd", 6)::LP);
    $display("T|D06 td bare string:%s", h_t::type_name);
    $display("T|D07 td bare int:%0d", h_t::num);
    $display("T|D08 td static prop:%0d", h_t::count);
    $display("T|D09 td param P:%0d", h_t::P);
    $display("T|D10 td localparam LP:%0d", h_t::LP);
    h_t::bump;
    $display("T|D11 td bare task:%0d", h_t::count);
    h_t::bump();
    $display("T|D12 td paren task:%0d", h_t::count);
    $display("T|D13 td typedef elem_t bits:%0d", $bits(e));
    $display("T|D14 td paren string:%s", h_t::type_name());
  end
endmodule

