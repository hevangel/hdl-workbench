// DPI audit, family A: unpacked arrays of scalar bit / logic carry one
// svBit / svLogic byte per element (§35.5.6.1, Annex H.7.4), x and z
// included. The expected values are the reference simulator's, loading this
// same C file.
module dpi_audit_scalar_arrays;
  import "DPI-C" function int asa_get(input int i);
  import "DPI-C" function void asa_fixed(input logic a[4], input bit b[3], output logic o[3]);
  import "DPI-C" function void asa_open(input logic h[]);
  import "DPI-C" function int asa_bits(input bit h[]);
  import "DPI-C" function void asa_put(output logic h[]);
  int f = 0;
  `define CK(e, m) if (!(e)) begin f++; $display("FAIL %s", m); end
  logic la[4]; bit ba[3]; logic lo[3]; logic lq[1:4];
  initial begin
    la = '{1'b0, 1'b1, 1'bz, 1'bx}; ba = '{1'b1, 1'b0, 1'b1};
    asa_fixed(la, ba, lo);
    `CK(asa_get(0) == 0 && asa_get(1) == 1 && asa_get(2) == 2 && asa_get(3) == 3, "fixed logic in")
    `CK(asa_get(4) == 1 && asa_get(5) == 0 && asa_get(6) == 1, "fixed bit in")
    `CK(lo[0] === 1'bx && lo[1] === 1'b1 && lo[2] === 1'bz, "fixed logic out")
    lq = '{1'b1, 1'bz, 1'bx, 1'b0}; asa_open(lq);
    `CK(asa_get(0) == 1 && asa_get(1) == 4, "open dims / size")
    `CK(asa_get(2) == 1 && asa_get(3) == 2 && asa_get(4) == 3 && asa_get(5) == 0, "open elements")
    `CK(asa_get(6) == 1, "open array pointer")
    `CK(asa_bits(ba) == 101, "open bit accessor")
    asa_put(lq); `CK(lq[1] === 1'b0 && lq[2] === 1'b1 && lq[3] === 1'bz && lq[4] === 1'bx, "open put")
    if (f == 0) $display("TEST_PASS"); else $display("TEST_FAIL count=%0d", f);
    $finish;
  end
endmodule
