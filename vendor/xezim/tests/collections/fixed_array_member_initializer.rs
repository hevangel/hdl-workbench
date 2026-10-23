//! Regression test for fixed-size UNPACKED array class-member initializers.
//!
//! `int fa[4] = '{1,2,3,4};` — the elaboration-side gate that previously dropped
//! all dimensioned member initializers was relaxed (so queue/dynamic-array
//! members initialize), but the simulator only special-cased queues and dynamic
//! arrays. A fixed unpacked array fell through to `inst.properties.insert(...)`,
//! storing the assignment pattern as a scalar, so `fa[0]`/`fa[3]` read as 0.
//! The fix applies the assignment pattern member-wise to the per-instance
//! element signals `<handle>#fa[i]` (registered as a real fixed array).

use xezim::simulate;

const SRC: &str = r#"
class C;
  int fa[4] = '{1,2,3,4};
  function void show();
    if (fa[0] == 1 && fa[1] == 2 && fa[2] == 3 && fa[3] == 4)
      $display("TAG_PASS");
    else
      $display("TAG_FAIL fa=%0d %0d %0d %0d", fa[0], fa[1], fa[2], fa[3]);
  endfunction
endclass

module top;
  initial begin
    C c = new();
    c.show();
  end
endmodule
"#;

#[test]
fn test_fixed_array_member_initializer() {
    let sim = simulate(SRC, 10_000).expect("simulation failed");
    assert!(
        sim.output
            .iter()
            .any(|line| line.message.contains("TAG_PASS")),
        "expected TAG_PASS in output, got: {:?}",
        sim.output
    );
}
