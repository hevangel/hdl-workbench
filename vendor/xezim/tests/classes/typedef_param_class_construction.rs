//! §8.25 / §6.18 — `new` through a typedef alias of a parameterized class
//! specialization (`typedef ev_c#(int) ev_t; ev_t g; g = new;`) must build
//! that specialization. Only non-parameterized aliases were followed, so a
//! module-scope or procedural variable stayed null (every call through it was
//! dropped and every property read x), and a class property initialized
//! inline through such an alias (UVM's `uvm_event_pool events = new(...)` in
//! each transaction) was built with the DEFAULT type argument, so the pool
//! handed out plain objects whose `wait_trigger()` returned at once. The
//! expected lines are the reference simulator's output.

use xezim::simulate;

fn lines(src: &str) -> Vec<String> {
    simulate(src, 1000)
        .expect("simulate failed")
        .output
        .iter()
        .map(|o| o.message.clone())
        .collect()
}

#[test]
fn typedef_alias_of_specialization_constructs_it() {
    let src = r#"
virtual class ev_base;
  protected event m_event;
  int k = 7;
  virtual task wait_trigger();
    @m_event;
  endtask
endclass
class ev_c #(type T = int) extends ev_base;
  T last;
  function new(string name = ""); endfunction
  virtual function void trigger(T data = 0);
    last = data;
    ->m_event;
  endfunction
endclass
typedef ev_c#(int) ev_t;
module top;
  ev_t g, a;
  initial begin
    g = new;
    a = new("a");
    $display("g.k=%0d a.k=%0d", g.k, a.k);
    #10 g.trigger(1); a.trigger(2);
    #10 g.trigger(3); a.trigger(4);
    #10 g.trigger(5); a.trigger(6);
  end
  initial begin
    ev_t b;
    #5;
    g.wait_trigger();
    $display("%0t g woke last=%0d", $time, g.last);
    b = a;
    b.wait_trigger();
    $display("%0t a woke last=%0d", $time, a.last);
  end
endmodule
"#;
    assert_eq!(
        lines(src),
        vec!["g.k=7 a.k=7", "10 g woke last=1", "20 a woke last=4"]
    );
}

#[test]
fn typedef_alias_property_initializer_uses_the_specialization() {
    let src = r#"
class ev_c #(type T = int);
  T last;
  event m_event;
  function new(string name = ""); endfunction
  function void trigger(T data = 0);
    last = data;
    ->m_event;
  endfunction
  task wait_trigger();
    @m_event;
  endtask
endclass
class box #(type T = int);
  T items[string];
  function new(string name = ""); endfunction
  function T get(string key);
    if (!items.exists(key))
      items[key] = new(key);
    return items[key];
  endfunction
endclass
typedef box #(ev_c#(int)) ev_box;
class holder;
  ev_box events = new("events");
endclass
module top;
  holder h;
  initial begin
    h = new;
    #10 h.events.get("done").trigger(1);
    #10 h.events.get("done").trigger(2);
  end
  initial begin
    ev_c#(int) e;
    #10;
    e = h.events.get("done");
    e.wait_trigger();
    $display("%0t woke last=%0d", $time, h.events.get("done").last);
  end
endmodule
"#;
    assert_eq!(lines(src), vec!["20 woke last=2"]);
}
