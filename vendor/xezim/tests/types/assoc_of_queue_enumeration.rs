//! IEEE 1800-2017 §7.9.1 / §12.7.3: an associative array whose ELEMENTS are
//! queues (`T all[key][$]`). Two defects were fixed together:
//!
//! 1. `num()`/`first()`/`next()`/`last()`/`prev()` counted/enumerated the
//!    WRONG key set: the prefix scan `all[` matched every element signal
//!    `all[KEY][IDX]` and the key text was sliced as `KEY][IDX` (everything
//!    up to the LAST `]`) instead of up to the FIRST `]`. So `num()`
//!    over-counted (5 for 2 keys) and `first()` returned a garbage key.
//!    Fixed by extracting top-level keys up to the first `]`, dedup'd.
//!
//! 2. A two-variable `foreach (all[k, i])` over such an array only ever bound
//!    the outer index `k` — the inner index `i` stayed at its last value and
//!    the body ran once per KEY, dropping every queue element but the first.
//!    This is the exact shape of the resource pool's precedence sort, whose
//!    2-D `foreach(all[aa_iter, q_iter])` repopulates a queue. Fixed by
//!    iterating the outer var over the (sparse) assoc keys and the inner var
//!    over each key's queue [0, size), higher cardinality changing fastest.
//!
//! Both are validated byte-for-byte against the reference simulator.

use xezim::simulate;

#[test]
fn assoc_of_queue_num_first_next() {
    let src = r#"
module top;
   initial begin
      int all[int][$];
      int k;
      int got[$];
      all[10].push_back(1);
      all[20].push_back(2);
      all[20].push_back(3);   // same key as the previous push
      // num() counts distinct KEYS, not element signals.
      if (all.num() != 2)  $display("TAG_FAIL num=%0d", all.num());
      // first/next walk the distinct keys in ascending order.
      else if (!all.first(k) || k != 10) $display("TAG_FAIL first=%0d", k);
      else if (!all.next(k)  || k != 20) $display("TAG_FAIL next=%0d", k);
      else if (all.next(k))             $display("TAG_FAIL extra key=%0d", k);
      else $display("TAG_PASS");
   end
endmodule
"#;
    let sim = simulate(src, 50).expect("simulate");
    assert!(
        sim.output.iter().any(|l| l.message == "TAG_PASS"),
        "num/first/next on an assoc-of-queue must use distinct top-level keys.\n{}",
        sim.output
            .iter()
            .map(|l| l.message.clone())
            .collect::<Vec<_>>()
            .join("\n")
    );
}

#[test]
fn foreach_two_var_assoc_of_queue() {
    let src = r#"
module top;
   initial begin
      int all[int][$];
      int q[$];
      // Distribute into all[precedence] (mutate-in-place must persist).
      q.push_back(10);
      q.push_back(20);
      q.push_back(10);
      begin
         int i, prec;
         for (i = 0; i < q.size(); ++i) begin
            prec = q[i];
            all[prec].push_front(q[i]);
         end
      end
      // Rebuild q via a 2-variable foreach (the precedence-sort repopulate).
      q.delete();
      foreach (all[aa_iter, q_iter])
         q.push_front(all[aa_iter][q_iter]);
      // q should hold all three values again.
      if (q.size() != 3) $display("TAG_FAIL size=%0d", q.size());
      else $display("TAG_PASS");
   end
endmodule
"#;
    let sim = simulate(src, 50).expect("simulate");
    assert!(
        sim.output.iter().any(|l| l.message == "TAG_PASS"),
        "two-variable foreach over an assoc-of-queue must iterate keys AND queue elements.\n{}",
        sim.output
            .iter()
            .map(|l| l.message.clone())
            .collect::<Vec<_>>()
            .join("\n")
    );
}
