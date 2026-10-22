//! §15.3/§15.4 blocking IPC through CLASS METHODS — reference-verified.
//! Three fixed bugs: (1) a blocking method reached through a nested handle
//! chain (`w.fifo.peek_it(x)`, flattened >=3-segment Ident) missed the
//! suspend-aware inliner and ran synchronously — blocking peek returned
//! garbage immediately; (2) put/try_put delivered to only ONE parked
//! waiter, so a peek+get pair on one mailbox deadlocked with the item in
//! the box (the uvm sequencer get_next_item/item_done shape); (3) the
//! synchronous empty-get fallthrough is now a loud one-shot warning.

use std::process::Command;

fn run(name: &str, src: &str) -> String {
    let dir = std::env::temp_dir().join(format!("xezim_mmb_{}_{}", name, std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join(format!("{name}.sv"));
    std::fs::write(&path, src).unwrap();
    let out = Command::new(env!("CARGO_BIN_EXE_xezim"))
        .args([
            "--simulate",
            "-s",
            "test",
            path.to_str().unwrap(),
            "--no-cache",
            "--max-time",
            "1000",
        ])
        .output()
        .expect("run xezim");
    format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    )
}

#[test]
fn nested_handle_peek_get_chain() {
    let text = run(
        "nested_handle_peek_get_chain",
        r#"class fifo_c;
  mailbox #(int) m;
  function new(); m = new(1); endfunction
  function bit try_put_it(int v); return m.try_put(v); endfunction
  task peek_it(output int x); m.peek(x); endtask
  task get_it(output int x); m.get(x); endtask
endclass
class wrap_c; fifo_c fifo; function new(); fifo = new(); endfunction endclass
module test;
  wrap_c w = new();
  int a, b; bit ok;
  initial begin w.fifo.peek_it(a); $display("T|peek a=%0d @%0t", a, $time); end
  initial begin w.fifo.get_it(b);  $display("T|get b=%0d @%0t", b, $time); end
  initial begin #5 ok = w.fifo.try_put_it(42); $display("T|tp=%0d @%0t", ok, $time); #10 $finish; end
endmodule"#,
    );
    assert!(text.contains("T|tp=1 @5"), "{text}");
    assert!(text.contains("T|peek a=42 @5"), "{text}");
    assert!(text.contains("T|get b=42 @5"), "{text}");
}

#[test]
fn bounded_put_blocks_and_admits() {
    let text = run(
        "bounded_put_blocks_and_admits",
        r#"class fifo_c;
  mailbox #(int) m;
  function new(); m = new(2); endfunction
  task put_it(int v); m.put(v); endtask
  task get_it(output int x); m.get(x); endtask
endclass
module test;
  fifo_c f = new();
  int x;
  initial begin
    f.put_it(1); $display("T|put1 @%0t", $time);
    f.put_it(2); $display("T|put2 @%0t", $time);
    f.put_it(3); $display("T|put3 @%0t", $time);
    $finish;
  end
  initial begin
    #10 f.get_it(x); $display("T|got %0d @%0t", x, $time);
  end
endmodule"#,
    );
    assert!(text.contains("T|put2 @0"), "{text}");
    assert!(text.contains("T|got 1 @10"), "{text}");
    assert!(text.contains("T|put3 @10"), "{text}");
}

#[test]
fn semaphore_through_methods() {
    let text = run(
        "semaphore_through_methods",
        r#"class sem_c;
  semaphore s;
  function new(); s = new(0); endfunction
  task grab(); s.get(1); endtask
  function void free(); s.put(1); endfunction
endclass
module test;
  sem_c c = new();
  initial begin
    $display("T|grab-wait @%0t", $time);
    c.grab();
    $display("T|grabbed @%0t", $time);
    $finish;
  end
  initial begin #7 c.free(); $display("T|freed @%0t", $time); end
endmodule"#,
    );
    assert!(text.contains("T|grabbed @7"), "{text}");
}

#[test]
fn class_event_through_methods() {
    let text = run(
        "class_event_through_methods",
        r#"class ev_c;
  event ev;
  task wait_it(); @(ev); endtask
  function void fire(); -> ev; endfunction
endclass
module test;
  ev_c c = new();
  initial begin
    $display("T|wait @%0t", $time);
    c.wait_it();
    $display("T|woke @%0t", $time);
    $finish;
  end
  initial begin #6 c.fire(); end
endmodule"#,
    );
    assert!(text.contains("T|woke @6"), "{text}");
}

#[test]
fn try_family_and_num() {
    let text = run(
        "try_family_and_num",
        r#"class fifo_c;
  mailbox #(int) m;
  function new(); m = new(4); endfunction
  function bit tg(output int x); return m.try_get(x); endfunction
  function bit tp2(output int x); return m.try_peek(x); endfunction
  function int n(); return m.num(); endfunction
endclass
module test;
  fifo_c f = new();
  int x; bit g, p;
  initial begin
    g = f.tg(x); p = f.tp2(x);
    $display("T|empty tg=%0d tp=%0d num=%0d", g, p, f.n());
    void'(f.m.try_put(5)); void'(f.m.try_put(6));
    g = f.tp2(x); $display("T|peeked=%0d x=%0d num=%0d", g, x, f.n());
    g = f.tg(x);  $display("T|got=%0d x=%0d num=%0d", g, x, f.n());
    $finish;
  end
endmodule"#,
    );
    assert!(text.contains("T|empty tg=0 tp=0 num=0"), "{text}");
    assert!(text.contains("T|peeked=1 x=5 num=2"), "{text}");
    assert!(text.contains("T|got=1 x=5 num=1"), "{text}");
}

#[test]
fn two_gets_round_robin() {
    let text = run(
        "two_gets_round_robin",
        r#"class fifo_c;
  mailbox #(int) m;
  function new(); m = new(); endfunction
  task get_it(output int x); m.get(x); endtask
endclass
module test;
  fifo_c f = new();
  int a, b;
  initial begin f.get_it(a); $display("T|A=%0d @%0t", a, $time); end
  initial begin f.get_it(b); $display("T|B=%0d @%0t", b, $time); end
  initial begin
    #5 f.m.put(1);
    #5 f.m.put(2);
    #5 $finish;
  end
endmodule"#,
    );
    assert!(text.contains("T|A=1 @5"), "{text}");
    assert!(text.contains("T|B=2 @10"), "{text}");
}
