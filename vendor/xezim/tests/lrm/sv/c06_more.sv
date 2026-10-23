// top: c06x
module tp #(parameter type T = int, parameter T DEF = 0, parameter int ARR [3] = '{1, 2, 3}, parameter W = 8);
  T v = DEF;
  logic [W-1:0] wv = '1;
  initial #1 begin $display("T|6.20.3|%m %0d bits=%0d w=%0d", v, $bits(T), $bits(wv)); $display("T|6.20.2|%m arr=%p", ARR); end
endmodule
module c06x;
  tp #(.T(byte), .DEF(-3)) t1();
  tp #(.T(logic [11:0]), .DEF(12'hfff), .ARR('{7, 8, 9})) t2();
  tp #(.W(3)) t3();
  // 6.8 var keyword
  var logic [3:0] vv = 4'h3;
  var vimp = 1'b1;
  // 6.20.6 const
  const int CI = 42;
  // 6.11.3 signed/unsigned with real
  shortreal sr;
  real r;
  // 6.12.1 real conversions
  int i;
  logic [63:0] l64;
  // 6.19.5 enum in expressions, out of range values
  typedef enum logic [2:0] {IDLE = 3'b000, RUN = 3'b011, STOP = 3'b111} st_t;
  st_t st;
  // 6.19 enum with x value
  typedef enum logic [1:0] {XA = 2'b00, XB = 2'bx1} xe_t;
  // 6.25 parameterized data types via class
  class PT #(parameter int N = 2); typedef logic [N-1:0] vec_t; endclass
  PT#(5)::vec_t pv;
  // 6.22.1 matching types / 6.22.2 equivalence: assign between differently named equivalent unpacked arrays
  typedef int ia3_t [3];
  int other3 [0:2];
  ia3_t a3;
  initial begin
    #2;
    $display("T|6.8a|%h %b %0d", vv, vimp, $bits(vimp));
    $display("T|6.20.6|%0d", CI);
    sr = 1.0 / 3.0; r = sr; $display("T|6.12a|%0.10f", r);
    r = 1e40; sr = r; $display("T|6.12b|%e", sr);
    r = 3.7; i = r; $display("T|6.12.1a|%0d", i);
    r = -3.5; i = r; $display("T|6.12.1b|%0d", i);
    r = 1e20; l64 = r; $display("T|6.12.1c|%0d", l64);
    r = 2.0**40; i = r; $display("T|6.12.1d|%0d", i);
    l64 = 64'hFFFF_FFFF_FFFF_FFFF; r = l64; $display("T|6.12.1e|%e", r);
    r = 'bx; $display("T|6.12.1f|%f", r);
    st = RUN; $display("T|6.19a|%s %b", st.name(), st);
    st = st_t'(3'b001); $display("T|6.19b|[%s] %b", st.name(), st);
    i = st + RUN; $display("T|6.19c|%0d", i);
    begin xe_t x = XB; $display("T|6.19d|%b %s", x, x.name()); end
    $display("T|6.25|%0d", $bits(pv));
    other3 = '{4, 5, 6}; a3 = other3; $display("T|6.22|%p", a3);
    // 6.24.1 casting with size from parameter and type
    $display("T|6.24.1k|%h %h", 6'(-1), st_t'(2));
    $display("T|6.24.1l|%0d", type(i)'(3.9));
    // 6.24 implicit truncation of real to logic
    begin logic [3:0] l4; l4 = 17.6; $display("T|6.24m|%0d", l4); end
    // signed cast of x
    $display("T|6.24.1n|%0d", signed'(4'b1x00));
    // 6.11.3 signed integer types
    begin byte unsigned bu = 200; int unsigned iu = 32'hffffffff; longint unsigned lu = -1; integer unsigned igu = -1;
      $display("T|6.11.3|%0d %0d %0d %0d", bu, iu, lu, igu); end
    // 6.3 4-state vs 2-state assignment
    begin bit [3:0] b4 = 4'b1x0z; int i2 = 'x; $display("T|6.3|%b %0d", b4, i2); end
    // 6.14 chandle default null, comparison
    begin chandle c1, c2; $display("T|6.14b|%0d %0d", c1 == c2, c1 == null); end
    // 6.16 string to integral, and %s of int
    begin string s = "AB"; logic [15:0] l16 = 16'h4142; int i3; i3 = s.len(); $display("T|6.16w|%0d %s", i3, string'(l16)); end
  end
endmodule
