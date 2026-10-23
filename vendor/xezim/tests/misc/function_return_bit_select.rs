//! Bit-select writes to a FUNCTION RETURN variable, sized to the declared
//! return-type width.
//!
//! IEEE 1800-2017 §13.4: a function's implicit return variable (named after
//! the function) has the function's declared return type. A bit-select write
//! `funcname[i] = ...` must land for every bit `i` in `[0, width-1]`.
//!
//! Previously the implicit return cell was hardcoded `Value::zero(32)` and
//! NOT registered in the width table, so `funcname = 0; funcname[63] = 1;`
//! (e.g. `uvm_packer::unpack_field_int` filling a 64-bit `uvm_integral_t`)
//! silently dropped every bit >= 32 — the upper half of a packed 64-bit
//! value came back as zero after unpack.

use xezim::simulate;

fn u(sim: &xezim::compiler::Simulator, n: &str) -> u64 {
    sim.get_signal(n)
        .or_else(|| sim.get_signal(&format!("tb.{}", n)))
        .unwrap_or_else(|| panic!("signal not found: {}", n))
        .to_u64()
        .unwrap_or_else(|| panic!("{} not u64-able", n))
}

/// §13.4 — a 64-bit return-typed function fills all 64 bits via bit-select
/// writes to its return variable.
#[test]
fn return_var_bit_select_64bit() {
    const SRC: &str = "typedef logic signed [63:0] u64_t;
class C;
  function u64_t all_ones();
    all_ones = 64'h0;
    for (int i = 0; i < 64; i++) all_ones[i] = 1'b1;
    return all_ones;
  endfunction
  function u64_t upper_only();
    upper_only = 64'h0;
    for (int i = 32; i < 64; i++) upper_only[i] = 1'b1;
    return upper_only;
  endfunction
endclass

module tb;
  u64_t r1;
  u64_t r2;
  int failures = 0;
  initial begin
    C c = new;
    r1 = c.all_ones();
    r2 = c.upper_only();
    if (r1 != 64'hFFFFFFFFFFFFFFFF) failures++;
    if (r2 != 64'hFFFFFFFF00000000) failures++;
  end
endmodule
";
    let sim = simulate(SRC, 100).expect("simulate failed");
    assert_eq!(
        u(&sim, "failures"),
        0,
        "§13.4 bit-select writes to a 64-bit function return variable must land for all 64 bits"
    );
}
