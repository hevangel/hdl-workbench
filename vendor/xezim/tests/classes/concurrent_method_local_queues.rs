//! §6.21 / §13.3 — queue and dynamic-array locals of a class method are
//! automatic: every invocation owns its storage. They were registered under
//! their bare names, so concurrent calls of one method shared a queue: each
//! declaration emptied the other call's, and a call returning first freed
//! it under the other (`q=x`). The per-call key must also serve whole-array
//! copies (`m_info[k].addr = addrs` after a `ref` fill, the UVM register-map
//! shape) and for a queue of structs handed to a `ref` formal (UVM's
//! `perform_accesses(ref uvm_reg_bus_op accesses[$])`, whose members came
//! through blank). The expected lines were cross-checked against the
//! reference simulator.

use xezim::simulate;

fn run(src: &str) -> Vec<String> {
    simulate(src, 1000)
        .expect("simulate failed")
        .output
        .iter()
        .map(|o| o.message.clone())
        .collect()
}

const CONCURRENT: &str = r#"
class worker;
  task run(int id, int d);
    int q[$];
    int da[];
    string sq[$];
    int aa[int];
    q.push_back(id);
    da = new[1];
    da[0] = id;
    sq.push_back($sformatf("s%0d", id));
    aa[id] = id;
    #d;
    q.push_back(id * 10);
    da = new[2](da);
    da[1] = id * 10;
    #d;
    $display("%0t id=%0d q=%p da=%p sq=%p aa.num=%0d", $time, id, q, da, sq, aa.num());
  endtask
endclass
module top;
  worker w;
  initial begin
    w = new;
    fork
      w.run(1, 3);
      w.run(2, 5);
    join
    w.run(3, 1);
  end
endmodule
"#;

const STRUCT_QUEUE_BY_REF: &str = r#"
typedef struct { int kind; longint addr; longint data; } bus_op;
class map_c;
  task perform(ref bus_op accesses[$], input int tag);
    foreach (accesses[i]) begin
      bus_op rw_access = accesses[i];
      #1;
      $display("%0t tag=%0d i=%0d kind=%0d addr=%0d data=%0d", $time, tag, i, rw_access.kind, rw_access.addr, rw_access.data);
    end
  endtask
  task do_access(int base, int cnt);
    bus_op accesses[$];
    longint adr[];
    adr = new[cnt];
    foreach (adr[i]) adr[i] = base + 4*i;
    accesses.delete();
    foreach (adr[i]) begin
      bus_op rw_access;
      rw_access.kind = 1;
      rw_access.addr = adr[i];
      rw_access.data = 100 + i;
      accesses.push_back(rw_access);
    end
    perform(accesses, base);
  endtask
endclass
module top;
  map_c m;
  initial begin
    m = new;
    m.do_access(0, 1);
    m.do_access(8, 2);
    m.do_access(100, 2);
  end
endmodule
"#;

const REF_FILL_COPY: &str = r#"
class info_c;
  int unsigned addr[];
  int offset;
endclass
class map_c;
  info_c m_info[string];
  function int to_map(int base, int n, ref int unsigned addr[]);
    int unsigned local_addr[];
    local_addr = new[n];
    foreach (local_addr[i]) local_addr[i] = base + i * 4;
    addr = new [local_addr.size()] (local_addr);
    foreach (addr[idx]) addr[idx] += 1000;
    return 4;
  endfunction
  virtual function int phys(int base, int n, ref int unsigned addr[]);
    return to_map(base, n, addr);
  endfunction
  function void init();
    m_info["a"] = new; m_info["a"].offset = 16;
    m_info["b"] = new; m_info["b"].offset = 32;
    foreach (m_info[k]) begin
      int unsigned addrs[];
      int w;
      w = phys(m_info[k].offset, 2, addrs);
      foreach (addrs[i]) $display("%s addr[%0d]=%0d", k, i, addrs[i]);
      m_info[k].addr = addrs;
    end
  endfunction
endclass
module top;
  map_c m;
  initial begin
    m = new;
    m.init();
    foreach (m.m_info[k]) $display("%s -> %p", k, m.m_info[k].addr);
  end
endmodule
"#;

#[test]
fn concurrent_calls_get_their_own_queues() {
    let out = run(CONCURRENT);
    let want = [
        "6 id=1 q='{1, 10} da='{1, 10} sq='{\"s1\"} aa.num=1",
        "10 id=2 q='{2, 20} da='{2, 20} sq='{\"s2\"} aa.num=1",
        "12 id=3 q='{3, 30} da='{3, 30} sq='{\"s3\"} aa.num=1",
    ];
    assert_eq!(out, want, "{out:?}");
}

#[test]
fn local_dyn_array_filled_by_ref_copies_into_property() {
    let out = run(REF_FILL_COPY);
    let want = [
        "a addr[0]=1016",
        "a addr[1]=1020",
        "b addr[0]=1032",
        "b addr[1]=1036",
        "a -> '{1016, 1020}",
        "b -> '{1032, 1036}",
    ];
    assert_eq!(out, want, "{out:?}");
}

#[test]
fn local_struct_queue_passed_by_ref_keeps_members() {
    let out = run(STRUCT_QUEUE_BY_REF);
    let want = [
        "1 tag=0 i=0 kind=1 addr=0 data=100",
        "2 tag=8 i=0 kind=1 addr=8 data=100",
        "3 tag=8 i=1 kind=1 addr=12 data=101",
        "4 tag=100 i=0 kind=1 addr=100 data=100",
        "5 tag=100 i=1 kind=1 addr=104 data=101",
    ];
    assert_eq!(out, want, "{out:?}");
}
