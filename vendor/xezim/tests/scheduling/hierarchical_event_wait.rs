//! GitHub #235: an event control `@(single.pulse)` / `@(bank[0].pulse)` on a named event
//! declared in an instantiated submodule or interface must wake when a process
//! inside that instance triggers `->pulse`.
//!
//! Root cause:
//! 1. In `xezim-core/src/elaborate.rs`, `rewrite_stmt` for `EventTrigger` used
//!    `format!("{}.{}", prefix, name.name)` instead of `cat2(prefix, &name.name)`.
//!    Because `prefix` ends with a dot (`"single."`), this synthesized an event name
//!    with two dots (`"single..pulse"` / `"bank[0]..pulse"`), which never matched the
//!    registered signal name (`"single.pulse"` / `"bank[0].pulse"`).
//! 2. Submodule events were never inserted into `elab.events` during inlining.
//! 3. `member_chain_as_flat_ident_impl` did not handle `ExprKind::Index`, so
//!    `bank[0].pulse` failed to normalize to a hierarchical identifier and produced
//!    empty sensitivity (spurious wake at time 0).
//! 4. `resolve_hier_name_uncached` dropped constant segment selects on
//!    hierarchical identifiers.
//!
//! Verified byte-for-byte against reference simulators.

fn single(sim: &xezim::compiler::Simulator, n: &str) -> u64 {
    sim.get_signal(n)
        .or_else(|| sim.get_signal(&format!("top.{}", n)))
        .or_else(|| sim.get_signal(&format!("deep_top.{}", n)))
        .unwrap_or_else(|| panic!("signal not found: {}", n))
        .to_u64()
        .unwrap_or_else(|| panic!("{} not u64-able (x/z?)", n))
}

#[test]
fn scalar_and_array_instance_event_wait() {
    let sim = xezim::simulate(
        r#"
module event_leaf;
  event pulse;
  initial repeat (3) #10 ->pulse;
endmodule

module top;
  event_leaf single ();
  event_leaf bank[1] ();
  int single_hits = 0, bank_hits = 0;
  initial begin
    fork
      repeat (3) begin @(single.pulse);     single_hits++;  end
      repeat (3) begin @(bank[0].pulse); bank_hits++; end
    join_none
    #100;
    $finish;
  end
endmodule
"#,
        200,
    )
    .expect("simulate");

    assert_eq!(
        single(&sim, "single_hits"),
        3,
        "@(single.pulse) must wake 3 times"
    );
    assert_eq!(
        single(&sim, "bank_hits"),
        3,
        "@(bank[0].pulse) must wake 3 times"
    );
}

#[test]
fn forever_loop_instance_event_wait() {
    let sim = xezim::simulate(
        r#"
module event_leaf;
  event pulse;
  initial repeat (3) #10 ->pulse;
endmodule

module top;
  event_leaf single ();
  event_leaf bank[1] ();
  int single_hits = 0, bank_hits = 0;
  initial begin
    fork
      forever begin @(single.pulse);     single_hits++;  end
      forever begin @(bank[0].pulse); bank_hits++; end
    join_none
    #100;
    $finish;
  end
endmodule
"#,
        200,
    )
    .expect("simulate");

    assert_eq!(
        single(&sim, "single_hits"),
        3,
        "forever @(single.pulse) must wake 3 times"
    );
    assert_eq!(
        single(&sim, "bank_hits"),
        3,
        "forever @(bank[0].pulse) must wake 3 times"
    );
}

#[test]
fn deep_arbitrary_nested_hierarchy_event_wait() {
    let sim = xezim::simulate(
        r#"
module leaf;
  event pulse;
  initial repeat (3) #10 ->pulse;
endmodule

module mid;
  leaf l ();
  leaf la[2] ();
endmodule

module deep_top;
  mid m ();
  mid ma[2] ();

  int n_m_l = 0;
  int n_m_la0 = 0;
  int n_m_la1 = 0;
  int n_ma0_l = 0;
  int n_ma1_la1 = 0;

  initial begin
    fork
      forever begin @(m.l.pulse);         n_m_l++;     end
      forever begin @(m.la[0].pulse);     n_m_la0++;   end
      forever begin @(m.la[1].pulse);     n_m_la1++;   end
      forever begin @(ma[0].l.pulse);     n_ma0_l++;   end
      forever begin @(ma[1].la[1].pulse); n_ma1_la1++; end
    join_none
    #100;
    $finish;
  end
endmodule
"#,
        200,
    )
    .expect("simulate");

    assert_eq!(single(&sim, "n_m_l"), 3, "@(m.l.pulse)");
    assert_eq!(single(&sim, "n_m_la0"), 3, "@(m.la[0].pulse)");
    assert_eq!(single(&sim, "n_m_la1"), 3, "@(m.la[1].pulse)");
    assert_eq!(single(&sim, "n_ma0_l"), 3, "@(ma[0].l.pulse)");
    assert_eq!(single(&sim, "n_ma1_la1"), 3, "@(ma[1].la[1].pulse)");
}

#[test]
fn virtual_interface_event_wait() {
    let sim = xezim::simulate(
        r#"
interface pulse_bus;
  event pulse;
  initial repeat (3) #10 ->pulse;
endinterface

class client;
  virtual pulse_bus vif;
  int count = 0;
  function new(virtual pulse_bus v);
    vif = v;
  endfunction
  task run();
    repeat (3) begin
      @(vif.pulse);
      count++;
    end
  endtask
endclass

module top;
  pulse_bus if_inst ();
  client c = new(if_inst);
  int direct_count = 0;
  int receiver_count = 0;

  initial begin
    fork
      c.run();
      repeat (3) begin
        @(if_inst.pulse);
        direct_count++;
      end
    join_none
    #100;
    receiver_count = c.count;
    $finish;
  end
endmodule
"#,
        200,
    )
    .expect("simulate");

    assert_eq!(
        single(&sim, "direct_count"),
        3,
        "direct interface event wait"
    );
    assert_eq!(
        single(&sim, "receiver_count"),
        3,
        "bound interface event wait"
    );
}
