//! §8.9, §15.5: a static class `event` property is one object shared by all
//! instances. A wait on it (`@ev` in a static method, `@(C::ev)`) had no
//! backing signal and woke at once instead of at the next `->`. Expected
//! values are the reference simulator's.

use xezim::simulate;

fn t_lines(src: &str) -> Vec<String> {
    let sim = simulate(src, 1_000).expect("simulate failed");
    let mut v: Vec<String> = sim
        .output
        .iter()
        .map(|o| o.message.clone())
        .filter(|m| m.starts_with("T|"))
        .collect();
    v.sort();
    v
}

/// `@ev` in a static task (bare, `C::` and inline) and from a module.
#[test]
fn static_event_waits_in_static_tasks() {
    const SRC: &str = r#"class B;
  static event ev;
  static int cnt;
  static task e(); @ev; endtask
  static task e2(); @(ev); endtask
  static task re(); e(); $display("T|bare t=%0t", $time); endtask
  static task rs(); B::e(); $display("T|scoped t=%0t", $time); endtask
  static task ri(); @ev; $display("T|inline t=%0t", $time); endtask
endclass
module tb;
  int x;
  initial begin
    fork B::re(); B::rs(); B::ri(); begin B::e(); $display("T|mod t=%0t", $time); end join_none
    #4 x = 1;
    #8 -> B::ev;
    #5 $finish;
  end
endmodule
"#;
    assert_eq!(
        t_lines(SRC),
        [
            "T|bare t=12",
            "T|inline t=12",
            "T|mod t=12",
            "T|scoped t=12",
        ]
    );
}

/// `@(C::ev)` from a module, a static task, and an instance event control.
#[test]
fn static_event_from_module_and_instance_event() {
    const SRC: &str = r#"class B;
  static event ev;
  event iev;
  static int flag;
  static task e(); @ev; endtask
  task ie(); @iev; $display("T|inst-ev t=%0t", $time); endtask
  static task w(); wait (flag == 1); $display("T|wait t=%0t", $time); endtask
  static task tr(); -> ev; endtask
endclass
module tb;
  B h;
  initial begin
    h = new();
    fork
      begin @(B::ev); $display("T|mod-scoped t=%0t", $time); end
      begin B::e(); $display("T|mod-call t=%0t", $time); end
      h.ie();
      B::w();
    join_none
    #4 B::flag = 1;
    #8 B::tr();
    #1 -> h.iev;
    #5 $finish;
  end
endmodule
"#;
    assert_eq!(
        t_lines(SRC),
        [
            "T|inst-ev t=13",
            "T|mod-call t=12",
            "T|mod-scoped t=12",
            "T|wait t=4",
        ]
    );
}

/// `wait(ev.triggered)`, `->>` and an inherited static event.
#[test]
fn static_event_triggered_and_nonblocking() {
    const SRC: &str = r#"class B;
  static event ev;
  static task tw(); wait (ev.triggered); $display("T|trig-wait t=%0t", $time); endtask
  static task nb(); ->> ev; endtask
  static task w2(); @ev; $display("T|nb-woke t=%0t", $time); endtask
endclass
class D extends B;
  static task w3(); @ev; $display("T|derived t=%0t", $time); endtask
endclass
module tb;
  initial begin
    fork B::tw(); B::w2(); D::w3(); join_none
    #6 B::nb();
    #0 $display("T|triggered-now=%0d t=%0t", B::ev.triggered, $time);
    #5 $finish;
  end
endmodule
"#;
    assert_eq!(
        t_lines(SRC),
        [
            "T|derived t=6",
            "T|nb-woke t=6",
            "T|trig-wait t=6",
            "T|triggered-now=0 t=6",
        ]
    );
}
