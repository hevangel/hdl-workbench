//! §20.6.1: `$typename` of a class-typed operand. `this` names the class
//! declaring the running method (not the object's runtime class), and a
//! variable, property or class typedef names its declared class type —
//! qualified by its package and with every parameter listed, defaults
//! included. Only procedural locals were handled; everything else printed
//! `logic`. The expected lines were cross-checked against the reference
//! simulator.

use xezim::simulate;

const SRC: &str = r#"
package p;
  class pc;
    function string me(); return $typename(this); endfunction
  endclass
  class pp #(int N = 2);
    function string me(); return $typename(this); endfunction
  endclass
endpackage
import p::*;
class base;
  int x;
  function string me(); return $typename(this); endfunction
endclass
class derived extends base;
  base inner;
  function string me2(); return $typename(this); endfunction
  function string inner_t(); return $typename(inner); endfunction
endclass
class par #(type T=int, int W=4);
  function string me(); return $typename(this); endfunction
endclass
class holder #(type T=p::pc);
  T h;
  function string ht(); return $typename(h); endfunction
endclass
typedef base base_t;
typedef par #(byte, 3) par_t;
module top;
  base b; derived d; par #(bit, 8) q; pc c; p::pc c2; base_t bt; par_t pt; pp #(5) ppv;
  par #(bit) q1; par q2; par #(p::pc, 2) q3; holder #(par#(bit,1)) h2;
  initial begin
    base lb;
    d = new; q = new; c = new; pt = new; ppv = new; q1 = new; q2 = new; h2 = new;
    $display("1 %s", d.me());
    $display("2 %s", d.me2());
    $display("3 %s", d.inner_t());
    $display("4 %s", q.me());
    $display("5 %s", c.me());
    $display("6 %s", $typename(b));
    $display("7 %s", $typename(c));
    $display("8 %s", $typename(c2));
    $display("9 %s", $typename(bt));
    $display("10 %s", $typename(pt));
    $display("11 %s", pt.me());
    $display("12 %s", $typename(lb));
    $display("13 %s", $typename(d.inner));
    $display("14 %s", ppv.me());
    $display("15 %s", $typename(ppv));
    $display("16 %s", $typename(derived));
    $display("17 %s", $typename(base_t));
    $display("18 %s", $typename(q1));
    $display("19 %s", q1.me());
    $display("20 %s", $typename(q2));
    $display("21 %s", q2.me());
    $display("22 %s", $typename(q3));
    $display("23 %s", h2.ht());
    $display("24 %s", $typename(h2));
  end
endmodule
"#;

#[test]
fn typename_of_class_operands() {
    let out: Vec<String> = simulate(SRC, 1000)
        .expect("simulate failed")
        .output
        .iter()
        .map(|o| o.message.clone())
        .collect();
    let want = [
        "1 class base",
        "2 class derived",
        "3 class base",
        "4 class par #(bit, 8)",
        "5 class p::pc",
        "6 class base",
        "7 class p::pc",
        "8 class p::pc",
        "9 class base",
        "10 class par #(byte, 3)",
        "11 class par #(byte, 3)",
        "12 class base",
        "13 class base",
        "14 class p::pp #(5)",
        "15 class p::pp #(5)",
        "16 class derived",
        "17 class base",
        "18 class par #(bit, 4)",
        "19 class par #(bit, 4)",
        "20 class par #(int, 4)",
        "21 class par #(int, 4)",
        "22 class par #(class p::pc, 2)",
        "23 class par #(bit, 1)",
        "24 class holder #(class par #(bit, 1))",
    ];
    assert_eq!(out, want, "{out:?}");
}
