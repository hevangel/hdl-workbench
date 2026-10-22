// top: c05
module c05;
  logic [15:0] v16;
  logic [63:0] v64;
  logic [99:0] v100;
  logic signed [7:0] s8;
  int i;
  real r;
  string s;
  (* full_case, my_attr = 3 *) logic attrv;
  logic \bus+index ;
  logic \escaped_id ;
  logic _under$dollar;
  initial begin
    // 5.7.1 integer literals
    v16 = 'hFF; $display("T|5.7.1a|%h", v16);
    v16 = 8'hFFF; $display("T|5.7.1b|%h", v16);       // truncation
    v16 = 4'b1; $display("T|5.7.1c|%b", v16);
    v16 = 'bx; $display("T|5.7.1d|%b", v16);           // x extends
    v16 = 'bz1; $display("T|5.7.1e|%b", v16);
    v16 = 4'bx1; $display("T|5.7.1f|%b", v16);         // sized, zero-extended above
    v16 = 12'hz_A; $display("T|5.7.1g|%b", v16);
    v64 = 'hFFFF_FFFF_FFFF; $display("T|5.7.1h|%h", v64);
    v64 = 'b1 << 40; $display("T|5.7.1i|%h", v64);
    v100 = 'hx; $display("T|5.7.1j|%h", v100);
    v100 = 100'd1 << 99; $display("T|5.7.1k|%h", v100);
    s8 = -8'd3; $display("T|5.7.1l|%0d %b", s8, s8);
    s8 = 8'sd200; $display("T|5.7.1m|%0d", s8);
    i = -'d5; $display("T|5.7.1n|%0d", i);
    i = 'sd5 - 8; $display("T|5.7.1o|%0d", i);
    $display("T|5.7.1p|%0d %0d", -4'sd1, 4'shf);
    $display("T|5.7.1q|%0d", 32'd4294967295 + 1);
    $display("T|5.7.1r|%b", 3'o7);
    $display("T|5.7.1s|%h", 16'h?);
    $display("T|5.7.1t|%0d", 1_000_000);
    $display("T|5.7.1u|%0d", 'd12);
    $display("T|5.7.1v|%b", 5'd3 ? 2'b10 : 2'b01);
    // unbased unsized
    v16 = '1; $display("T|5.7.1w|%h", v16);
    v100 = '1; $display("T|5.7.1x|%h", v100);
    v16 = 'x; $display("T|5.7.1y|%h", v16);
    v16 = 'z; $display("T|5.7.1z|%h", v16);
    v16 = '0; $display("T|5.7.1A|%h", v16);
    $display("T|5.7.1B|%0d", $bits('1));
    v16 = {'1, 4'h0}; $display("T|5.7.1C|%h", v16); // '1 in concat is 1 bit
    v64 = 64'(signed'('1)); $display("T|5.7.1D|%h", v64);
    // 5.7.2 real literals
    r = 1.5e3; $display("T|5.7.2a|%f", r);
    r = 2E-2; $display("T|5.7.2b|%e", r);
    r = 1_000.000_5; $display("T|5.7.2c|%f", r);
    r = 0.1; $display("T|5.7.2d|%0.17g", r);
    i = 2.5; $display("T|5.7.2e|%0d", i);  // rounds away from zero -> 3
    i = -2.5; $display("T|5.7.2f|%0d", i);
    i = 1.4999; $display("T|5.7.2g|%0d", i);
    v16 = 3.5; $display("T|5.7.2h|%0d", v16);
    // 5.8 time literals
    $display("T|5.8|%0t %0t", 1ns, 2.5ns);
    // 5.9 strings
    s = "a\tb\\c\"d\x41\101\n"; $display("T|5.9a|%s|len=%0d", s, s.len());
    s = "abc\
def"; $display("T|5.9b|%s", s);
    v16 = "ab"; $display("T|5.9c|%h", v16);
    v16 = "a"; $display("T|5.9d|%h", v16);
    v64 = "hello"; $display("T|5.9e|%s|%h", v64, v64);
    $display("T|5.9f|%0d", $bits("abc"));
    s = "x\vy\fz\a"; $display("T|5.9g|%0d", s.len());
    // 5.6 identifiers
    \bus+index = 1; \escaped_id = 0; _under$dollar = 1;
    $display("T|5.6.1|%b %b %b", \bus+index , escaped_id, _under$dollar);
    // 5.12 attributes ignored semantically
    attrv = 1'b0 | (* opt *) 1'b1;
    $display("T|5.12|%b", attrv);
    // 5.10/5.11 structure and array literals
    begin
      int arr[3] = '{1, 2, 3};
      struct {int a; byte b;} st = '{a:5, b:6};
      $display("T|5.11|%p %p", arr, st);
    end
    /* block comment // nested line */ // line /* */
    $display("T|5.4|comments ok");
  end
endmodule
