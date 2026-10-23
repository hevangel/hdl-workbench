//! IEEE 1800-2023 §9.6.2: `disable` of a named block (or task) from a
//! process forked inside it. The block's owner resumes after the block and
//! every process forked inside it, the disabling one included, ends. The
//! disable used to unwind only the disabling child. Expected values come
//! from the reference simulator.
use xezim::simulate;

fn t_lines(sim: &xezim::compiler::Simulator) -> Vec<String> {
    sim.output
        .iter()
        .map(|o| o.message.trim_end().to_string())
        .filter(|l| l.starts_with("T|"))
        .collect()
}

/// The audit repro.
#[test]
fn disable_block_from_fork_child_audit_repro() {
    const SRC: &str = r#"
module rdf;
  string lg[$];
  initial begin
    begin : blk
      fork
        begin #1 lg.push_back("H"); disable blk; end
        begin #3 lg.push_back("I"); end
      join
      lg.push_back("not_here");
    end
    #5 $display("T|r1|%p t=%0t", lg, $time);
  end
endmodule
"#;
    let sim = simulate(SRC, 1000000).expect("sim");
    assert_eq!(t_lines(&sim), ["T|r1|'{\"H\"} t=6",],);
}

/// `join`, `join_any` and `join_none`; an outer block from a nested one; an
/// inner block only; a task; a fork label from a grandchild; the block of a
/// grandparent; and a process disabling its own block.
#[test]
fn disable_enclosing_scopes_from_fork_children() {
    const SRC: &str = r#"
module dis1t;
  string lg[$];
  task automatic tk(input int id);
    fork
      begin #1 lg.push_back($sformatf("t%0d_a", id)); disable tk; end
      begin #3 lg.push_back($sformatf("t%0d_b", id)); end
    join
    lg.push_back($sformatf("t%0d_after", id));
  endtask
  initial begin
    // A: disable enclosing named block from fork child (join)
    begin : blk
      fork
        begin #1 lg.push_back("A1"); disable blk; lg.push_back("A1x"); end
        begin #3 lg.push_back("A2"); end
      join
      lg.push_back("A_not");
    end
    lg.push_back("A_after");
    $display("T|A|%p t=%0t", lg, $time); lg.delete();
    // B: join_any
    begin : blkb
      fork
        begin #1 lg.push_back("B1"); disable blkb; end
        begin #3 lg.push_back("B2"); end
      join_any
      lg.push_back("B_mid");
      #5 lg.push_back("B_not");
    end
    lg.push_back("B_after");
    #5 $display("T|B|%p t=%0t", lg, $time); lg.delete();
    // C: join_none, child disables block after parent already waiting later in block
    begin : blkc
      fork
        begin #1 lg.push_back("C1"); disable blkc; end
        begin #3 lg.push_back("C2"); end
      join_none
      lg.push_back("C_mid");
      #5 lg.push_back("C_not");
    end
    lg.push_back("C_after");
    #5 $display("T|C|%p t=%0t", lg, $time); lg.delete();
    // D: nested: disable outer block from inside inner block in fork child
    begin : outer
      begin : inner
        fork
          begin #1 lg.push_back("D1"); disable outer; end
          begin #3 lg.push_back("D2"); end
        join
        lg.push_back("D_inner_not");
      end
      lg.push_back("D_outer_not");
    end
    lg.push_back("D_after");
    #5 $display("T|D|%p t=%0t", lg, $time); lg.delete();
    // E: disable inner only (outer continues)
    begin : outer2
      begin : inner2
        fork
          begin #1 lg.push_back("E1"); disable inner2; end
          begin #3 lg.push_back("E2"); end
        join
        lg.push_back("E_inner_not");
      end
      lg.push_back("E_outer_yes");
    end
    #5 $display("T|E|%p t=%0t", lg, $time); lg.delete();
    // F: task disabled from fork child within it
    tk(1);
    lg.push_back("F_after");
    #5 $display("T|F|%p t=%0t", lg, $time); lg.delete();
    // G: disable fork label from a nested child
    fork : fl
      begin
        fork
          begin #1 lg.push_back("G1"); disable fl; end
          begin #3 lg.push_back("G2"); end
        join
        lg.push_back("G_not");
      end
      begin #4 lg.push_back("G3"); end
    join
    lg.push_back("G_after");
    #5 $display("T|G|%p t=%0t", lg, $time); lg.delete();
    // H: grandchild disables block of grandparent
    begin : blkh
      fork
        begin
          fork
            begin #1 lg.push_back("H1"); disable blkh; end
            begin #3 lg.push_back("H2"); end
          join
          lg.push_back("H_not1");
        end
        begin #3 lg.push_back("H3"); end
      join
      lg.push_back("H_not2");
    end
    lg.push_back("H_after");
    #5 $display("T|H|%p t=%0t", lg, $time); lg.delete();
    // J: same-process blocking disable (sanity)
    begin : bj
      #1 lg.push_back("J1");
      disable bj;
      lg.push_back("J_not");
    end
    lg.push_back("J_after");
    $display("T|J|%p t=%0t", lg, $time); lg.delete();
    $finish;
  end
endmodule
"#;
    let sim = simulate(SRC, 1000000).expect("sim");
    assert_eq!(
        t_lines(&sim),
        [
            "T|A|'{\"A1\", \"A_after\"} t=1",
            "T|B|'{\"B1\", \"B_after\"} t=7",
            "T|C|'{\"C_mid\", \"C1\", \"C_after\"} t=13",
            "T|D|'{\"D1\", \"D_after\"} t=19",
            "T|E|'{\"E1\", \"E_outer_yes\"} t=25",
            "T|F|'{\"t1_a\", \"F_after\"} t=31",
            "T|G|'{\"G1\", \"G_after\"} t=37",
            "T|H|'{\"H1\", \"H_after\"} t=43",
            "T|J|'{\"J1\", \"J_after\"} t=44",
        ],
    );
}

/// A block disabled by an unrelated process while its owner waits in a fork,
/// and a process disabling its own block kills the `join_none` children forked
/// inside it.
#[test]
fn disable_block_from_another_process() {
    const SRC: &str = r#"
module dis2;
  // disable a block from a separate initial (other process) while it waits in a fork
  string lg[$];
  initial begin
    begin : blk
      fork
        begin #3 lg.push_back("a"); end
        begin #5 lg.push_back("b"); end
      join
      lg.push_back("not");
    end
    lg.push_back("after");
    $display("T|K|%p t=%0t", lg, $time);
  end
  initial #2 disable dis2.blk;
  // L: join_none children spawned inside blk survive? (they are inside blk: killed)
  string l2[$];
  initial begin
    #20;
    begin : blk2
      fork
        begin #1 l2.push_back("L1"); end
        begin #5 l2.push_back("L2"); end
      join_none
      #2 disable blk2;
      l2.push_back("L_not");
    end
    l2.push_back("L_after");
    #10 $display("T|L|%p t=%0t", l2, $time);
  end
endmodule
"#;
    let sim = simulate(SRC, 1000000).expect("sim");
    assert_eq!(
        t_lines(&sim),
        ["T|K|'{\"after\"} t=2", "T|L|'{\"L1\", \"L_after\"} t=32",],
    );
}
