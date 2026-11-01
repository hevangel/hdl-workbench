// §11.8.1: one unsigned operand makes an expression unsigned; an unsized
// unbased literal ('0, '1, 'x, 'z) is unsigned (§5.7.1). §11.8.2: the type
// reaches the context-determined operands (§11.6.1) before they are
// evaluated, so a division, a modulus, an arithmetic shift or a sign
// extension in the other arm of a conditional, or in a sibling operand,
// runs unsigned. Concatenation members, cast operands, call arguments and
// exponents are self-determined and keep their own type.
class holder;
  int p = 0;
  logic signed [15:0] s = -50;
  // Every operand is signed (§11.8.1), so s + 1 sign-extends; p holds a
  // bitwise result, which must not make the sum unsigned.
  function int sum(int j);
    p = j & 255;
    return p + (s + 1);
  endfunction
endclass
module top;
  logic signed [7:0] a = -10, b = 3; // a is 8'hf6, 246 unsigned
  logic signed [7:0] e = -2, f = 127;
  logic [7:0] u = 100;
  logic [15:0] u16 = 1000;
  logic z0 = 0, z1 = 1, y1 = 1, zx = 1'bx;
  logic signed [95:0] wa = -10, wb = 3, wc = 1;
  int calls = 0;
  int errors = 0;
  function automatic logic signed [7:0] num();
    calls++;
    return a;
  endfunction
  // Compiled processes: continuous assignments and always_comb.
  wire [7:0] c_else = z0 ? '0 : (a / b);
  wire [7:0] c_then = z1 ? (a / b) : '0;
  wire [7:0] c_ones = z0 ? '1 : (a / b);
  wire [7:0] c_x = z0 ? 'x : (a / b);
  wire [7:0] c_z = z0 ? 'z : (a / b);
  wire [7:0] c_sized = z0 ? 8'd0 : (a / b);
  wire [7:0] c_var = z0 ? u : (a / b);
  wire [7:0] c_mod = z0 ? '0 : (a % b);
  wire [7:0] c_outer = z0 ? '0 : (y1 ? (a / b) : 8'sd1);
  wire [7:0] c_inner = z0 ? (y1 ? '0 : a) : (a / b);
  wire [15:0] c_extend = z0 ? '0 : (a + b);
  wire [7:0] c_shift = z0 ? '0 : (a >>> 1);
  wire c_cmp = (a / b) > u;
  wire c_cmp_cond = (z0 ? '0 : a) < b;
  wire c_cmp_add = (a + b) < u16;
  wire [7:0] c_sum = u + a / b;
  wire [7:0] c_signed = z0 ? 8'sd0 : (a / b);
  wire [7:0] c_int = z0 ? 0 : (a / b);
  wire [15:0] c_concat = {u, a / b};
  wire [7:0] c_power = u + (8'sd2 ** (e / f));
  logic signed [95:0] c_wide;
  always_comb c_wide = z0 ? '0 : (wa / wb / wc);
  task automatic ck(string label, logic [95:0] got, logic [95:0] want);
    if (got !== want) begin
      errors++;
      $display("FAIL %s: got %h, want %h", label, got, want);
    end else $display("PASS %s", label);
  endtask
  initial begin
    #1;
    ck("continuous else arm", c_else, 82);
    ck("continuous then arm", c_then, 82);
    ck("continuous '1 arm", c_ones, 82);
    ck("continuous 'x arm", c_x, 82);
    ck("continuous 'z arm", c_z, 82);
    ck("continuous sized unsigned arm", c_sized, 82);
    ck("continuous unsigned variable arm", c_var, 82);
    ck("continuous modulus", c_mod, 0);
    ck("continuous outer conditional", c_outer, 82);
    ck("continuous inner conditional", c_inner, 82);
    ck("continuous zero extension", c_extend, 16'h00f9);
    ck("continuous arithmetic shift", c_shift, 8'h7b);
    ck("continuous comparison", c_cmp, 0);
    ck("continuous comparison conditional", c_cmp_cond, 0);
    ck("continuous comparison extension", c_cmp_add, 1);
    ck("continuous sibling operand", c_sum, 8'hb6);
    ck("continuous signed arms", c_signed, 8'hfd);
    ck("continuous int literal arm", c_int, 8'hfd);
    ck("continuous concatenation member", c_concat, 16'h64fd);
    ck("continuous exponent", c_power, 8'd101);
    ck("always_comb 96-bit", c_wide, 96'h5555_5555_5555_5555_5555_5552);
    // Procedural: the same expressions in an initial block.
    begin logic [7:0] v; v = z0 ? '0 : (a / b); ck("else arm", v, 82); end
    begin logic [7:0] v; v = z1 ? (a / b) : '0; ck("then arm", v, 82); end
    begin logic [7:0] v; v = z0 ? '1 : (a / b); ck("'1 arm", v, 82); end
    begin logic [7:0] v; v = z0 ? 'x : (a / b); ck("'x arm", v, 82); end
    begin logic [7:0] v; v = z0 ? 8'd0 : (a / b); ck("sized unsigned arm", v, 82); end
    begin logic [7:0] v; v = z0 ? 'd0 : (a / b); ck("unsized based arm", v, 82); end
    begin logic [7:0] v; v = z0 ? u : (a / b); ck("unsigned variable arm", v, 82); end
    begin logic [7:0] v; v = z0 ? '0 : (a % b); ck("modulus", v, 0); end
    begin logic [7:0] v; v = z0 ? '0 : (y1 ? (a / b) : 8'sd1); ck("outer conditional", v, 82); end
    begin logic [7:0] v; v = z0 ? (y1 ? '0 : a) : (a / b); ck("inner conditional", v, 82); end
    begin logic [15:0] v; v = z0 ? '0 : a; ck("zero extension", v, 16'h00f6); end
    begin logic [15:0] v; v = z0 ? '0 : (a + b); ck("nested zero extension", v, 16'h00f9); end
    begin logic [15:0] v; v = z0 ? u : (a + b); ck("variable arm extension", v, 16'h00f9); end
    begin logic [7:0] v; v = z0 ? '0 : (a >>> 1); ck("arithmetic shift", v, 8'h7b); end
    begin logic v; v = (a / b) > u; ck("comparison", v, 0); end
    begin logic v; v = (z0 ? '0 : a) < b; ck("comparison conditional", v, 0); end
    begin logic v; v = (a + b) < u16; ck("comparison extension", v, 1); end
    begin logic v; v = (z0 ? '0 : (a / b)) == 8'd82; ck("comparison equality", v, 1); end
    begin logic [7:0] v; v = u + a / b; ck("sibling operand", v, 8'hb6); end
    begin logic [15:0] v; v = u16 + (a + b); ck("sibling sum", v, 16'd1249); end
    begin logic [15:0] v; v = u16 + (a / b); ck("sibling quotient", v, 16'd1082); end
    begin logic [15:0] v; v = u16 + -a; ck("sibling negation", v, 16'd754); end
    begin logic [7:0] v; v = (a / b) | '0; ck("unbased sibling", v, 82); end
    begin logic [7:0] v; v = z0 ? 8'sd0 : (a / b); ck("signed arms", v, 8'hfd); end
    begin logic [7:0] v; v = z0 ? 0 : (a / b); ck("int literal arm", v, 8'hfd); end
    begin logic [7:0] v; v = a / b; ck("signed quotient", v, 8'hfd); end
    begin logic [15:0] v; v = {u, a / b}; ck("concatenation member", v, 16'h64fd); end
    begin logic [15:0] v; v = {z0 ? '0 : (a / b), u}; ck("concatenated conditional", v, 16'h5264); end
    begin logic [7:0] v; v = z0 ? '0 : 8'($signed(a / b)); ck("cast operand", v, 8'hfd); end
    // §11.6.1: the exponent is self-determined, so e / f divides signed (0).
    begin logic [7:0] v; v = u + (8'sd2 ** (e / f)); ck("exponent", v, 8'd101); end
    begin holder h; h = new; ck("class property", h.sum(7), -42); end
    begin logic signed [95:0] v; v = z0 ? '0 : (wa / wb / wc); ck("96-bit", v, 96'h5555_5555_5555_5555_5555_5552); end
    // §11.4.11: an x condition merges both arms bit by bit, at the unsigned type.
    begin logic [7:0] v; v = zx ? '0 : (a / b); ck("x condition", v, 8'b0x0x_00x0); end
    // The type probe reads the arms' declared types and never calls.
    begin logic [7:0] v; calls = 0; v = z0 ? '0 : (num() / b); ck("call operand", v, 82); ck("call count", calls, 1); end
    if (errors == 0) $display("ALL PASS"); else $display("FAILED %0d", errors);
    $finish;
  end
endmodule
