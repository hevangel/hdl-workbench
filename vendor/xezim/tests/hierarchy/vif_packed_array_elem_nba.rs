//! §25.8 / §7.4.1 — a NONBLOCKING write to an ELEMENT of a packed array
//! reached through a virtual interface kept only the RHS's least significant
//! bit.
//!
//! `infer_lhs_width`'s Index arm walks the chained index nodes down to a root
//! and then asks the packed-dimension maps how wide the selected element is —
//! but it only accepted an `Ident` root. For `vif.data[p]` the root is a
//! `MemberAccess`, so the whole block was skipped and the lvalue fell through
//! to the 1-bit default. The NBA scheduler resizes its value with
//! `val.resize_for_assign(w)` BEFORE queueing, so the flit was truncated to
//! one bit on the way into the queue and no amount of correctness at the
//! commit site could recover it.
//!
//! Only this combination broke. The blocking form never consults
//! `infer_lhs_width`; a module-scope array and a hierarchical `i0.d[i] <= ...`
//! both have an `Ident` root and were always right. That is what made it look
//! like a virtual-interface binding bug rather than a width-inference one.
//!
//! Found on a FlooNoC router UVM bench, where the driver's
//! `vif.data_in[p] <= item.as_flit();` delivered an all-zero 81-bit flit. The
//! router then routed every input to the same port (`dst_id` 0), which
//! `XYRouteOpt` ties off, and nothing was ever forwarded.
//!
//! Expected values are the reference simulator's.

use xezim::simulate;

fn out(src: &str) -> String {
    let sim = simulate(src, 400).expect("simulate failed");
    sim.output
        .iter()
        .map(|o| o.message.as_str())
        .collect::<Vec<_>>()
        .join("\n")
}

const SRC: &str = r#"
package p;
  typedef struct packed { logic [7:0] a; logic [7:0] b; } s_t;
endpackage

interface itf;
  import p::*;
  logic [4:0][15:0] dl;   // packed 2-D
  s_t   [4:0]       ds;   // packed array of a packed STRUCT, same bit count
endinterface

class Drv;
  virtual itf vif;
  function new(virtual itf v); vif = v; endfunction
  task nba_dyn(int i, logic [15:0] val);
    vif.dl[i] <= val;
    vif.ds[i] <= val;
  endtask
  task nba_const(logic [15:0] val);
    vif.dl[2] <= val;
    vif.ds[2] <= val;
  endtask
  task blk_dyn(int i, logic [15:0] val);
    vif.dl[i] = val;
    vif.ds[i] = val;
  endtask
endclass

module top;
  itf i0();
  logic [4:0][15:0] m;
  int k = 2;
  Drv d;
  initial begin
    d = new(i0);

    i0.dl = '0; i0.ds = '0; #1; d.nba_dyn(2, 16'hbeef); #1;
    $display("VIF_NBA_DYN %h %h", i0.dl, i0.ds);

    i0.dl = '0; i0.ds = '0; #1; d.nba_const(16'hbeef); #1;
    $display("VIF_NBA_CONST %h %h", i0.dl, i0.ds);

    i0.dl = '0; i0.ds = '0; #1; d.blk_dyn(2, 16'hbeef); #1;
    $display("VIF_BLK %h %h", i0.dl, i0.ds);

    m = '0; #1; m[k] <= 16'hbeef; #1;
    $display("MOD_NBA %h", m);

    i0.dl = '0; #1; i0.dl[k] <= 16'hbeef; #1;
    $display("HIER_NBA %h", i0.dl);
    $finish;
  end
endmodule
"#;

/// The regression: an NBA through a vif must write the whole ELEMENT.
#[test]
fn vif_nba_element_write_keeps_its_full_width() {
    let o = out(SRC);
    assert!(
        o.contains("VIF_NBA_DYN 00000000beef00000000 00000000beef00000000"),
        "vif NBA with a dynamic index must write all 16 bits:\n{}",
        o
    );
    assert!(
        o.contains("VIF_NBA_CONST 00000000beef00000000 00000000beef00000000"),
        "a constant index is the same lvalue shape:\n{}",
        o
    );
}

/// The forms that already worked must keep working — the width question is
/// now asked about a name the vif path resolves, so a regression here would
/// mean the Ident root stopped being answered.
#[test]
fn blocking_module_and_hierarchical_element_writes_are_unchanged() {
    let o = out(SRC);
    for expect in [
        "VIF_BLK 00000000beef00000000 00000000beef00000000",
        "MOD_NBA 00000000beef00000000",
        "HIER_NBA 00000000beef00000000",
    ] {
        assert!(o.contains(expect), "expected {expect:?} in:\n{}", o);
    }
}
