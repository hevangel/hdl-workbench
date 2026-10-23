//! §6.17, §9.4.2 and §15.5: named-event timestamp interning and cached
//! trigger plans preserve identity, time slots and notification consumers.
//! Expected T| lines were recorded from the reference simulator.
use std::process::Command;

const DESIGN: &str = r#"
`timescale 1ns/1ns
module probe_stamp;
  event pulse, alias_event;
  int edge_hits = 0, level_hits = 0;
  initial begin
    alias_event = pulse;
    $display("T|stamp initial=%0d alias=%0d", pulse.triggered, alias_event.triggered);
    fork
      begin @pulse; edge_hits++; end
      begin wait(pulse.triggered); level_hits++; end
    join_none
    #0;
    ->pulse;
    ->alias_event;
    $display("T|stamp active=%0d qualified=%0d alias=%0d hits=%0d/%0d", pulse.triggered,
      tb.stamp.pulse.triggered, alias_event.triggered, edge_hits, level_hits);
    #0;
    $display("T|stamp delta=%0d hits=%0d/%0d", pulse.triggered, edge_hits, level_hits);
    #1;
    $display("T|stamp next=%0d alias=%0d", pulse.triggered, alias_event.triggered);
  end
endmodule
module probe_alias;
  event first, second, handle_event;
  int first_hits = 0, second_hits = 0, or_hits = 0;
  initial begin
    handle_event = first;
    fork
      forever begin @first; first_hits++; end
      forever begin @second; second_hits++; end
      forever begin @(first or second); or_hits++; end
    join_none
    #2;
    ->>handle_event;
    handle_event = second;
    #0;
    $display("T|alias inactive first=%0d second=%0d hits=%0d/%0d/%0d",
      first.triggered, second.triggered, first_hits, second_hits, or_hits);
    #1;
    $display("T|alias deferred=%0d/%0d/%0d same=%0d", first_hits, second_hits, or_hits, handle_event == second);
    ->handle_event;
    #1;
    handle_event = null;
    ->handle_event;
    #1;
    $display("T|alias final=%0d/%0d/%0d null=%0d", first_hits, second_hits, or_hits, handle_event == null);
  end
endmodule
module pulse_leaf;
  event pulse;
  int hits = 0;
  always @(pulse) hits++;
endmodule
module probe_hierarchy;
  pulse_leaf left(), right();
  int bare_hits = 0, qualified_hits = 0;
  initial begin
    fork
      begin @(left.pulse); bare_hits++; end
      begin @(tb.hier.left.pulse); qualified_hits++; end
    join_none
    #1 ->left.pulse;
    #1 ->tb.hier.right.pulse;
    #1 ->tb.hier.left.pulse;
    #1;
    $display("T|hier counts=%0d/%0d waits=%0d/%0d", left.hits, right.hits, bare_hits, qualified_hits);
  end
endmodule
module probe_automatic;
  event first_pulse, second_pulse;
  task automatic activation(input int tag, ref event local_pulse);
    @local_pulse;
    $display("T|automatic tag=%0d stamp=%0t", tag, $time);
  endtask
  initial fork
    activation(1, first_pulse);
    activation(2, second_pulse);
    begin #3 ->first_pulse; #2 ->second_pulse; end
  join
endmodule
module probe_comb_clock;
  logic clk = 0;
  always #2 clk = ~clk;
  event pulse;
  logic triggered_flag;
  int property_hits = 0;
  always_comb triggered_flag = pulse.triggered;
  initial begin @(pulse.triggered); property_hits++; end
  clocking cb @(posedge clk);
    default input #0;
    input triggered_flag;
  endclocking
  initial begin
    #1;
    $display("T|comb before=%0d hits=%0d", triggered_flag, property_hits);
    #1 ->pulse;
    #0;
    $display("T|comb same=%0d stamp=%0d", triggered_flag, pulse.triggered);
    #1;
    $display("T|comb next=%0d stamp=%0d hits=%0d", triggered_flag, pulse.triggered, property_hits);
    #4 ->pulse;
    #1;
    $display("T|comb final=%0d stamp=%0d hits=%0d", triggered_flag, pulse.triggered, property_hits);
  end
  initial repeat (3) begin
    @(cb);
    $display("T|clock t=%0t flag=%0d", $time, cb.triggered_flag);
  end
endmodule
module tb;
  probe_stamp stamp();
  probe_alias aliases();
  probe_hierarchy hier();
  class notifier;
    event pulse;
    static event shared_pulse;
    task fire(); ->pulse; endtask
  endclass
  notifier first, second;
  int first_hits = 0, second_hits = 0, shared_hits = 0;
  initial begin
    first = new(); second = new();
    fork
      begin @(first.pulse); first_hits++; end
      begin wait(second.pulse.triggered); second_hits++; end
      begin @(notifier::shared_pulse); shared_hits++; end
    join_none
    #1 first.fire();
    #1 ->second.pulse;
    #1 ->notifier::shared_pulse;
    #1;
    $display("T|class counts=%0d/%0d/%0d expired=%0d/%0d", first_hits, second_hits, shared_hits,
      first.pulse.triggered, notifier::shared_pulse.triggered);
  end
  probe_automatic locals();
  probe_comb_clock comb();
  initial #13 $finish;
endmodule
"#;

const EXPECTED: &str = "T|stamp initial=0 alias=0
T|stamp active=1 qualified=1 alias=1 hits=0/0
T|stamp delta=1 hits=1/1
T|comb before=0 hits=0
T|stamp next=0 alias=0
T|alias inactive first=0 second=0 hits=0/0/0
T|comb same=1 stamp=1
T|clock t=2 flag=1
T|alias deferred=0/1/1 same=1
T|comb next=1 stamp=0 hits=1
T|automatic tag=1 stamp=3
T|hier counts=2/1 waits=1/1
T|class counts=1/1/1 expired=0/0
T|alias final=0/2/2 null=1
T|automatic tag=2 stamp=5
T|clock t=6 flag=1
T|comb final=1 stamp=0 hits=1
T|clock t=10 flag=1";

fn run(profile: bool) {
    static SEQ: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);
    let directory = std::env::temp_dir().join(format!(
        "xezim_named_event_intern_{}_{}",
        std::process::id(),
        SEQ.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
    ));
    std::fs::create_dir_all(&directory).unwrap();
    let source = directory.join("design.sv");
    std::fs::write(&source, DESIGN).unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_xezim"))
        .args(["--simulate", "--no-cache", "--max-time", "50ns", "-s", "tb"])
        .arg(&source)
        .env("XEZIM_JIT", "0")
        .env_remove("XEZIM_TS_JIT")
        .env("XEZIM_VERBOSE", if profile { "1" } else { "0" })
        .env("XEZIM_PROFILE_REPORT", if profile { "1" } else { "0" })
        .output()
        .expect("run event interning fixture");
    let _ = std::fs::remove_dir_all(directory);
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        output.status.success(),
        "event fixture failed:\n{stdout}\n{stderr}"
    );
    let mut actual: Vec<_> = stdout
        .lines()
        .filter(|line| line.starts_with("T|"))
        .collect();
    let mut expected: Vec<_> = EXPECTED.lines().collect();
    // Independent processes may print in either order within one timestamp;
    // retain duplicates while comparing the complete reference multiset.
    actual.sort_unstable();
    expected.sort_unstable();
    assert_eq!(actual, expected, "profile={profile}\n{stdout}\n{stderr}");
    if profile {
        assert!(
            stderr.contains("[PROF] settle_calls="),
            "profile did not run:\n{stderr}"
        );
    }
}

#[test]
fn named_event_plans_preserve_reference_identity_and_triggered_consumers() {
    run(false);
    run(true);
}
