//! `randomize()` must never return 1 with a constraint violated (§18.6.1).
//! The trial loop's acceptance check skipped every `foreach` whose array is
//! not a rand collection or a rand fixed array, and every `foreach` nested
//! in another loop's body: a loop over a state vector (`bit [3:0] it;`),
//! a state queue, dynamic array or packed array was never judged, so a
//! trial that left its body violated was accepted (§18.5.8.1). Each loop
//! is now judged element by element, and the joint solver translates a
//! loop over a state queue or dynamic array one index at a time.
//!
//! Expected values are the reference simulator's: every call succeeds with
//! the constraints met.

use xezim::simulate;

const SRC_STATE: &str = r#"
typedef struct packed { bit [3:0] a; bit [31:0] pad; } s_t;
class p1; rand s_t r; int q[$] = '{0,1,2,3};
  constraint c { foreach (q[i]) r.pad[q[i]*8 + 4] == 1'b1; } endclass
class p2; rand s_t r; int fa[4] = '{0,1,2,3};
  constraint c { foreach (fa[i]) r.pad[fa[i]*8 + 4] == 1'b1; } endclass
class p3; rand s_t r; bit [3:0] it;
  constraint c { foreach (it[i]) r.pad[i*8 + 4] == 1'b1; } endclass
class p4; rand s_t r; int da[]; function new(); da = new[4]; endfunction
  constraint c { foreach (da[i]) r.pad[i*8 + 4] == 1'b1; } endclass
class p5; rand bit [1:0] x; int q[$] = '{0,1,2};
  constraint c { foreach (q[i]) x != q[i]; } endclass
class p6; rand s_t r; bit [3:0][1:0] it2;
  constraint c { foreach (it2[i]) r.pad[i*8 + 4] == 1'b1; } endclass
class p7; rand s_t r; bit [0:3] it;
  constraint c { foreach (it[i]) r.pad[i*8 + 4] == 1'b1; } endclass
module top; initial begin
  p1 o1 = new(); p2 o2 = new(); p3 o3 = new(); p4 o4 = new(); p5 o5 = new(); p6 o6 = new(); p7 o7 = new();
  for (int t = 0; t < 3; t++) begin
    int r;
    r = o1.randomize(); $display("p1 r=%0d sat=%0d", r, o1.r.pad[4] && o1.r.pad[12] && o1.r.pad[20] && o1.r.pad[28]);
    r = o2.randomize(); $display("p2 r=%0d sat=%0d", r, o2.r.pad[4] && o2.r.pad[12] && o2.r.pad[20] && o2.r.pad[28]);
    r = o3.randomize(); $display("p3 r=%0d sat=%0d", r, o3.r.pad[4] && o3.r.pad[12] && o3.r.pad[20] && o3.r.pad[28]);
    r = o4.randomize(); $display("p4 r=%0d sat=%0d", r, o4.r.pad[4] && o4.r.pad[12] && o4.r.pad[20] && o4.r.pad[28]);
    r = o5.randomize(); $display("p5 r=%0d x=%0d", r, o5.x);
    r = o6.randomize(); $display("p6 r=%0d sat=%0d", r, o6.r.pad[4] && o6.r.pad[12] && o6.r.pad[20] && o6.r.pad[28]);
    r = o7.randomize(); $display("p7 r=%0d sat=%0d", r, o7.r.pad[4] && o7.r.pad[12] && o7.r.pad[20] && o7.r.pad[28]);
  end
end endmodule
"#;

const SRC_NESTED: &str = r#"
typedef struct packed { bit [3:0] a; bit [31:0] pad; } s_t;
class b1; rand bit [3:0] x; int q[$] = '{1,2}; constraint c { x < q.size(); } endclass
class b2; rand bit [7:0] x; int q[$] = '{1,2}; constraint c { x == q.sum(); } endclass
class b4; rand bit [7:0] x; int q[$] = '{5,9}; constraint c { x inside {q}; } endclass
class b5; rand bit [1:0] x, y; constraint c { unique {x, y}; x < 2; y < 2; } endclass
class b6; rand s_t r; bit [1:0] it; bit [1:0] jt;
  constraint c { foreach (it[i]) foreach (jt[j]) r.pad[i*8 + j*2 + 4] == 1'b1; } endclass
class b7; rand s_t r; bit [1:0] it; constraint c { if (1) { foreach (it[i]) r.pad[i*8 + 4] == 1'b1; } } endclass
class b8; rand bit [7:0] x; int q[$] = '{3,4}; constraint c { x == q[1] * 2 + q.size(); } endclass
class b9; rand bit [7:0] a[2]; bit [1:0] jt; constraint c { foreach (a[i]) foreach (jt[j]) a[i][j*2+1] == 1'b1; } endclass
module top; initial begin
  b1 o1 = new(); b2 o2 = new(); b4 o4 = new(); b5 o5 = new(); b6 o6 = new(); b7 o7 = new(); b8 o8 = new(); b9 o9 = new();
  for (int t = 0; t < 3; t++) begin
    int r;
    r = o1.randomize(); $display("b1 r=%0d ok=%0d", r, o1.x < 2);
    r = o2.randomize(); $display("b2 r=%0d ok=%0d", r, o2.x == 3);
    r = o4.randomize(); $display("b4 r=%0d ok=%0d", r, o4.x inside {5,9});
    r = o5.randomize(); $display("b5 r=%0d ok=%0d", r, o5.x != o5.y);
    r = o6.randomize(); $display("b6 r=%0d ok=%0d", r, o6.r.pad[4] && o6.r.pad[6] && o6.r.pad[12] && o6.r.pad[14]);
    r = o7.randomize(); $display("b7 r=%0d ok=%0d", r, o7.r.pad[4] && o7.r.pad[12]);
    r = o8.randomize(); $display("b8 r=%0d ok=%0d", r, o8.x == 10);
    r = o9.randomize(); $display("b9 r=%0d ok=%0d", r, o9.a[0][1] && o9.a[0][3] && o9.a[1][1] && o9.a[1][3]);
  end
end endmodule
"#;

fn lines(src: &str) -> Vec<String> {
    simulate(src, 100)
        .expect("simulate failed")
        .output
        .iter()
        .map(|o| o.message.clone())
        .filter(|m| m.contains(" r="))
        .collect()
}

/// Loops over state members: a queue, a fixed array, a vector (both
/// directions), a dynamic array and a packed array; `p5` is a plain `x !=
/// q[i]` over a state queue.
#[test]
fn randomize_judges_foreach_over_state_members() {
    let mut expected = Vec::new();
    for _ in 0..3 {
        for c in ["p1", "p2", "p3", "p4"] {
            expected.push(format!("{c} r=1 sat=1"));
        }
        expected.push("p5 r=1 x=3".to_string());
        for c in ["p6", "p7"] {
            expected.push(format!("{c} r=1 sat=1"));
        }
    }
    assert_eq!(lines(SRC_STATE), expected);
}

/// Nested loops (`b6`, `b9`) and neighbouring shapes the check already
/// judged.
#[test]
fn randomize_judges_nested_foreach() {
    let mut expected = Vec::new();
    for _ in 0..3 {
        for c in ["b1", "b2", "b4", "b5", "b6", "b7", "b8", "b9"] {
            expected.push(format!("{c} r=1 ok=1"));
        }
    }
    assert_eq!(lines(SRC_NESTED), expected);
}
