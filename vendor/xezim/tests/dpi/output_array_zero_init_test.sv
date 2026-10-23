/* An output array formal starts as a zeroed buffer and is copied back
 * whole: elements C does not write read 0 afterwards, and C reads 0 on
 * entry. An inout array keeps the actual. Expected values are the
 * reference simulator's. */
module tb;
  import "DPI-C" function void oaz_ifix (output int a[4]);
  import "DPI-C" function void oaz_lfix (output logic a[4]);
  import "DPI-C" function void oaz_oa_i (output int o[]);
  import "DPI-C" function void oaz_oa_l (output logic o[]);
  import "DPI-C" function void oaz_oa_l8(output logic [7:0] o[]);
  import "DPI-C" function void oaz_oa_2d(output logic o[][]);
  import "DPI-C" function int  oaz_entry(output int o[]);
  import "DPI-C" task          oaz_task (output int o[]);
  import "DPI-C" function void oaz_io   (inout int o[]);

  int f = 0;
  int ia[4], ib[4], ic[4], id[4], ie[4];
  int dy[];
  int ex[4];
  logic la[4], lb[4];
  logic [7:0] l8[4];
  logic l2[2][2];
  `define CK(e,m) if(!(e)) begin f++; $display("FAIL %s", m); end

  initial begin
    ia = '{5, 5, 5, 5}; oaz_ifix(ia);
    ex = '{9, 0, 0, 0}; `CK(ia == ex, "fixed int")
    la = '{1'b1, 1'b1, 1'bz, 1'b1}; oaz_lfix(la);
    `CK(la[0] === 1'bx && la[1] === 1'b0 && la[2] === 1'b0 && la[3] === 1'b0, "fixed logic")
    ib = '{5, 5, 5, 5}; oaz_oa_i(ib);
    ex = '{9, 0, 0, 0}; `CK(ib == ex, "open int")
    lb = '{1'b1, 1'b1, 1'bz, 1'b1}; oaz_oa_l(lb);
    `CK(lb[0] === 1'bx && lb[1] === 1'b0 && lb[2] === 1'b0 && lb[3] === 1'b0, "open logic")
    l8 = '{8'h11, 8'h22, 8'h33, 8'h44}; oaz_oa_l8(l8);
    `CK(l8[0] === 8'h77 && l8[1] === 8'h00 && l8[2] === 8'h00 && l8[3] === 8'h00, "open logic [7:0]")
    l2[0][0] = 1'b1; l2[0][1] = 1'b1; l2[1][0] = 1'bz; l2[1][1] = 1'bz; oaz_oa_2d(l2);
    `CK(l2[0][0] === 1'bx && l2[0][1] === 1'b0 && l2[1][0] === 1'b0 && l2[1][1] === 1'b1, "open 2-D")
    ic = '{5, 5, 5, 5};
    `CK(oaz_entry(ic) == 0, "entry value")
    ex = '{0, 4, 0, 0}; `CK(ic == ex, "entry writeback")
    dy = new[3]; dy = '{5, 5, 5};
    `CK(oaz_entry(dy) == 0 && dy.size() == 3 && dy[0] == 0 && dy[1] == 4 && dy[2] == 0, "dynamic")
    id = '{5, 5, 5, 5}; oaz_task(id);
    ex = '{0, 7, 0, 0}; `CK(id == ex, "task")
    ie = '{5, 5, 5, 5}; oaz_io(ie);
    ex = '{9, 5, 5, 5}; `CK(ie == ex, "inout keeps the actual")

    if (f == 0) $display("TEST_PASS");
    else        $display("TEST_FAIL count=%0d", f);
    $finish;
  end
endmodule
