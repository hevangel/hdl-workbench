//! §9.4.2 / §14.13: same-slot clocking waits retain a suspended task's
//! locals and receiver (#335, reported by urbanij). The issue supplies the
//! expected time 15; existing clocking skew/iff tests establish the same-slot
//! and sampled-guard ordering used by the neighboring shapes.

use xezim::simulate;

fn messages(src: &str) -> Vec<String> {
    let sim = simulate(src, 100).expect("simulate");
    assert!(
        !sim.output.iter().any(|o| o.message.contains("[fatal]")),
        "unexpected fatal: {:?}",
        sim.output.iter().map(|o| &o.message).collect::<Vec<_>>()
    );
    sim.output
        .iter()
        .map(|o| o.message.trim().to_owned())
        .filter(|m| m.starts_with("PASS") || m.starts_with("T|") || m == "item == null")
        .collect()
}

const ORIGINAL: &str = r#"
interface pulse_if (input logic clk);
  clocking cb @(posedge clk);
  endclocking
endinterface

class item_c;
  int unsigned pulse_cycles = 1;
endclass

class driver_c;
  virtual pulse_if vif;
  task run(item_c item_in);
    item_c item;
    item = item_in;
    @(posedge vif.clk);
    @(vif.cb);
    if (item == null) $display("item == null");
    repeat (item.pulse_cycles) @(posedge vif.clk);
    $display("PASS at %0t", $time);
  endtask
endclass

module top;
  logic clk = 0;
  always #5 clk = ~clk;

  pulse_if pif (clk);

  initial begin
    driver_c d = new();
    item_c it = new();
    d.vif = pif;
    d.run(it);
    $finish;
  end
endmodule
"#;

#[test]
fn task_local_handle_survives_same_slot_clocking_wait() {
    assert_eq!(messages(ORIGINAL), ["PASS at 15"]);
}

#[test]
fn task_formals_properties_and_scalars_survive_clocking_wait() {
    let cases = [
        (
            "formal",
            ORIGINAL.replace(
                "repeat (item.pulse_cycles)",
                "repeat (item_in.pulse_cycles)",
            ),
            15,
        ),
        (
            "property",
            ORIGINAL.replace(
                "  task run(item_c item_in);\n    item_c item;",
                "  item_c item;\n  task run(item_c item_in);",
            ),
            15,
        ),
        (
            "scalar",
            ORIGINAL
                .replace("    item_c item;", "    item_c item;\n    int n;")
                .replace(
                    "    item = item_in;",
                    "    item = item_in;\n    n = item.pulse_cycles;",
                )
                .replace("repeat (item.pulse_cycles)", "repeat (n)"),
            15,
        ),
        (
            "inactive",
            ORIGINAL.replace("    @(vif.cb);", "    #0;\n    @(vif.cb);"),
            15,
        ),
        (
            "future",
            ORIGINAL.replace("    @(vif.cb);", "    #1;\n    @(vif.cb);"),
            25,
        ),
        ("raw_only", ORIGINAL.replace("    @(vif.cb);\n", ""), 15),
        (
            "clocking_only",
            ORIGINAL.replacen("    @(posedge vif.clk);\n", "", 1),
            15,
        ),
        (
            "two_raw",
            ORIGINAL.replace("    @(vif.cb);", "    @(posedge vif.clk);"),
            25,
        ),
    ];
    for (name, src, time) in cases {
        assert_eq!(messages(&src), [format!("PASS at {time}")], "{name}");
    }
}

#[test]
fn clocking_guard_keeps_task_context_when_true_or_rearmed() {
    for (init, time) in [
        ("logic gate = 1;", 15),
        ("logic gate = 0; initial #12 gate = 1;", 25),
    ] {
        let src = ORIGINAL
            .replace("  clocking cb", &format!("  {init}\n  clocking cb"))
            .replace("  endclocking", "    input #0 gate;\n  endclocking")
            .replace(
                "    @(vif.cb);",
                "    @(vif.cb iff (item.pulse_cycles == 1 && vif.cb.gate));",
            );
        assert_eq!(messages(&src), [format!("PASS at {time}")]);
    }
}

#[test]
fn fork_join_waits_for_clocking_task_to_finish() {
    let src = ORIGINAL.replace(
        "    d.run(it);",
        r#"
    fork
      begin d.run(it); end
      begin #1; end
    join
    $display("T|joined at %0t", $time);"#,
    );
    assert_eq!(messages(&src), ["PASS at 15", "T|joined at 15"]);
}

fn cancellation_source(wait: &str, launch: &str) -> String {
    format!(
        r#"
interface bus_if(input logic clk);
  clocking cb @(posedge clk); endclocking
endinterface
module top;
  logic clk = 0;
  always #5 clk = ~clk;
  bus_if bus(clk);
  int resumed = 0;
  process handle;
  task automatic worker;
    int value = 42;
    @(posedge clk);
    {wait}
    resumed = value;
  endtask
  {launch}
  initial begin #30; $display("T|resumed=%0d", resumed); $finish; end
endmodule
"#
    )
}

#[test]
fn canceled_clocking_tasks_do_not_resume() {
    // #0 runs before clocking continuations in Reactive. Both the ready
    // continuation and the still-guarded wait must be canceled. The guard
    // stays false so stale re-arming would survive beyond this slot.
    let launches = [
        r#"initial begin
          fork worker(); join_none
          @(posedge clk); #0; disable fork;
        end"#,
        r#"initial begin
          fork : pending
            worker();
            begin @(posedge clk); #0; disable pending; end
          join
        end"#,
        r#"initial begin
          fork begin handle = process::self(); worker(); end join_none
          @(posedge clk); #0; handle.kill();
        end"#,
    ];
    for wait in ["@(bus.cb);", "@(bus.cb iff 1'b0);"] {
        for launch in launches {
            assert_eq!(
                messages(&cancellation_source(wait, launch)),
                ["T|resumed=0"]
            );
        }
    }
}

#[test]
fn remote_disable_unwinds_a_block_parked_on_clocking() {
    for wait in ["@(bus.cb);", "@(bus.cb iff 1'b1);"] {
        let src = cancellation_source(
            wait,
            &format!(
                r#"
        initial begin
          begin : pending
            int value = 42;
            @(posedge clk);
            {wait}
            resumed = value;
          end
          $display("T|after_disable=%0t", $time);
        end
        initial begin @(posedge clk); #0; disable pending; end
        "#
            ),
        );
        assert_eq!(messages(&src), ["T|after_disable=5", "T|resumed=0"]);
    }
}
