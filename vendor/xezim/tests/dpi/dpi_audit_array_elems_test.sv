// DPI audit, family C: array formals of enums, packed structs, `integer`,
// `time` and implicitly typed vectors take their base type's canonical C
// element (enum and packed-struct arrays were left unbound; integer / time
// elements went as plain ints), and an assignment-pattern actual passes
// every element (only the last one used to arrive). The expected values are
// the reference simulator's, loading this same C file.
module dpi_audit_array_elems;
  typedef enum int { E1 = 1, E2, E3 } e_t;
  typedef struct packed { logic [3:0] a; logic [3:0] b; } ps_t;
  import "DPI-C" function int ael_get(input int i);
  import "DPI-C" function int ael_enum(input e_t h[]);
  import "DPI-C" ael_vec = function void ael_ps(input ps_t h[]);
  import "DPI-C" ael_vec = function void ael_integer(input integer h[]);
  import "DPI-C" ael_fixed_vec = function void ael_impl(input [7:0] a[2]);
  import "DPI-C" ael_fixed_vec = function void ael_integer2(input integer a[2]);
  import "DPI-C" function void ael_time_out(output time t[2]);
  import "DPI-C" function int ael_ints(input int a[2]);
  import "DPI-C" function int ael_open(input int a[]);
  import "DPI-C" function int ael_2d(input int a[][]);
  int f = 0;
  `define CK(e, m) if (!(e)) begin f++; $display("FAIL %s", m); end
  e_t e[3]; ps_t ps[2]; integer ig[2]; time tt[2];
  initial begin
    e = '{E3, E1, E2}; `CK(ael_enum(e) == 312, "enum elements")
    ps = '{8'h1x, 8'hz2}; ael_ps(ps);
    `CK(ael_get(0) == 32'h1f && ael_get(1) == 32'h0f && ael_get(2) == 32'h02 && ael_get(3) == 32'hf0, "packed struct elements")
    ig = '{32'h0000_00xz, -2}; ael_integer(ig);
    `CK(ael_get(0) == 32'hf0 && ael_get(1) == 32'hff && ael_get(2) == -2 && ael_get(3) == 0, "integer open elements")
    ael_integer2(ig);
    `CK(ael_get(0) == 32'hf0 && ael_get(1) == 32'hff && ael_get(2) == -2 && ael_get(3) == 0, "integer fixed elements")
    ael_time_out(tt); `CK(tt[0] === 64'b10x && tt[1] == 64'h0000_0000_ffff_ffff, "time elements out")
    ael_impl('{8'h1z, 8'h2x});
    `CK(ael_get(0) == 32'h10 && ael_get(1) == 32'h0f && ael_get(2) == 32'h2f && ael_get(3) == 32'h0f, "implicit vector pattern")
    `CK(ael_ints('{3, 4}) == 34, "pattern to a fixed int array")
    `CK(ael_open('{1, 2, 3}) == 12303, "pattern to an open array")
    `CK(ael_2d('{'{1, 2, 3}, '{4, 5, 6}}) == 12345623, "nested pattern to a 2-D open array")
    if (f == 0) $display("TEST_PASS"); else $display("TEST_FAIL count=%0d", f);
    $finish;
  end
endmodule
