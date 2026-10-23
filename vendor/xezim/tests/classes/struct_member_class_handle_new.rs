//! §7.2/§8.4: `s.h = new` where `h` is a CLASS-HANDLE member of an unpacked
//! struct constructs the member's class — for a function-local struct, a
//! nested struct member and a module-scope struct alike. The struct's member
//! leaves carry no type name, so the constructor found no class, `s.h` stayed
//! null and `s.h.v` read 0 or x. Expected output cross-checked against the
//! reference simulator.

use xezim::simulate;

const SRC: &str = r#"
module tb;
  class H; int v = 7; endclass
  typedef struct { H h; int k; } S;
  typedef struct { S in; int n; } N;
  S gx;
  function automatic int f1();
    S x;
    x.h = new;
    return x.h.v;
  endfunction
  function automatic int f2();
    N y;
    y.in.h = new;
    y.in.h.v = 9;
    return y.in.h.v;
  endfunction
  function automatic int f3();
    S x;
    H t;
    x.h = new;
    t = x.h;
    return (t == null) ? -1 : t.v;
  endfunction
  initial begin
    S y;
    $display("f1=%0d f2=%0d f3=%0d", f1(), f2(), f3());
    gx.h = new;
    $display("g=%0d", gx.h.v);
    y.h = new;
    $display("y=%0d", y.h.v);
  end
endmodule
"#;

#[test]
fn new_on_struct_class_member() {
    let sim = simulate(SRC, 100).expect("simulate failed");
    let msgs: Vec<String> = sim.output.iter().map(|o| o.message.clone()).collect();
    assert_eq!(msgs, ["f1=7 f2=9 f3=7", "g=7", "y=7"]);
}
