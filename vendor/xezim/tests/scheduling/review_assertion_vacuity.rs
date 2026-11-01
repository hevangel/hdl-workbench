//! Behavioral regressions for reviewed PR follow-ups (IEEE 1800 §6.21, §11.6, §16.5, §18.12, §20.7).
fn output(src: &str) -> String {
    xezim::simulate(src, 1_000_000)
        .expect("simulation failed")
        .output
        .iter()
        .map(|o| o.message.as_str())
        .collect::<Vec<_>>()
        .join("\n")
}

#[test]
fn and_failed_vacuity() {
    let got = output(
        r###"module top;
 bit clk=0; int passes=0, fails=0;
 always #5 clk=~clk;
 initial $assertvacuousoff;
 a: assert property (@(posedge clk) ((1 |-> 0) and (0 |-> 1)) or (0 |-> 1)) passes++; else fails++;
 initial begin #6; $display("T|passes %0d fails %0d",passes,fails); $finish; end
endmodule
"###,
    );
    assert!(got.contains("T|passes 1 fails 0"), "{got}");
}

#[test]
fn iff_failed_vacuity() {
    let got = output(
        r###"module top;
 bit clk=0; int passes=0, fails=0;
 always #5 clk=~clk;
 initial $assertvacuousoff;
 a: assert property (@(posedge clk) ((1 |-> 0) iff (1 |-> 1)) or (0 |-> 1)) passes++; else fails++;
 initial begin #6; $display("T|passes %0d fails %0d",passes,fails); $finish; end
endmodule
"###,
    );
    assert!(got.contains("T|passes 1 fails 0"), "{got}");
}

#[test]
fn until_failed_vacuity() {
    let got = output(
        r###"module top;
 bit clk=0; int passes=0, fails=0;
 always #5 clk=~clk;
 initial $assertvacuousoff;
 a: assert property (@(posedge clk) (0 until (1 |-> 0)) or (0 |-> 1)) passes++; else fails++;
 initial begin #6; $display("T|passes %0d fails %0d",passes,fails); $finish; end
endmodule
"###,
    );
    assert!(got.contains("T|passes 1 fails 0"), "{got}");
}

#[test]
fn iff_of_two_vacuous_failures_is_vacuous() {
    let got = output(
        r#"module top;
 bit clk=0; int passes=0, fails=0;
 initial $assertvacuousoff;
 a: assert property (@(posedge clk) (not (0 |-> 1)) iff (not (0 |-> 1))) passes++; else fails++;
 initial begin #5 clk=1; #1; $display("T|passes %0d fails %0d",passes,fails); $finish; end
endmodule"#,
    );
    assert!(got.contains("T|passes 0 fails 0"), "{got}");
}
