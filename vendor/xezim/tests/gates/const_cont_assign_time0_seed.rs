//! A continuous assign with a CONSTANT right-hand side must drive its net.
//!
//! Such an entry has an EMPTY read set, so nothing ever marks it dirty: it
//! reaches the settle worklist only through the one-shot `comb_time0_idx`
//! seed, which the first settle at time 0 consumes before latching
//! `comb_time0_fired`.
//!
//! A parameterized class with a `type_id` typedef has its statics initialized
//! during elaboration ("registry spec static init"), which runs BEFORE
//! "build combinational entries". A blocking write in one of those
//! initializers reaches `settle_after_proc_write`, whose settle finds an
//! EMPTY entry list — it seeded nothing, yet retired the one-shot anyway. The
//! real time-0 settle then skipped `comb_time0_idx` entirely and every
//! constant-driven net kept its x for the whole run.
//!
//! `uvm_object_registry` is exactly this shape, so ANY UVM testbench armed
//! it: in a FlooNoC router the crossbar tie-offs (`assign masked_valid[out]
//! [v][in] = '0;`) stayed x, the output arbiters saw an x valid, and the
//! router never forwarded a flit.
//!
//! Expected values are the reference simulator's.

use xezim::simulate;

fn out(src: &str) -> String {
    let sim = simulate(src, 200).expect("simulate failed");
    sim.output
        .iter()
        .map(|o| o.message.as_str())
        .collect::<Vec<_>>()
        .join("\n")
}

/// The regression itself: an elaboration-time static-init write must not
/// retire the time-0 seed that the constant continuous assigns depend on.
#[test]
fn const_cont_assign_survives_static_init_settle() {
    const SRC: &str = r#"
module top;
  logic [4:0] tied;
  logic       poked;

  class Poker #(int W = 1);
    typedef Poker#(W) type_id;
    static int dummy = poke();
    static function int poke();
      top.poked = 1'b1;
      return W;
    endfunction
  endclass

  Poker#(3) p3;

  for (genvar i = 0; i < 5; i++) begin : g
    assign tied[i] = '0;
  end

  initial begin
    #1 $display("T %b", tied);
    $finish;
  end
endmodule
"#;
    let o = out(SRC);
    assert!(
        o.contains("T 00000"),
        "constant-RHS continuous assigns must drive:\n{}",
        o
    );
}

/// The same shape the FlooNoC crossbar uses: a packed 3-D net whose cells are
/// split between a constant tie-off and a driven expression, with the static
/// initializer in play. Only the tie-offs were lost, so a whole-vector check
/// would have passed while half the net stayed x.
#[test]
fn packed_3d_tie_offs_and_driven_cells_agree() {
    const SRC: &str = r#"
module top;
  logic [4:0][0:0][4:0] mv;
  logic [4:0][0:0]      src;
  logic                 poked;

  class Poker #(int W = 1);
    typedef Poker#(W) type_id;
    static int dummy = poke();
    static function int poke();
      top.poked = 1'b1;
      return W;
    endfunction
  endclass

  Poker#(2) p2;

  for (genvar o = 0; o < 5; o++) begin : go
    for (genvar v = 0; v < 1; v++) begin : gv
      for (genvar i = 0; i < 5; i++) begin : gi
        if (o == i) begin : gen_no_conn
          assign mv[o][v][i] = '0;
        end else begin : gen_conn
          assign mv[o][v][i] = src[i][v];
        end
      end
    end
  end

  initial begin
    src = '0;
    #1 $display("M %b", mv);
    $finish;
  end
endmodule
"#;
    let o = out(SRC);
    assert!(
        o.contains("M 0000000000000000000000000"),
        "tied cells must resolve alongside the driven ones:\n{}",
        o
    );
}
