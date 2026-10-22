// Self-test: WHOLE-STRUCT cross-element equalities in a rand FIXED 2-D
// ARRAY OF PACKED STRUCTS — the 5446 reg-map coupling:
//
//     rand uvm_reg_map_cfg_t cfg[2][5];
//     constraint legal {
//       cfg[0][0] == cfg[1][0];
//       cfg[0][4] == cfg[1][4];
//     }
//
// Xezim's `solve_forced` only wrote a rand COLLECTION element when exactly
// ONE side of `==` was an element key (and `coll_elem_expr_key` does not
// even peel a 2-D fixed-array element), so a whole-struct equality between
// two array elements fell through to the generate-and-test backstop. On the
// preceding member `inside` repair this made randomize() return 0 for 5446.
// The Eq arm now copies RHS's element into LHS's when BOTH sides are whole
// array elements.
module top;
  typedef enum { NO=0, LITTLE=1, BIG=2 } endian_e;
  typedef struct packed {
    int unsigned au_bytes;
    endian_e endian;
    bit byte_addressing;
  } cfg_t;

  class blk;
    rand cfg_t cfg[2][5];
    constraint legal {
      foreach (cfg[x,y]) {
        cfg[x][y].au_bytes inside {1,2,4,8,16};
      }
      cfg[0][0] == cfg[1][0];
      cfg[0][4] == cfg[1][4];
      cfg[0][0].au_bytes == 1;
      cfg[0][4].au_bytes == 1;
      cfg[0][0].byte_addressing == 1;
      cfg[0][4].byte_addressing == 1;
    }
  endclass

  initial begin
    blk b = new();
    if (!b.randomize()) begin $display("TAG_FAIL rand"); $finish; end
    if (b.cfg[0][0] !== b.cfg[1][0]) begin $display("TAG_FAIL eq00"); $finish; end
    if (b.cfg[0][4] !== b.cfg[1][4]) begin $display("TAG_FAIL eq04"); $finish; end
    if (b.cfg[0][0].au_bytes !== 1 || b.cfg[0][4].au_bytes !== 1) begin
      $display("TAG_FAIL eq au"); $finish; end
    if (b.cfg[0][0].byte_addressing !== 1 || b.cfg[0][4].byte_addressing !== 1) begin
      $display("TAG_FAIL ba"); $finish; end
    $display("TAG_PASS");
  end
endmodule