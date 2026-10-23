//! `obj.randomize() with { obj.member[i] == v; }` — a RECEIVER-QUALIFIED
//! dynamic-array element in the with-clause. The element and whole-element
//! constraint paths key by the bare member name, and the receiver local is
//! out of scope in the solver frame, so `req.data[0] == data0` failed to
//! map onto a rand target and `randomize()` returned 0 — the ubus
//! `write_byte_seq` shape (`req.data[0] == data0` next to scalar pins).
//! Reference-verified: randomize succeeds and every pin holds.
use xezim::simulate;

fn messages(src: &str) -> Vec<String> {
    let sim = simulate(src, 1_000).expect("simulate failed");
    sim.output.iter().map(|o| o.message.clone()).collect()
}

const SRC: &str = r#"
module top;
  typedef enum bit [1:0] { NOP, READ, WRITE } rw_e;
  class ubus_transfer;
    rand bit [15:0] addr;
    rand rw_e read_write;
    rand int unsigned size;
    rand bit [7:0] data[];
    rand bit [3:0] wait_state[];
    rand int unsigned error_pos;
    rand int unsigned transmit_delay = 0;
    constraint c_rw { read_write inside { READ, WRITE }; }
    constraint c_size { size inside {1,2,4,8}; }
    constraint c_sizes { data.size() == size; wait_state.size() == size; }
    constraint c_td { transmit_delay <= 10; }
  endclass

  class write_byte_seq;
    rand bit [15:0] start_addr;
    rand bit [7:0] data0;
    rand int unsigned transmit_del = 0;
    constraint td_ct { (transmit_del <= 10); }
    ubus_transfer req;
    task body();
      req = new;
      if (!req.randomize() with {
          req.addr == start_addr;
          req.read_write == WRITE;
          req.size == 1;
          req.data[0] == data0;
          req.error_pos == 1000;
          req.transmit_delay == transmit_del;
        })
        $display("RAND_FAIL");
      else
        $display("RAND_OK addr=%h rw=%s size=%0d d0=%h",
                 req.addr, req.read_write.name(), req.size, req.data[0]);
    endtask
  endclass

  initial begin
    write_byte_seq s = new();
    s.start_addr = 16'hc4bc;
    s.data0 = 8'hb0;
    s.body();
    if (s.req.addr == 16'hc4bc && s.req.size == 1 && s.req.data[0] == 8'hb0
        && s.req.error_pos == 1000 && s.req.read_write == WRITE
        && s.req.wait_state.size() == 1 && s.req.transmit_delay == 0)
      $display("TAG_PASS");
    else
      $display("TAG_FAIL");
  end
endmodule
"#;

#[test]
fn randomize_with_qualifies_element_constraint() {
    let msgs = messages(SRC);
    assert!(
        msgs.iter()
            .any(|m| m.starts_with("RAND_OK addr=c4bc rw=WRITE size=1 d0=b0")),
        "expected RAND_OK with pinned fields, got {msgs:?}"
    );
    assert!(
        msgs.iter().any(|m| m == "TAG_PASS"),
        "expected TAG_PASS, got {msgs:?}"
    );
}
