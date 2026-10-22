//! §7.4: an ARRAY member of a struct element of a class-property queue
//! (`tq[0].vals[1]`, `tq[0].rows[1].level` in a method) read 0 and swallowed
//! writes.
//!
//! The element's leaves live under the instance (`<h>#tq[0].vals[1]`), but
//! the member-array element paths only looked the flat name up (`tq[0]` has
//! no instance prefix) and, for a struct element of the member array, only
//! resolved the collection at the LAST select (`tq[0].rows`, which names no
//! collection). Both now resolve the collection named by the first select.
//! Every expectation below was cross-checked against the reference simulator.

use xezim::simulate;

#[test]
fn member_arrays_of_queue_elements_read_and_write() {
    let sim = simulate(
        r#"
typedef struct { int level; string name; } row_t;
typedef struct { int id; row_t rows[2]; int vals[2]; } tab_t;
class C;
  tab_t tq[$];
  function void run();
    tab_t t;
    t.rows[1].level = 3; t.vals[1] = 7; t.id = 9;
    $display("CL %0d %0d %0d", t.rows[1].level, t.vals[1], t.id);
    tq.push_back(t);
    tq[0].vals[0] = 5; tq[0].rows[0].level = 6;
    $display("CW %0d %0d", tq[0].vals[0], tq[0].rows[0].level);
    $display("CQ %0d %0d %0d", tq[0].rows[1].level, tq[0].vals[1], tq[0].id);
  endfunction
endclass
module top;
  initial begin
    C c = new;
    c.run();
  end
endmodule
"#,
        100,
    )
    .expect("simulate failed");
    let o: Vec<String> = sim.output.iter().map(|o| o.message.clone()).collect();
    for w in ["CL 3 7 9", "CW 5 6", "CQ 3 7 9"] {
        assert!(o.iter().any(|l| l == w), "expected {w:?} in:\n{o:#?}");
    }
}
