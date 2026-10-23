//! `always @(m[i])` on a small array must fire even when the design also
//! declares an array too large for per-element names (over 100,000 cells).
//!
//! Array cells took their signal ids in name order, and a large array's cells
//! get no entry in `id_to_name`. Every array allocated AFTER it (`m1` sorts
//! after `big`) therefore got ids past the end of `id_to_name`, which the
//! simulator treats as "unnamed large-array cell": no edge snapshot, no
//! sensitivity, so the process never woke. Renaming the array to sort before
//! `big` made the bug disappear. Expected lines are the reference simulator's.

use xezim::simulate;

fn lines(src: &str) -> Vec<String> {
    let sim = simulate(src, 100).expect("simulate");
    sim.output
        .iter()
        .map(|o| o.message.clone())
        .filter(|m| m.starts_with("E|"))
        .collect()
}

const SRC: &str = r#"
module top;
  logic [7:0] m1 [3:0];
  logic [3:0] big [0:199999];
  int m2d [0:1][0:1];
  wire [7:0] w2 = m1[1];
  int hits = 0, whits = 0;
  always @(m1[2]) begin
    hits = hits + 1;
    $display("E|%0t fire m1[2]=%h", $time, m1[2]);
  end
  always @(w2) whits = whits + 1;
  initial begin
    for (int i = 0; i < 4; i++) m1[i] = 8'h10 + i;
    big[150000] = 4'ha;
    m2d[1][0] = 7;
    #1 $display("E|H0 hits=%0d", hits);
    m1[2] = 8'h55;
    #1 $display("E|H1 hits=%0d", hits);
    for (int i = 0; i < 4; i++) m1[i] = 8'h20 + i;
    #1 $display("E|H2 hits=%0d whits=%0d w2=%h big=%h m2d=%0d",
                hits, whits, w2, big[150000], m2d[1][0]);
    $finish;
  end
endmodule
"#;

#[test]
fn element_sensitivity_survives_an_unnamed_large_array() {
    assert_eq!(
        lines(SRC),
        [
            "E|0 fire m1[2]=12",
            "E|H0 hits=1",
            "E|1 fire m1[2]=55",
            "E|H1 hits=2",
            "E|2 fire m1[2]=22",
            "E|H2 hits=3 whits=2 w2=21 big=a m2d=7",
        ]
    );
}

/// The same design with the small array sorting BEFORE the large one always
/// worked; kept as the control.
#[test]
fn element_sensitivity_before_the_large_array_is_unchanged() {
    let src = SRC.replace("m1", "a1");
    assert_eq!(
        lines(&src),
        [
            "E|0 fire a1[2]=12",
            "E|H0 hits=1",
            "E|1 fire a1[2]=55",
            "E|H1 hits=2",
            "E|2 fire a1[2]=22",
            "E|H2 hits=3 whits=2 w2=21 big=a m2d=7",
        ]
    );
}
