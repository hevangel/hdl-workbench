//! IEEE 1800 §11.4.10: arithmetic shifts replicate four-state sign bits.
use xezim::simulate;
fn check(source: &str, expected: &[&str]) {
    let result = simulate(source, 100).unwrap();
    let actual: Vec<_> = result
        .output
        .iter()
        .map(|o| o.message.as_str())
        .filter(|s| s.starts_with("T|"))
        .collect();
    assert_eq!(actual, expected);
}

#[test]
fn signed_unknown_sign_bits_are_replicated() {
    check(
        r#"module top;logic signed[7:0] x=8'bx0000000,z=8'bz0000000;
initial $display("T|sign %b %b",x>>>1,z>>>1);endmodule"#,
        &["T|sign xx000000 zz000000"],
    );
}

#[test]
fn overshifts_repeat_each_sign_state() {
    check(
        r#"module top;logic signed[7:0] a=0,b=-1,x=8'bx0000000,z=8'bz0000000;
initial begin $display("T|at %b %b %b %b",a>>>8,b>>>8,x>>>8,z>>>8);
$display("T|beyond %b %b",x>>>9,z>>>9);end endmodule"#,
        &[
            "T|at 00000000 11111111 xxxxxxxx zzzzzzzz",
            "T|beyond xxxxxxxx zzzzzzzz",
        ],
    );
}

#[test]
fn wide_sign_extension_matches_inline_sign_extension() {
    check(
        r#"module top;logic signed[127:0] x={1'bx,127'd0},z={1'bz,127'd0};
logic signed[127:0] rx,rz;
initial begin rx=x>>>1;rz=z>>>1;
$display("T|wide_sign %b %b",rx[127:126],rz[127:126]);end endmodule"#,
        &["T|wide_sign xx zz"],
    );
}

#[test]
fn unknown_amounts_and_unsigned_left_operands() {
    check(
        r#"module top;logic signed[7:0] a=-10;logic[3:0] q=4'b000x;
logic[7:0] x=8'bx0000000,z=8'bz0000000;
initial begin $display("T|amount %b",a>>>q);
$display("T|logical %b %b",x>>>1,z>>>1);end endmodule"#,
        &["T|amount xxxxxxxx", "T|logical 0x000000 0z000000"],
    );
}

#[test]
fn one_bit_and_zero_distance_boundaries() {
    check(
        r#"module top;logic signed[0:0] x=1'bx,z=1'bz,a=1'b1,b=1'b0;
initial $display("T|single %b %b %b %b %b %b",x>>>0,x>>>1,z>>>1,a>>>1,b>>>1,z>>>0);
endmodule"#,
        &["T|single x x z 1 0 z"],
    );
}
