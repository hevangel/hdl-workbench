//! Formal-metadata save/restore must round-trip a MODULE-scope signal whose
//! name a function FORMAL shadows.
//!
//! Calling a function snapshots, clears and restores the structural metadata
//! for each formal name (`packed_signal_elem_widths` et al.), so a struct
//! signal named like a formal has its dotted `name.member` keys removed for
//! the call's duration and reinstated after. That bookkeeping used to
//! linear-scan the whole design-sized map three times per call — ~10ms per
//! evaluation of one assign on a 55k-signal design — and is now answered by a
//! dotted-prefix index. The index invariant is that it may only
//! OVER-approximate; this test pins the dangerous direction: after a call
//! (clear + restore) the dotted keys must still be found, i.e. member selects
//! on the shadowed struct must still resolve.

use xezim::simulate;

fn notes(src: &str) -> Vec<String> {
    let sim = simulate(src, 1_000_000).expect("simulate failed");
    sim.output
        .iter()
        .map(|o| o.message.trim().to_string())
        .filter(|l| l.starts_with("NOTE:"))
        .collect()
}

const SRC: &str = r#"
module top;
  typedef struct packed {
    logic [3:0][7:0] lanes;   // member that owns a dotted elem-width key
    logic [7:0]      tag;
  } bus_t;

  // Module-scope struct whose name the formal below shadows.
  bus_t inst;

  function automatic [7:0] f(input [31:0] inst);
    f = inst[7:0] + 8'd1;
  endfunction

  initial begin
    inst.lanes[2] = 8'hAB;
    inst.tag      = 8'h5A;
    // Before any call: member select resolves.
    $display("NOTE: pre lanes2=%h tag=%h", inst.lanes[2], inst.tag);
    // The call clears + restores the metadata for the name `inst`.
    $display("NOTE: call=%h", f(32'h10));
    // After the round-trip the SAME selects must still resolve — a lost
    // dotted key here reads 0 or x.
    $display("NOTE: post lanes2=%h tag=%h", inst.lanes[2], inst.tag);
    // And a second call keeps working (index state after restore).
    $display("NOTE: call2=%h", f(32'h20));
    inst.lanes[1] = 8'hCD;
    $display("NOTE: write lanes1=%h lanes2=%h", inst.lanes[1], inst.lanes[2]);
    $finish;
  end
endmodule
"#;

#[test]
fn struct_signal_survives_shadowing_formal_call() {
    let got = notes(SRC);
    assert!(
        got.contains(&"NOTE: pre lanes2=ab tag=5a".to_string()),
        "{got:?}"
    );
    assert!(got.contains(&"NOTE: call=11".to_string()), "{got:?}");
    assert!(
        got.contains(&"NOTE: post lanes2=ab tag=5a".to_string()),
        "dotted metadata lost across the call's clear/restore: {got:?}"
    );
    assert!(got.contains(&"NOTE: call2=21".to_string()), "{got:?}");
    assert!(
        got.contains(&"NOTE: write lanes1=cd lanes2=ab".to_string()),
        "member write after the round-trip resolved wrongly: {got:?}"
    );
}
