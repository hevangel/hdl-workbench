//! IEEE 1800 §23.11: bare bind targets prefer lexical instances over types.
use xezim::simulate_multi;

fn check(source: &str, expected: &[&str]) {
    let sim = simulate_multi(
        &[source.to_string()],
        10,
        Some("top"),
        &[],
        &[],
        None,
        false,
        None,
        None,
        &[],
        &[],
        None,
        &[],
        0,
        u64::MAX,
        None,
        &[],
        None,
        None,
        None,
        None,
        false,
    )
    .expect("simulate bare bind target");
    let actual: Vec<_> = sim
        .output
        .iter()
        .map(|o| o.message.trim_end())
        .filter(|line| line.starts_with("T|"))
        .collect();
    assert_eq!(actual, expected);
}

#[test]
fn bare_interface() {
    check(
        r###"// IEEE 1800 §23.11: bare bind targets resolve module types or lexical instances.
interface phy;int value=7;endinterface interface spy(input int value);initial #1 $display("T|%m=%0d",value);endinterface
module top;phy h();bind h spy b(.value(value));initial #3 $finish;endmodule
"###,
        &["T|top.h.b=7"],
    );
}

#[test]
fn bare_module() {
    check(
        r###"// IEEE 1800 §23.11: bare bind targets resolve module types or lexical instances.
module unit;int value=7;endmodule module spy(input int value);initial #1 $display("T|%m=%0d",value);endmodule
module top;unit h();bind h spy b(.value(value));initial #3 $finish;endmodule
"###,
        &["T|top.h.b=7"],
    );
}

#[test]
fn bare_nested() {
    check(
        r###"// IEEE 1800 §23.11: bare bind targets resolve module types or lexical instances.
module unit;int value=7;endmodule module spy(input int value);initial #1 $display("T|%m=%0d",value);endmodule
module wrapper;unit h();bind h spy b(.value(value));endmodule module top;wrapper w();initial #3 $finish;endmodule
"###,
        &["T|top.w.h.b=7"],
    );
}

#[test]
fn bare_upward() {
    check(
        r###"// IEEE 1800 §23.11: bare bind targets resolve module types or lexical instances.
module unit;int value=7;endmodule module spy(input int value);initial #1 $display("T|%m=%0d",value);endmodule
module binder;bind h spy b(.value(value));endmodule module top;unit h();binder cfg();initial #3 $finish;endmodule
"###,
        &["T|top.h.b=7"],
    );
}

#[test]
fn bare_generate() {
    check(
        r###"// IEEE 1800 §23.11: bare bind targets resolve module types or lexical instances.
interface phy;int value=7;endinterface interface spy(input int value);initial #1 $display("T|%m=%0d",value);endinterface
module top;if(1)begin:area phy h();bind h spy b(.value(value));end initial #3 $finish;endmodule
"###,
        &["T|top.area.h.b=7"],
    );
}

#[test]
fn bare_nearest() {
    check(
        r###"// IEEE 1800 §23.11: bare bind targets resolve module types or lexical instances.
module unit;int value=7;endmodule module spy(input int value);initial #1 $display("T|%m=%0d",value);endmodule
module wrapper;unit h();bind h spy b(.value(value));endmodule module top;unit h();wrapper w();initial #3 $finish;endmodule
"###,
        &["T|top.w.h.b=7"],
    );
}

#[test]
fn bare_precedence() {
    check(
        r###"// IEEE 1800 §23.11: bare bind targets resolve module types or lexical instances.
module unit;int value=7;endmodule module spy(input int value);initial #1 $display("T|%m=%0d",value);endmodule
module h;int value=9;endmodule module top;unit h();h other();bind h spy b(.value(value));initial #3 $finish;endmodule
"###,
        &["T|top.h.b=7"],
    );
}

#[test]
fn bare_unused_type() {
    check(
        r###"// IEEE 1800 §23.11: bare bind targets resolve module types or lexical instances.
module unit;int value=7;endmodule module spy(input int value);initial #1 $display("T|%m=%0d",value);endmodule
module h;int value=9;endmodule module top;unit h();bind h spy b(.value(value));initial begin #2 $display("T|done");#1 $finish;end endmodule
"###,
        &["T|top.h.b=7", "T|done"],
    );
}
