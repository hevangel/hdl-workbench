//! IEEE 1800-2023 §8.15 / §8.4: writes through chained handles of any depth (`n.next.next = new(3)`, `a.b.c.x = 5`), at module scope, in an initial block and inside methods.
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

const CHAIN: &str = r#"
// top: t8_15
class Node; int val; Node next; function new(int v); val = v; endfunction endclass
module t8_15;
  Node n;
  initial begin
    n = new(1); n.next = new(2); n.next.next = new(3);
    $display("T|a|n.val=%0d n.next.val=%0d n.next.next.val=%0d", n.val, n.next.val, n.next.next.val);
    $display("T|b|n.next.next==n %0d", n.next.next == n);
  end
endmodule
"#;

const CHAIN_FAMILY: &str = r#"
class Node; int val; Node next; int x;
  function new(int v); val = v; endfunction
  function void deep(); next.next.next = new(40); next.next.x = 6; endfunction
endclass
class Box; Node n; function new(); n = new(100); endfunction endclass
module t;
  Node n; Box b; Node nn = new(1);
  initial begin
    nn.next = new(2); nn.next.next = new(3);
    n = new(1); n.next = new(2); n.next.next = new(3);
    $display("T|a|n.val=%0d n.next.val=%0d n.next.next.val=%0d mod=%0d", n.val, n.next.val, n.next.next.val, nn.next.next.val);
    $display("T|b|n.next.next==n %0d", n.next.next == n);
    n.next.next.next = new(4); n.next.next.next.x = 5; n.next.next.val = 33;
    $display("T|c|%0d %0d %0d", n.next.next.next.val, n.next.next.next.x, n.next.next.val);
    n.deep(); $display("T|d|%0d %0d", n.next.next.next.val, n.next.next.x);
    b = new; b.n.next = new(7); b.n.next.next = new(8); b.n.next.next.x = 9;
    $display("T|e|%0d %0d %0d", b.n.next.val, b.n.next.next.val, b.n.next.next.x);
    n.next.next.x++; n.next.next.x += 10;
    $display("T|f|%0d", n.next.next.x);
  end
endmodule
"#;

const CHAIN_MODSCOPE: &str = r#"
class Node; int val; Node next; function new(int v); val = v; endfunction endclass
module t;
  Node n = new(1);
  initial begin n.next = new(2); n.next.next = new(3); end
  initial #1 $display("T|a|%0d %0d", n.next.val, n.next.next.val);
endmodule
"#;

const CHAIN_METHOD: &str = r#"
class C; C other; int v; function new(int x = 0); v = x; endfunction
  function void mk(); other = new(1); other.other = new(2); other.other.other = new(3); endfunction endclass
class W; C c; function new(); c = new(9); endfunction endclass
module t; C c; W w2;
  initial begin c = new; c.mk(); w2 = new;
    $display("T|a|%0d %0d %0d null=%0d n2=%0d", c.other.v, c.other.other.v, c.other.other.other.v, c.other.other.other.other == null, w2.c.other == null);
  end
endmodule
"#;

#[test]
fn audit_repro() {
    let want = [
        "T|a|n.val=1 n.next.val=2 n.next.next.val=3",
        "T|b|n.next.next==n 0",
    ];
    assert_eq!(t_lines(CHAIN), want);
}

#[test]
fn depths_and_scopes() {
    let want = [
        "T|a|n.val=1 n.next.val=2 n.next.next.val=3 mod=3",
        "T|b|n.next.next==n 0",
        "T|c|4 5 33",
        "T|d|40 6",
        "T|e|7 8 9",
        "T|f|17",
    ];
    assert_eq!(t_lines(CHAIN_FAMILY), want);
}

#[test]
fn module_scope_handle_init() {
    let want = ["T|a|2 3"];
    assert_eq!(t_lines(CHAIN_MODSCOPE), want);
}

#[test]
fn three_level_write_in_method() {
    let want = ["T|a|1 2 3 null=1 n2=1"];
    assert_eq!(t_lines(CHAIN_METHOD), want);
}
