//! §13.5 / §8.23: a blocking task called on a call result
//! (`h.events.get("done").wait_trigger()`) must suspend the caller like the
//! same call through a local handle. The receiver had no side-effect-free
//! handle form, so the call ran on the synchronous path and returned at
//! once. The receiver call must also run only once per statement (`gets`).
//! The expected lines were cross-checked against the reference simulator.

use xezim::simulate;

const SRC: &str = r#"
class ev_c;
  event m_event;
  task wait_trigger();
    @m_event;
  endtask
  function void trigger();
    ->m_event;
  endfunction
endclass
class pool_c;
  ev_c m[string];
  int gets;
  function ev_c get(string k);
    gets++;
    if (!m.exists(k)) m[k] = new;
    return m[k];
  endfunction
endclass
class holder;
  pool_c events;
  function new(); events = new; endfunction
  function pool_c get_events(); return events; endfunction
endclass
module top;
  holder h;
  initial begin
    h = new;
    #10 h.events.get("done").trigger();
    $display("%0t triggered", $time);
    #10 h.events.get("done").trigger();
    $display("%0t triggered 2", $time);
    #10 h.get_events().get("done").trigger();
    $display("%0t triggered 3", $time);
  end
  initial begin
    ev_c e;
    #1;
    h.events.get("done").wait_trigger();
    $display("%0t woke 1 (call result) gets=%0d", $time, h.events.gets);
    e = h.events.get("done");
    e.wait_trigger();
    $display("%0t woke 2 (local)", $time);
    h.get_events().get("done").wait_trigger();
    $display("%0t woke 3 (call chain) gets=%0d", $time, h.events.gets);
  end
endmodule
"#;

#[test]
fn blocking_task_on_call_result_suspends() {
    let out: Vec<String> = simulate(SRC, 1000)
        .expect("simulate failed")
        .output
        .iter()
        .map(|o| o.message.clone())
        .collect();
    let want = [
        "10 triggered",
        "10 woke 1 (call result) gets=2",
        "20 triggered 2",
        "20 woke 2 (local)",
        "30 triggered 3",
        "30 woke 3 (call chain) gets=6",
    ];
    assert_eq!(out, want, "{out:?}");
}
