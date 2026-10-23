//! A compiled `return '{...}` hands its elements over without writing the
//! by-name return snapshot; every consumer that reads the snapshot by name
//! (a dotted or hierarchical destination, a cast, a nested return) must see
//! this call's elements, and an interpreted return must never reuse values a
//! compiled call left behind. A once-called function whose loop stores a
//! collection-returning call into a local queue must also read the stored
//! elements. Expected values are the reference simulator's.

use xezim::simulate;

const SRC: &str = r#"
typedef int intq_t[$];
class Src;
  int base = 10;
  function intq_t trio(); return '{base, base + 1, base + 2}; endfunction
  function intq_t one(); return '{base * 5}; endfunction
  function intq_t nonlit(); intq_t t; t.push_back(base * 100); return t; endfunction
endclass
class User;
  Src s;
  int mq[$];
  function new(); s = new; endfunction
  function int warm(int n);
    int acc = 0;
    for (int i = 0; i < n; i++) begin
      mq = s.one();
      acc += mq[0];
      mq = s.trio();
      mq = s.nonlit();
      acc += mq[0];
    end
    return acc;
  endfunction
endclass
function automatic longint loop_b(Src r, int n);
  longint acc = 0;
  intq_t q;
  for (int i = 0; i < n; i++) begin
    r.base = i;
    q = r.trio();
    acc += q[0] + q[2];
  end
  return acc;
endfunction
module top;
  int mtop[$];
  User u;
  initial begin
    u = new;
    $display("T|warm=%0d", u.warm(1100));
    u.s.base = 7;
    mtop = u.s.trio();
    top.mtop = u.s.one();
    $display("T|top.mtop=%p", mtop);
    u.mq = u.s.trio();
    void'(u.s.one());
    mtop = u.s.nonlit();
    $display("T|u.mq=%p mtop=%p", u.mq, mtop);
    $display("T|loop=%0d", loop_b(u.s, 10));
  end
endmodule
"#;

#[test]
fn compiled_collection_return_by_name_consumers() {
    let sim = simulate(SRC, 10).expect("simulate failed");
    let o: Vec<String> = sim
        .output
        .iter()
        .map(|o| o.message.trim().to_string())
        .collect();
    for want in [
        "T|warm=1155000",
        "T|top.mtop='{35}",
        "T|u.mq='{7, 8, 9} mtop='{700}",
        "T|loop=110",
    ] {
        assert!(o.iter().any(|l| l == want), "missing {want:?} in {o:?}");
    }
}
