//! §23.6 downward hierarchical writes (`u_sub.sig = v`) while class objects
//! exist. Reference-validated.
//!
//! The lvalue path tries `prefix.prop` as a property write through a class
//! handle, evaluating the prefix from a clone of the identifier. The clone
//! kept the FULL path's resolution cache, so after the first write resolved
//! `u_sub.zzq`, the prefix `u_sub` evaluated as that signal: its value named
//! a live object, and every later write landed in the object instead. Any
//! testbench with a class object (every UVM bench) lost them.

use xezim::simulate;

fn u(sim: &xezim::compiler::Simulator, n: &str) -> u64 {
    sim.get_signal(n)
        .or_else(|| sim.get_signal(&format!("tb.{}", n)))
        .unwrap_or_else(|| panic!("signal not found: {}", n))
        .to_u64()
        .unwrap_or_else(|| panic!("{} not u64-able (x/z?)", n))
}

#[test]
fn repeated_downward_writes_with_a_live_object() {
    let src = r#"
module sub;
  logic [7:0] zzq;
endmodule
class c; int x; endclass
module tb;
  sub u_sub();
  c h;
  logic [7:0] s1, s2, s3, s4;
  initial begin
    h = new;
    u_sub.zzq = 8'h01; s1 = u_sub.zzq;
    u_sub.zzq = 8'h02; s2 = u_sub.zzq;
    #1 u_sub.zzq = 8'h03; s3 = u_sub.zzq;
    u_sub.zzq[3:0] = 4'h9; s4 = u_sub.zzq;
  end
endmodule
"#;
    let sim = simulate(src, 100).expect("simulate failed");
    assert_eq!(u(&sim, "s1"), 0x01);
    assert_eq!(u(&sim, "s2"), 0x02);
    assert_eq!(u(&sim, "s3"), 0x03);
    assert_eq!(u(&sim, "s4"), 0x09);
}
