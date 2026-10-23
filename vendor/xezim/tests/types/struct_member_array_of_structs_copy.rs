//! §7.2/§7.4: copying an unpacked struct whose member is a fixed ARRAY OF
//! unpacked structs (`row_t rows[2]`) lost that member's contents — in a
//! whole-struct assignment (`t2 = t`) and in `push_back` alike.
//!
//! The leaf-by-leaf copier treated each member-array element as a scalar
//! leaf (`t.rows[1]`), but an element that is itself an unpacked struct has
//! no leaf of its own, only `t.rows[1].<member>`; the element was read as
//! nothing and the destination kept x. Such elements now copy member by
//! member. Every expectation below was cross-checked against the reference
//! simulator.

use xezim::simulate;

#[test]
fn nested_struct_array_members_copy() {
    let sim = simulate(
        r#"
typedef struct { int level; string name; } row_t;
typedef struct { int id; row_t rows[2]; int vals[2]; } tab_t;
module top;
  tab_t mq[$];
  tab_t g, g2;
  function automatic void f();
    tab_t t, t2;
    t.id = 9; t.rows[1].level = 3; t.rows[1].name = "x"; t.vals[1] = 7;
    mq.push_back(t);
    t2 = t;
    $display("F %0d %0d '%s' %0d", t2.id, t2.rows[1].level, t2.rows[1].name, t2.vals[1]);
  endfunction
  initial begin
    f();
    $display("M %0d %0d '%s' %0d", mq[0].id, mq[0].rows[1].level, mq[0].rows[1].name, mq[0].vals[1]);
    g.rows[1].level = 4; g.rows[1].name = "y"; g.vals[0] = 8;
    g2 = g;
    $display("G %0d '%s' %0d", g2.rows[1].level, g2.rows[1].name, g2.vals[0]);
    mq.push_back(g);
    $display("Q %0d '%s' %0d", mq[1].rows[1].level, mq[1].rows[1].name, mq[1].vals[0]);
  end
endmodule
"#,
        100,
    )
    .expect("simulate failed");
    let o: Vec<String> = sim.output.iter().map(|o| o.message.clone()).collect();
    for w in ["F 9 3 'x' 7", "M 9 3 'x' 7", "G 4 'y' 8", "Q 4 'y' 8"] {
        assert!(o.iter().any(|l| l == w), "expected {w:?} in:\n{o:#?}");
    }
}
