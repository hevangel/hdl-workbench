//! §15.3 — a semaphore that is a CLASS MEMBER must honour its constructor's
//! key count and the no-argument `put()`. A member initialized inline
//! (`semaphore lock = new(1);`) was allocated with zero keys (and an inline
//! `mailbox #(int) box = new(1);` lost its bound), and `lock.put()` with no
//! argument returned nothing when called from a class method, so the next
//! `get()` blocked forever. A pipelined UVM driver guarding its command phase
//! with such a semaphore never fetched its first item. The expected lines are
//! the reference simulator's output.

use xezim::simulate;

const SRC: &str = r#"
class drv;
  semaphore lock = new(1);
  semaphore two = new(2);
  mailbox #(int) box = new(1);
  task run();
    fork
      xfer(1);
      xfer(2);
    join
  endtask
  task automatic xfer(int id);
    repeat (2) begin
      lock.get();
      $display("%0t xfer %0d got lock", $time, id);
      #10;
      lock.put();
      #1;
    end
  endtask
  task keys();
    two.get(2);
    $display("%0t got two keys", $time);
    two.put();
    two.put();
    $display("%0t try_get(2)=%0d", $time, two.try_get(2));
    $display("%0t box try_put=%0d %0d", $time, box.try_put(1), box.try_put(2));
  endtask
endclass
module top;
  drv d;
  initial begin
    d = new;
    d.keys();
    d.run();
    $display("%0t done", $time);
  end
endmodule
"#;

#[test]
fn class_member_semaphore_keys_and_bare_put() {
    let out: Vec<String> = simulate(SRC, 1000)
        .expect("simulate failed")
        .output
        .iter()
        .map(|o| o.message.clone())
        .collect();
    let want = [
        "0 got two keys",
        "0 try_get(2)=1",
        "0 box try_put=1 0",
        "0 xfer 1 got lock",
        "10 xfer 2 got lock",
        "20 xfer 1 got lock",
        "30 xfer 2 got lock",
        "41 done",
    ];
    assert_eq!(out, want, "{out:?}");
}
