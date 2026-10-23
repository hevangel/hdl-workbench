//! §7.12: an array reduction or sort with a `with` clause whose receiver is a
//! CLASS PROPERTY — `o.arr.sum() with (item * 2)`, through a nested handle
//! (`o.in.arr`), an array-of-handles element (`objs[1].arr`) or `this.arr`
//! inside a method. The receiver was resolved by its parse-level name, which
//! is not the property's storage, so the `with` clause was dropped: the
//! reductions returned the plain result
//! (`o.arr.sum() with (item * 2)` gave 7 for `'{1, 2, 4}` instead of 14) and
//! `sort`/`rsort` ignored their key.
//!
//! Every expectation was cross-checked against the reference simulator.

use xezim::simulate;

#[test]
fn with_clause_reaches_class_property_receivers() {
    let src = r#"
class wi_inner;
  int arr[3];
  int q[$];
  function new(); arr = '{1, 2, 4}; q = {3, 5, 7}; endfunction
endclass
class wc6;
  int arr[3];
  int q[$];
  int da[];
  wi_inner in;
  function new(); arr = '{1, 2, 4}; q = {3, 5, 7}; da = '{2, 3, 6}; in = new; endfunction
  function int msum(); return arr.sum() with (item * 2); endfunction
  function int mthis(); return this.arr.sum() with (item * 3); endfunction
  function void srt(); q.sort() with (-item); endfunction
  function void rsrt(); this.q.rsort() with (item % 3); endfunction
endclass
module top;
  initial begin
    automatic wc6 o = new;
    automatic wc6 objs[2];
    objs[0] = new; objs[1] = o;
    $display("A %0d %0d %0d %0d %0d", o.arr.sum() with (item * 2), o.arr.product() with (item + 1), o.arr.and() with (item | 8), o.arr.or() with (item << 4), o.arr.xor() with (item + 1));
    $display("B %0d %0d %0d %0d", o.q.sum() with (item * item), o.da.sum() with (item * 10), o.in.arr.sum() with (item * 5), o.in.q.sum() with (item - 1));
    $display("C %0d %0d", o.msum(), o.mthis());
    $display("D %0d %0d %0d", o.arr.sum() with (item * item.index), o.arr.sum(x) with (x + 100), o.q.sum(x) with (x * x.index));
    $display("E %0d %0d %0d", o.in.arr.sum() with (int'(item > 1)), (o.arr.sum() with (item * 2)) == 14, objs[1].arr.sum() with (item * 2));
    o.q.sort() with (-item);
    $display("F %p", o.q);
    o.q.rsort() with (item % 5);
    $display("G %p", o.q);
    o.srt();
    $display("H %p", o.q);
    o.rsrt();
    $display("I %p", o.q);
    $display("J %0d", o.arr.sum());
  end
endmodule
"#;
    let sim = simulate(src, 10).expect("simulate failed");
    let o: Vec<String> = sim.output.iter().map(|o| o.message.clone()).collect();
    for want in [
        "A 14 30 8 112 4",
        "B 83 110 35 12",
        "C 14 21",
        "D 10 307 19",
        "E 2 1 14",
        "F '{7, 5, 3}",
        "G '{3, 7, 5}",
        "H '{7, 5, 3}",
        "I '{5, 7, 3}",
        "J 7",
    ] {
        assert!(
            o.iter().any(|l| l.trim() == want),
            "missing {want:?} in {o:?}"
        );
    }
}
