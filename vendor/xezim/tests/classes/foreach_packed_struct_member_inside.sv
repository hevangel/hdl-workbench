// Self-test: a `foreach` over a rand FIXED ARRAY OF PACKED STRUCTS whose
// body constrains a MEMBER of each element — the exact shape the 5446 reg-
// map config uses:
//
//     rand uvm_reg_map_cfg_t cfg[2][5];
//     constraint legal {
//       foreach (cfg[x,y]) {
//         cfg[x][y].au_bytes inside {1,2,4,8,16};
//         cfg[x][y].endian  inside {UVM_LITTLE_ENDIAN};
//       }
//     }
//
// Xezim's fixed-array foreach repair (`solve_forced_array_elem`) only
// matched a BARE element target (`cfg[y]`): `index_chain_root_is` is blind
// to the `MemberAccess` wrapping the `Index`, so `cfg[y].au_bytes inside`
// fell through to the generate-and-test fallback, which drew the WHOLE
// element uniformly and could never land a 32-bit member in {1,2,4,8,16} —
// randomize() returned 0 the way it did for 5446. This test asserts every
// element's member lands in-range.
module top;
  typedef enum logic [1:0] { NO=0, LITTLE=1, BIG=2 } endian_e;
  typedef struct packed {
    int unsigned au_bytes;
    endian_e endian;
    bit byte_addressing;
  } cfg_t;

  class blk;
    rand cfg_t cfg[5];
    constraint legal {
      foreach (cfg[y]) {
        cfg[y].au_bytes inside {1,2,4,8,16};
        cfg[y].endian inside {LITTLE};
      }
    }
  endclass

  initial begin
    blk b = new();
    if (!b.randomize()) begin $display("TAG_FAIL rand"); $finish; end
    foreach (b.cfg[y]) begin
      if (!(b.cfg[y].au_bytes inside {1,2,4,8,16})) begin
        $display("TAG_FAIL au y=%0d au=%0d", y, b.cfg[y].au_bytes);
        $finish;
      end
      if (!(b.cfg[y].endian inside {LITTLE})) begin
        $display("TAG_FAIL endian y=%0d endian=%0d", y, b.cfg[y].endian);
        $finish;
      end
    end
    $display("TAG_PASS");
  end
endmodule