//! IEEE 1800-2023 §8.25: `int items[N]` sized by a class value parameter is a fixed array of each specialization's own size; a `return items[--sp];` in a type-parameterized class evaluates its index once.
//!
//! Expected lines come from the reference simulator.

use xezim::simulate;

fn t_lines(src: &str) -> Vec<String> {
    simulate(src, 1000)
        .expect("simulate failed")
        .output
        .iter()
        .map(|o| o.message.clone())
        .filter(|m| m.starts_with("T|"))
        .collect()
}

const PSIZED: &str = r#"
// top: t8_25a
class SP #(int N = 4); int items[N]; endclass
class Stack #(type T = int, int N = 4);
  T items[N]; int sp;
  function void push(T x); items[sp++] = x; endfunction
  function T pop(); return items[--sp]; endfunction
endclass
module t8_25a;
  SP#(3) p; Stack s;
  initial begin
    p = new; $display("T|a|items=%p size=%0d", p.items, $size(p.items));
    s = new; s.push(10); s.push(20); $display("T|b|pop=%0d", s.pop());
  end
endmodule
"#;

const PSIZED_FAMILY: &str = r#"
class SP #(int N = 4); int items[N]; bit [N-1:0] pk; int md[N][2]; logic [7:0] bytes_a[N];
  function int sz(); return $size(items); endfunction
endclass
class Stack #(type T = int, int N = 4);
  T items[N]; int sp;
  function void push(T x); items[sp++] = x; endfunction
  function T pop(); return items[--sp]; endfunction
endclass
class Q #(int N = 2, int M = 3); int g[N][M]; int h[N-1:0]; endclass
module t;
  SP#(3) p; SP#(5) p5; SP p4; SP#(4) p4e; Stack s; Stack#(byte, 2) s2; Q#(3,2) q;
  initial begin
    p = new; p5 = new; p4 = new; p4e = new;
    $display("T|a|items=%p size=%0d sz=%0d bits=%0d", p.items, $size(p.items), p.sz(), $bits(p.pk));
    $display("T|b|p5=%p size=%0d sz=%0d p4=%0d p4e=%0d", p5.items, $size(p5.items), p5.sz(), $size(p4.items), $size(p4e.items));
    p.items[2] = 7; p5.items[4] = 9; p.md[2][1] = 5;
    foreach (p.items[i]) p.items[i] = i*10;
    $display("T|e|%p", p.items);
    s = new; s.push(10); s.push(20); $display("T|f|pop=%0d", s.pop());
    s2 = new; s2.push(-3); s2.push(100); $display("T|g|pop=%0d pop=%0d sz=%0d", s2.pop(), s2.pop(), $size(s2.items));
    p.pk = '1; $display("T|i|pk=%0d", p.pk);
  end
endmodule
"#;

const POP_ONCE: &str = r#"
class S2 #(type T = int); T items[4]; int sp;
  function void push(T x); items[sp++] = x; endfunction
  function T pop(); return items[--sp]; endfunction
endclass
module t; S2 b; S2#(byte) d; int r;
  initial begin b = new; d = new;
    b.push(10); b.push(20); d.push(-3); d.push(100);
    $display("T|g|%0d sp=%0d", b.pop(), b.sp);
    r = d.pop(); $display("T|h|%0d sp=%0d", r, d.sp);
  end
endmodule
"#;

#[test]
fn audit_repro() {
    let want = ["T|a|items='{0, 0, 0} size=3", "T|b|pop=20"];
    assert_eq!(t_lines(PSIZED), want);
}

#[test]
fn specializations_and_type_param_stack() {
    let want = [
        "T|a|items='{0, 0, 0} size=3 sz=3 bits=3",
        "T|b|p5='{0, 0, 0, 0, 0} size=5 sz=5 p4=4 p4e=4",
        "T|e|'{0, 10, 20}",
        "T|f|pop=20",
        "T|g|pop=100 pop=-3 sz=2",
        "T|i|pk=7",
    ];
    assert_eq!(t_lines(PSIZED_FAMILY), want);
}

#[test]
fn side_effect_index_evaluated_once() {
    let want = ["T|g|20 sp=1", "T|h|100 sp=1"];
    assert_eq!(t_lines(POP_ONCE), want);
}
