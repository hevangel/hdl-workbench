//! Regression test for the NON-positional assignment-pattern forms of a
//! fixed-size UNPACKED array class-member initializer (IEEE 1800-2023
//! §10.9.1 / §10.10).
//!
//! The positional `'{1,2,3,4}` path (covered by
//! `fixed_array_member_initializer.rs`) was already correct, but the
//! replication form `'{4{9}}` and the `default:` fill `'{default:7}` collapsed
//! into a single packed value stored at element 0 — leaving the rest at 0.
//! Reference: every element receives the value (`9 9 9 9` / `7 7 7 7`). A
//! non-zero lower bound on the positional form (`fa[1:4]`) is also exercised.

use xezim::simulate;

const SRC: &str = r#"
class C;
  int pos[4]   = '{1,2,3,4};   // positional (unchanged)
  int rep[4]   = '{4{9}};      // replication -> every element 9
  int def[4]   = '{default:7}; // default fill -> every element 7
  int lsb[1:4] = '{10,20,30,40}; // positional, non-zero lower bound
  function void show();
    integer bad;
    bad = 0;
    if (!(pos[0]==1 && pos[1]==2 && pos[2]==3 && pos[3]==4)) bad = bad + 1;
    if (!(rep[0]==9 && rep[1]==9 && rep[2]==9 && rep[3]==9)) bad = bad + 1;
    if (!(def[0]==7 && def[1]==7 && def[2]==7 && def[3]==7)) bad = bad + 1;
    if (!(lsb[1]==10 && lsb[2]==20 && lsb[3]==30 && lsb[4]==40)) bad = bad + 1;
    if (bad == 0)
      $display("TAG_PASS");
    else
      $display("TAG_FAIL pos=%0d %0d %0d %0d rep=%0d %0d %0d %0d def=%0d %0d %0d %0d lsb=%0d %0d %0d %0d",
        pos[0],pos[1],pos[2],pos[3], rep[0],rep[1],rep[2],rep[3],
        def[0],def[1],def[2],def[3], lsb[1],lsb[2],lsb[3],lsb[4]);
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
fn test_fixed_array_member_pattern_forms() {
    let sim = simulate(SRC, 10_000).expect("simulation failed");
    assert!(
        sim.output
            .iter()
            .any(|line| line.message.contains("TAG_PASS")),
        "expected TAG_PASS in output, got: {:?}",
        sim.output
    );
}
