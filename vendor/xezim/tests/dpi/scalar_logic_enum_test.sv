/* 4-state scalar svLogic marshalling, enum-typed DPI prototypes, and
 * open arrays of scalar logic. Distilled from the DPI-C conformance
 * report on C-to-SV unknown propagation: every direction of a 1-bit
 * `logic` port must carry the svLogic cell (2=z, 3=x), an enum
 * prototype binds to its base C type, and a scalar-logic open array
 * round-trips 1-byte cells. Expected values verified against the
 * reference simulator. */
typedef enum int  { RED = 1, GREEN = 2, BLUE = 3 } color_e;
typedef enum byte { BOK = 100 } byte_e;

module tb;
  import "DPI-C" function void    sle_out_x     (output logic b);
  import "DPI-C" function void    sle_out_z     (output logic b);
  import "DPI-C" function logic   sle_ret_x     ();
  import "DPI-C" function int     sle_sees      (input logic b);
  import "DPI-C" function int     sle_io_sees   (inout  logic b);
  import "DPI-C" function color_e sle_next_color(input color_e c);
  import "DPI-C" function byte_e  sle_byte_enum ();
  import "DPI-C" function void    sle_color_out (output color_e c);
  import "DPI-C" function int     sle_oa_entry  (input  logic o []);
  import "DPI-C" function int     sle_oa_cells  (output logic o []);

  int f = 0;
  logic lo;
  logic o  [0:3];
  logic iv [0:3];
  color_e ce;
  `define CK(e,m) if(!(e)) begin f++; $display("FAIL %s", m); end

  initial begin
    /* C -> SV 4-state: output formals and the return are svLogic cells. */
    sle_out_x(lo);   `CK(lo === 1'bx, "out_x")
    sle_out_z(lo);   `CK(lo === 1'bz, "out_z")
    lo = sle_ret_x(); `CK(lo === 1'bx, "ret_x")

    /* SV -> C: the cell an input/inout formal receives keeps x/z. */
    `CK(sle_sees(1'bx) == 3, "in_x")
    `CK(sle_sees(1'bz) == 2, "in_z")
    `CK(sle_sees(1'b1) == 1, "in_1")
    lo = 1'bx;
    `CK(sle_io_sees(lo) == 3, "io_in_x")
    `CK(lo === 1'bz, "io_zback")

    /* Enum prototypes bind (35.5.5): base-type mapping both ways. */
    `CK(sle_next_color(GREEN) == BLUE, "enum_roundtrip")
    `CK(sle_next_color(BLUE) == RED, "enum_wrap")
    `CK(int'(sle_byte_enum()) == -56, "enum_byte_trunc")
    sle_color_out(ce);
    `CK(ce == BLUE, "enum_out")

    /* Open arrays of scalar logic: 1-byte svLogic cells each way. */
    iv[0] = 1'bx; iv[1] = 1'bz; iv[2] = 1'b1; iv[3] = 1'b0;
    `CK(sle_oa_entry(iv) == 3, "oa_in_cell")
    `CK(sle_oa_cells(o) == 0, "oa_out_zero_init")
    `CK(o[0] === 1'bx, "oa_cell_x")
    `CK(o[1] === 1'bz, "oa_cell_z")
    `CK(o[2] === 1'b1, "oa_cell_1")
    `CK(o[3] === 1'b0, "oa_cell_0")

    if (f == 0) $display("TEST_PASS");
    else        $display("TEST_FAIL count=%0d", f);
    $finish;
  end
endmodule
