//! §7.8: writes into a CLASS-MEMBER associative array of unpacked structs
//! keyed by a string (`aa["k"] = r;`, `aa["k"].level = 6;` in a method)
//! landed on the key's byte value (`aa[107]`), while every read keyed the
//! element `aa[k]` — so each read came back empty and `foreach (aa[k])`
//! listed the keys as numbers.
//!
//! The flat element name only used the canonical key format when the bare
//! collection name was registered as associative; a class member is
//! registered only under its instance-scoped name (`<h>#aa`), so the key was
//! rendered as an integer. The element name now takes its key format from
//! the store it resolves to. Every expectation below was cross-checked
//! against the reference simulator.

use xezim::simulate;

#[test]
fn string_keyed_struct_elements_round_trip() {
    let sim = simulate(
        r#"
typedef struct { int level; string name; } row_t;
class C;
  row_t aa[string];
  row_t ai[int];
  row_t q[$];
  function void run();
    row_t r;
    r.level = 4; r.name = "four";
    q.push_back(r);
    aa["k"] = r;
    $display("A1 %0d '%s' %0d", aa["k"].level, aa["k"].name, aa.num());
    aa["j"] = q[0];
    $display("A2 %0d '%s'", aa["j"].level, aa["j"].name);
    r = aa["k"];
    $display("A3 %0d '%s'", r.level, r.name);
    ai[3] = r;
    $display("A4 %0d '%s' %0d", ai[3].level, ai[3].name, ai.num());
    aa["k"].level = 6;
    $display("A5 %0d", aa["k"].level);
    foreach (aa[k]) $display("K '%s' %0d '%s'", k, aa[k].level, aa[k].name);
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
    let want = [
        "A1 4 'four' 1",
        "A2 4 'four'",
        "A3 4 'four'",
        "A4 4 'four' 1",
        "A5 6",
        "K 'j' 4 'four'",
        "K 'k' 6 'four'",
    ];
    let got: Vec<&String> = o.iter().filter(|l| !l.starts_with("Simulation")).collect();
    assert_eq!(got, want.iter().collect::<Vec<_>>(), "{o:?}");
}
