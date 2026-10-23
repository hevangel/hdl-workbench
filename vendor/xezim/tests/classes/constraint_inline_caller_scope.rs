//! §18.7 / §18.7.1 — name binding in `obj.randomize() with { … }`.
//!
//! A name in the inline block binds first in the randomized object's class
//! scope, then in the scope of the randomize() call; `local::name` binds in
//! the call's scope only. Operands of the call's scope are state variables of
//! the solve. Before the fix only a bare member of the caller's own class was
//! handed over (not an inherited one, an array element, a queue, a handle
//! member, a caller local used in an `if`/`inside`, …), those constraints
//! read x, and every randomize() failed — the UVM `start_item` / `randomize()
//! with {addr == src_addr; write_data == buffer[i];}` idiom among them. A
//! caller-local variable named like an object member also shadowed the
//! member inside the object's own constraints.
//!
//! Every count below was cross-checked against the reference simulator.

use xezim::simulate;

fn lines(src: &str) -> Vec<String> {
    let sim = simulate(src, 1000).expect("simulate failed");
    sim.output.iter().map(|o| o.message.clone()).collect()
}

const MEMBERS_AND_ELEMENTS: &str = r#"
class cfg_c;
  int lim = 40;
  int arr[3] = '{5, 6, 7};
endclass

class item;
  rand bit [31:0] addr;
  rand bit [31:0] data;
  rand int delay;
  rand bit [7:0] payload[];
  bit [31:0] note = 32'h77;
  constraint c_delay { delay inside {[1:20]}; }
  constraint c_soft { soft data == 32'hAAAA; }
  constraint c_len { payload.size() inside {[1:4]}; }
endclass

class base_seq;
  int read_addr = 32'h100;
  bit [31:0] note = 32'h1234;
endclass

class my_seq extends base_seq;
  bit [31:0] buffer[];
  bit [31:0] fixed_buf[4] = '{1, 2, 3, 4};
  int n = 3;
  cfg_c cfg = new;
  int delay = 999;

  function void run();
    item it = new;
    int ok, bad;
    int addr = 7;
    buffer = new[4];
    foreach (buffer[k]) buffer[k] = 32'h5000 + k;
    ok = 0; bad = 0;
    for (int i = 0; i < 10; i++) begin
      if (!it.randomize() with { addr == read_addr; }) bad++;
      else if (it.addr == read_addr) ok++;
    end
    $display("A inherited: ok=%0d fail=%0d", ok, bad);
    ok = 0; bad = 0;
    for (int i = 0; i < 4; i++) begin
      if (!it.randomize() with { data == buffer[i]; }) bad++;
      else if (it.data == buffer[i]) ok++;
    end
    $display("B dyn elem: ok=%0d fail=%0d", ok, bad);
    ok = 0; bad = 0;
    for (int i = 0; i < 3; i++) begin
      if (!it.randomize() with { data == fixed_buf[i+1]; addr == buffer[i] + 4; }) bad++;
      else if (it.data == fixed_buf[i+1] && it.addr == buffer[i] + 4) ok++;
    end
    $display("B2 fixed elem: ok=%0d fail=%0d", ok, bad);
    ok = 0; bad = 0;
    for (int i = 0; i < 5; i++) begin
      if (!it.randomize() with { data == note; }) bad++;
      else if (it.data == 32'h77) ok++;
    end
    $display("C object member wins: ok=%0d fail=%0d", ok, bad);
    ok = 0; bad = 0;
    for (int i = 0; i < 5; i++) begin
      if (!it.randomize() with { data == local::addr; }) bad++;
      else if (it.data == 7) ok++;
    end
    $display("D local::: ok=%0d fail=%0d", ok, bad);
    ok = 0; bad = 0;
    for (int i = 0; i < 5; i++) begin
      if (!it.randomize() with { delay < 3; }) bad++;
      else if (it.delay >= 1 && it.delay < 3) ok++;
    end
    $display("D2 object delay wins: ok=%0d fail=%0d", ok, bad);
    ok = 0; bad = 0;
    for (int i = 0; i < 5; i++) begin
      if (!it.randomize() with { addr < cfg.lim; data == cfg.arr[2]; }) bad++;
      else if (it.addr < 40 && it.data == 7) ok++;
    end
    $display("E handle member: ok=%0d fail=%0d", ok, bad);
    ok = 0; bad = 0;
    for (int i = 0; i < 10; i++) begin
      if (!it.randomize() with { addr inside {[read_addr : read_addr + n*4]}; }) bad++;
      else if (it.addr >= 32'h100 && it.addr <= 32'h10c) ok++;
    end
    $display("F inside range: ok=%0d fail=%0d", ok, bad);
    ok = 0; bad = 0;
    for (int i = 0; i < 5; i++) begin
      if (!it.randomize() with { data == 32'h5555; }) bad++;
      else if (it.data == 32'h5555) ok++;
    end
    $display("G soft overridden: ok=%0d fail=%0d", ok, bad);
    ok = 0; bad = 0;
    for (int i = 0; i < 5; i++) begin
      if (!it.randomize()) bad++;
      else if (it.data == 32'hAAAA) ok++;
    end
    $display("G2 soft kept: ok=%0d fail=%0d", ok, bad);
    ok = 0; bad = 0;
    for (int i = 0; i < 5; i++) begin
      if (!it.randomize() with { payload.size() == n; foreach (payload[j]) payload[j] == j + n; }) bad++;
      else if (it.payload.size() == 3 && it.payload[2] == 5) ok++;
    end
    $display("H inline size: ok=%0d fail=%0d", ok, bad);
    it.c_delay.constraint_mode(0);
    ok = 0; bad = 0;
    for (int i = 0; i < 5; i++) begin
      if (!it.randomize() with { delay == 50; }) bad++;
      else if (it.delay == 50) ok++;
    end
    $display("I constraint_mode off: ok=%0d fail=%0d", ok, bad);
    it.c_delay.constraint_mode(1);
    ok = 0; bad = 0;
    for (int i = 0; i < 5; i++) begin
      if (!it.randomize() with { delay == 50; }) bad++;
      else ok++;
    end
    $display("I2 constraint_mode on (conflict): ok=%0d fail=%0d", ok, bad);
    it.delay = 5;
    it.delay.rand_mode(0);
    ok = 0; bad = 0;
    for (int i = 0; i < 5; i++) begin
      if (!it.randomize() with { addr == delay + 1; }) bad++;
      else if (it.addr == 6 && it.delay == 5) ok++;
    end
    $display("J rand_mode off: ok=%0d fail=%0d", ok, bad);
    ok = 0; bad = 0;
    for (int i = 0; i < 3; i++) begin
      if (!it.randomize() with { delay == 9; }) bad++;
      else ok++;
    end
    $display("J2 rand_mode off conflict: ok=%0d fail=%0d delay=%0d", ok, bad, it.delay);
    it.delay.rand_mode(1);
  endfunction
endclass

module top;
  initial begin
    my_seq s = new;
    s.run();
  end
endmodule
"#;

#[test]
fn caller_members_elements_and_modes() {
    let got: Vec<String> = lines(MEMBERS_AND_ELEMENTS)
        .into_iter()
        .filter(|l| l.contains(": ok="))
        .collect();
    let want = [
        "A inherited: ok=10 fail=0",
        "B dyn elem: ok=4 fail=0",
        "B2 fixed elem: ok=3 fail=0",
        "C object member wins: ok=5 fail=0",
        "D local::: ok=5 fail=0",
        "D2 object delay wins: ok=5 fail=0",
        "E handle member: ok=5 fail=0",
        "F inside range: ok=10 fail=0",
        "G soft overridden: ok=5 fail=0",
        "G2 soft kept: ok=5 fail=0",
        "H inline size: ok=5 fail=0",
        "I constraint_mode off: ok=5 fail=0",
        "I2 constraint_mode on (conflict): ok=0 fail=5",
        "J rand_mode off: ok=5 fail=0",
        "J2 rand_mode off conflict: ok=0 fail=3 delay=5",
    ];
    assert_eq!(got, want);
}

const CALLER_LOCALS: &str = r#"
class item;
  rand bit [15:0] addr;
  rand bit [15:0] data;
  bit [15:0] base = 16'h40;
  constraint c { addr < 16'h100; }
endclass

class seq;
  bit [15:0] lim = 16'h20;
  task run();
    item it = new;
    bit [15:0] addr = 16'h9999;
    bit [15:0] base = 16'h7;
    bit [15:0] q[$] = '{16'h11, 16'h22, 16'h33};
    int ok, bad;
    ok = 0; bad = 0;
    for (int i = 0; i < 10; i++) begin
      if (!it.randomize() with { addr == data; data < lim; }) bad++;
      else if (it.addr == it.data && it.data < 16'h20) ok++;
    end
    $display("K local shadow rand: ok=%0d fail=%0d", ok, bad);
    ok = 0; bad = 0;
    for (int i = 0; i < 10; i++) begin
      if (!it.randomize() with { data == base + 1; }) bad++;
      else if (it.data == 16'h41) ok++;
    end
    $display("K2 local vs object nonrand: ok=%0d fail=%0d", ok, bad);
    ok = 0; bad = 0;
    for (int i = 0; i < 10; i++) begin
      if (!it.randomize() with { data == local::base + local::addr; }) bad++;
      else if (it.data == 16'h99a0) ok++;
    end
    $display("K3 local:: sum: ok=%0d fail=%0d", ok, bad);
    ok = 0; bad = 0;
    for (int i = 0; i < 10; i++) begin
      if (!it.randomize() with { data inside {q}; }) bad++;
      else if (it.data inside {16'h11, 16'h22, 16'h33}) ok++;
    end
    $display("L inside caller queue: ok=%0d fail=%0d", ok, bad);
    ok = 0; bad = 0;
    for (int i = 0; i < 10; i++) begin
      if (!it.randomize() with { data == q[i % 3]; addr == q.size(); }) bad++;
      else if (it.data == q[i % 3] && it.addr == 3) ok++;
    end
    $display("L2 queue elem/size: ok=%0d fail=%0d", ok, bad);
    ok = 0; bad = 0;
    for (int i = 0; i < 10; i++) begin
      if (!it.randomize() with { if (i > 4) data == 16'h5; else data == 16'h6; }) bad++;
      else if (it.data == ((i > 4) ? 16'h5 : 16'h6)) ok++;
    end
    $display("M if/else on caller loop var: ok=%0d fail=%0d", ok, bad);
    ok = 0; bad = 0;
    for (int i = 0; i < 10; i++) begin
      if (!it.randomize() with { (lim > 16'h10) -> data == lim; }) bad++;
      else if (it.data == 16'h20) ok++;
    end
    $display("M2 implication on caller member: ok=%0d fail=%0d", ok, bad);
  endtask
endclass

module top;
  initial begin
    seq s = new;
    s.run();
  end
endmodule
"#;

#[test]
fn caller_locals_queues_and_local_qualifier() {
    let got: Vec<String> = lines(CALLER_LOCALS)
        .into_iter()
        .filter(|l| l.contains(": ok="))
        .collect();
    let want = [
        "K local shadow rand: ok=10 fail=0",
        "K2 local vs object nonrand: ok=10 fail=0",
        "K3 local:: sum: ok=10 fail=0",
        "L inside caller queue: ok=10 fail=0",
        "L2 queue elem/size: ok=10 fail=0",
        "M if/else on caller loop var: ok=10 fail=0",
        "M2 implication on caller member: ok=10 fail=0",
    ];
    assert_eq!(got, want);
}
