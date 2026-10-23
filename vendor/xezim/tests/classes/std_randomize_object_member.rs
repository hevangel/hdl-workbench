//! §18.12: `std::randomize(obj.member) with {...}` — a class property named
//! through its handle is a target of the scope randomization. The argument
//! was collected as a target only when it was a plain identifier, so the
//! `with` constraints narrowed nothing and the acceptance check failed:
//! randomize returned 0 and the members kept unconstrained draws. The
//! expected lines were cross-checked against the reference simulator.

use xezim::simulate;

const SRC: &str = r#"
class fld;
  rand bit [7:0] value;
  bit [7:0] v2;
endclass
class outer;
  fld f;
  function new(); f = new; endfunction
endclass
module top;
  fld f; outer o;
  int ok, bad;
  initial begin
    f = new; o = new;
    bad = 0;
    repeat (20) begin
      ok = std::randomize(f.value) with { f.value < 10; f.value > 3; };
      if (!ok || f.value < 4 || f.value > 9) bad++;
      ok = std::randomize(f.v2) with { f.v2 inside {[20:22]}; };
      if (!ok || f.v2 < 20 || f.v2 > 22) bad++;
      ok = std::randomize(o.f.value) with { o.f.value == 42; };
      if (!ok || o.f.value != 42) bad++;
      ok = std::randomize(f.value, f.v2) with { f.value + f.v2 == 30; f.value == 10; };
      if (!ok || f.value != 10 || f.v2 != 20) bad++;
    end
    $display("bad=%0d", bad);
    ok = std::randomize(f.value) with { f.value > 250; f.value < 5; };
    $display("unsat ok=%0d", ok);
  end
endmodule
"#;

#[test]
fn std_randomize_object_member_targets() {
    let out: Vec<String> = simulate(SRC, 1000)
        .expect("simulate failed")
        .output
        .iter()
        .map(|o| o.message.clone())
        .collect();
    assert_eq!(out, ["bad=0", "unsat ok=0"], "{out:?}");
}
