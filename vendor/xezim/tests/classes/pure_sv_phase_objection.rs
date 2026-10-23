//! Self-checking test verifying pure SystemVerilog process control,
//! associative array keys/deletion, and phase/objection synchronization
//! without relying on any Rust-side UVM interceptors or shims.

use xezim::simulate;

#[test]
fn test_pure_sv_phase_objection() {
    const SRC: &str = r#"
module top;
  class ProcTest;
    task run();
      fork
        begin
          #10;
          $display("TAG_PROC_FAIL");
        end
        begin
          #5;
          $display("TAG_PROC_KILL");
        end
      join_any
      disable fork;
    endtask
  endclass

  class Item;
    string id;
    function new(string i); id = i; endfunction
  endclass

  class Map;
    bit presence[Item];
  endclass

  typedef enum { UNINIT, EXECUTING, ENDED, DONE } pstate_e;

  class SimplePhase;
    pstate_e state = UNINIT;
    int obj_count = 0;

    function void raise_obj();
      obj_count++;
    endfunction

    function void drop_obj();
      obj_count--;
    endfunction

    task execute();
      state = EXECUTING;
      $display("PHASE_STATE=%s", state.name());
      wait(obj_count == 0);
      state = ENDED;
      $display("PHASE_STATE=%s", state.name());
      state = DONE;
      $display("PHASE_STATE=%s", state.name());
    endtask
  endclass

  initial begin
    ProcTest pt;
    Item it1, it2;
    Map m;
    SimplePhase ph;

    pt = new();
    it1 = new("item1");
    it2 = new("item2");
    m = new();
    ph = new();

    pt.run();

    m.presence[it1] = 1;
    m.presence[it2] = 1;
    foreach (m.presence[k]) begin
      $display("MAP_KEY=%s", k.id);
    end

    m.presence.delete(it1);
    $display("MAP_SIZE_AFTER_DELETE=%0d", m.presence.num());

    ph.raise_obj();
    fork
      ph.execute();
      begin
        #20;
        ph.drop_obj();
      end
    join

    $display("FINAL_PHASE_STATE=%s", ph.state.name());
    $display("TAG_PASS");
  end
endmodule
"#;

    let sim = simulate(SRC, 100).expect("Simulation failed");

    let lines: Vec<String> = sim
        .output
        .iter()
        .map(|l| l.message.clone())
        .filter(|m| m.contains("TAG_") || m.contains("MAP_") || m.contains("PHASE_"))
        .collect();

    assert_eq!(
        lines,
        vec![
            "TAG_PROC_KILL",
            "MAP_KEY=item1",
            "MAP_KEY=item2",
            "MAP_SIZE_AFTER_DELETE=1",
            "PHASE_STATE=EXECUTING",
            "PHASE_STATE=ENDED",
            "PHASE_STATE=DONE",
            "FINAL_PHASE_STATE=DONE",
            "TAG_PASS",
        ]
    );
}
