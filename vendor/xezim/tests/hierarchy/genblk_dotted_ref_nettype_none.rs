//! §6.10/§23.6: a dotted reference rooted at a generate block label
//! (`Z.A.x`, `assign B.x = 0;`) names a scope, never an implicit net. The
//! implicit-net pass collected the root of every dotted name in a continuous
//! assignment, so under `default_nettype none` it rejected the design
//! ("Implicit net 'Z'", yosys `simple_genblk_dive`) and otherwise created a
//! phantom 1-bit net named after the label. Cross-checked against the
//! reference simulator.

fn lines(src: &str) -> Vec<String> {
    xezim::simulate(src, 10)
        .expect("simulate")
        .output
        .iter()
        .map(|o| o.message.clone())
        .collect()
}

#[test]
fn dotted_reference_through_labels_under_nettype_none() {
    let o = lines(
        r#"
`default_nettype none
module genblk_dive_top(output wire x);
  generate
    if (1) begin : Z
      if (1) begin : A
        wire x;
        if (1) begin : B
          wire x;
          if (1) begin : C
            wire x;
            assign B.x = 0;
            wire z = A.B.C.x;
          end
          assign A.x = A.B.C.x;
        end
        assign B.C.x = B.x;
      end
    end
  endgenerate
  assign x = Z.A.x;
endmodule
module tb;
  wire y;
  genblk_dive_top d(.x(y));
  initial #1 $display("D|%b", y);
endmodule
"#,
    );
    assert!(o.iter().any(|l| l == "D|0"), "{o:?}");
}
