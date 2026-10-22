//! §7.4.5: a class property that is a fixed array of collections
//! (`int q[2][2][$]`, `int d[2][]`, `int a[2][int]`). Elements had no
//! storage: `q[i][j].push_back(x)` silently did nothing and `q[i][j].size()`
//! stayed 0, while `$size(q)` and `foreach (q[i])` answered off the outer
//! shape. Core PR #41 records the shape; the simulator now keys each element
//! collection as `<handle>#q[i][j]`. Expected values from the reference
//! simulator.
use xezim::simulate;

fn messages(src: &str) -> Vec<String> {
    let sim = simulate(src, 1_000).expect("simulate failed");
    sim.output.iter().map(|o| o.message.clone()).collect()
}

#[test]
fn queue_dyn_and_assoc_elements_of_class_arrays() {
    let msgs = messages(
        "class sb;
  int expected [2][2][$];
  int dq [2][];
  int aa [2][int];
  int q1 [3][$];
  function void fill();
    expected[1][0].push_back(7);
    expected[1][0].push_back(8);
    dq[1] = new[3]; dq[1][2] = 5;
    aa[0][42] = 9;
    q1[2].push_back(11);
  endfunction
  function int dqsz(); return dq[1].size(); endfunction
  function int aaex(); return aa[0].exists(42); endfunction
endclass
module tb;
  initial begin
    sb s = new;
    s.fill();
    $display(\"size=%0d q=%0d %0d outer=%0d dq=%0d/%0d aa=%0d/%0d q1=%0d/%0d\",
      s.expected[1][0].size(), s.expected[1][0][0], s.expected[1][0][1], $size(s.expected),
      s.dqsz(), s.dq[1][2], s.aaex(), s.aa[0][42], q1sz(s), s.q1[2][0]);
    $finish;
  end
  function int q1sz(sb h); return h.q1[2].size(); endfunction
endmodule",
    );
    assert!(
        msgs.iter()
            .any(|m| m == "size=2 q=7 8 outer=2 dq=3/5 aa=1/9 q1=1/11"),
        "{msgs:?}"
    );
}
