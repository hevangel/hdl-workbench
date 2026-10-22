//! §18.7 — in `obj.randomize() with { obj.x == encl_member; … }` the inline may
//! compare a prefixed object member against a member of the ENCLOSING scope
//! (the sequence's `start_addr`, `transmit_del` — exactly what a UVM `uvm_do`
//! inline block does). Once the randomize switches `this` to the OBJECT, those
//! free identifiers no longer resolve and read 0, so `req.addr == start_addr`
//! forced `addr` to 0 and with an array-`.size() == <=s>` coupling the solve
//! dead-ended. The fix freezes each enclosing-scope member referenced (but not
//! a rand member of the object) to its current value and lets the prefixed
//! member read the object's own field. Validated byte-for-byte against the
//! reference simulator.

use xezim::simulate;

fn tags(src: &str) -> Vec<String> {
    let sim = simulate(src, 1000).expect("sim");
    sim.output
        .iter()
        .filter(|o| o.message.starts_with("TAG_"))
        .map(|o| o.message.clone())
        .collect()
}

const SEQUENCE_TRANSACTION: &str = r#"
typedef enum { WRITE, READ } rw_e;
class xfer;
  rand bit [15:0] addr;
  rand rw_e        read_write;
  rand int unsigned size;
  rand byte unsigned data[];
  rand int unsigned error_pos;
  rand int unsigned transmit_delay;
  constraint c_read { read_write inside { WRITE, READ }; }
  constraint c_size { size inside {1,2,4,8}; data.size() == size; }
  constraint c_tr { transmit_delay <= 10; }
endclass
class seq_c;
  int start_addr = 10;
  int transmit_del = 0;
  function void run();
    xfer m = new();
    int r = m.randomize() with {
      m.addr == start_addr;
      m.read_write == READ;
      m.size == 2;
      m.error_pos == 1000;
      m.transmit_delay == transmit_del;
    };
    if (r != 1 || m.addr != start_addr || m.read_write != READ || m.size != 2
        || m.data.size() != 2 || m.error_pos != 1000)
      $display("TAG_FAIL %0d %0d %0d %0d %0d %0d", r, m.addr,
               m.read_write, m.size, m.data.size(), m.error_pos);
    else $display("TAG_PASS");
  endfunction
endclass
module top;
  initial begin seq_c s = new(); s.run(); $finish; end
endmodule
"#;

#[test]
fn prefixed_inline_against_enclosing_scope_members() {
    assert_eq!(
        tags(SEQUENCE_TRANSACTION),
        vec!["TAG_PASS"],
        "enclosing-scope members must be frozen in the inline"
    );
}

const SIMPLE: &str = r#"
class req;
  rand bit [15:0] addr;
  rand int unsigned size;
endclass
class seq_c;
  int start_addr;
  function new(int a); start_addr = a; endfunction
  function void run();
    req m = new();
    int r = m.randomize() with { m.addr == start_addr; m.size == 1; };
    if (r != 1 || m.addr != start_addr || m.size != 1)
      $display("TAG_FAIL %0d %0d %0d", r, m.addr, m.size);
    else $display("TAG_PASS");
  endfunction
endclass
module top;
  initial begin seq_c s = new(32); s.run(); $finish; end
endmodule
"#;

#[test]
fn prefixed_member_via_enclosing_scope_value() {
    assert_eq!(
        tags(SIMPLE),
        vec!["TAG_PASS"],
        "enclosing member must drive prefixed member"
    );
}
