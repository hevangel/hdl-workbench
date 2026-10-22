//! §25.9 / §25.10 / §26.3: virtual-interface binding regression shapes.
//! Expected results were checked with the reference simulator.

use xezim::simulate;

fn check(source: &str, expected: &[&str]) {
    let sim = simulate(source, 10).expect("simulate virtual interfaces");
    let lines: Vec<_> = sim
        .output
        .iter()
        .map(|o| o.message.trim_end())
        .filter(|line| line.starts_with("T|") || line.starts_with("pkg vif s="))
        .collect();
    assert_eq!(lines, expected);
}

#[test]
fn vif_null_copy() {
    check(
        r####"interface bus_if(input logic [7:0] s);
 logic [7:0] data;
 function automatic int read_value(); return s; endfunction
 task automatic put(input int n); data=n; endtask
 modport mp(input s, output data, import read_value, put);
endinterface
package p; virtual bus_if v; virtual bus_if empty_v; endpackage
module top; logic [7:0] a=5, b=9; bus_if m(a); bus_if n(b); import p::*;

initial begin p::v=m; #1 p::v=p::empty_v; $display("T|null_copy|%0d %0d",p::v==null,p::empty_v==null); end endmodule
"####,
        &["T|null_copy|1 1"],
    );
}

#[test]
fn vif_null_getter() {
    check(
        r####"interface bus_if(input logic [7:0] s);
 logic [7:0] data;
 function automatic int read_value(); return s; endfunction
 task automatic put(input int n); data=n; endtask
 modport mp(input s, output data, import read_value, put);
endinterface
package p; virtual bus_if v; function automatic virtual bus_if empty_v(); return null; endfunction endpackage
module top; logic [7:0] a=5, b=9; bus_if m(a); bus_if n(b); import p::*;

initial begin p::v=m; #1 p::v=p::empty_v(); $display("T|null_getter|%0d",p::v==null); end endmodule
"####,
        &["T|null_getter|1"],
    );
}

#[test]
fn vif_class_null_copy() {
    check(
        r####"interface bus_if(input logic [7:0] s);
 logic [7:0] data;
 function automatic int read_value(); return s; endfunction
 task automatic put(input int n); data=n; endtask
 modport mp(input s, output data, import read_value, put);
endinterface

module top; logic [7:0] a=5, b=9; bus_if m(a); bus_if n(b);
class holder; virtual bus_if v; virtual bus_if empty_v; endclass holder o;
initial begin o=new; o.v=n; #1 o.v=o.empty_v; $display("T|class_null_copy|%0d %0d",o.v==null,o.empty_v==null); end endmodule
"####,
        &["T|class_null_copy|1 1"],
    );
}

#[test]
fn vif_class_null_getter() {
    check(
        r####"interface bus_if(input logic [7:0] s);
 logic [7:0] data;
 function automatic int read_value(); return s; endfunction
 task automatic put(input int n); data=n; endtask
 modport mp(input s, output data, import read_value, put);
endinterface

module top; logic [7:0] a=5, b=9; bus_if m(a); bus_if n(b);
class holder; virtual bus_if v; function virtual bus_if empty_v(); return null; endfunction endclass holder o;
initial begin o=new; o.v=n; #1 o.v=o.empty_v(); $display("T|class_null_getter|%0d",o.v==null); end endmodule
"####,
        &["T|class_null_getter|1"],
    );
}

#[test]
fn vif_frame_null_copy() {
    check(
        r####"interface bus_if(input logic [7:0] s);
 logic [7:0] data;
 function automatic int read_value(); return s; endfunction
 task automatic put(input int n); data=n; endtask
 modport mp(input s, output data, import read_value, put);
endinterface

module top; logic [7:0] a=5, b=9; bus_if m(a); bus_if n(b);
virtual bus_if v; task automatic clear_v(); virtual bus_if local_v; virtual bus_if empty_v; local_v=v; local_v=empty_v; $display("T|frame_null_copy|%0d outer=%0d",local_v==null,v.s); endtask
initial begin v=n; #1 clear_v(); end endmodule
"####,
        &["T|frame_null_copy|1 outer=9"],
    );
}

#[test]
fn vif_frame_null_getter() {
    check(
        r####"interface bus_if(input logic [7:0] s);
 logic [7:0] data;
 function automatic int read_value(); return s; endfunction
 task automatic put(input int n); data=n; endtask
 modport mp(input s, output data, import read_value, put);
endinterface

module top; logic [7:0] a=5, b=9; bus_if m(a); bus_if n(b);
virtual bus_if v; function automatic virtual bus_if empty_v(); return null; endfunction task automatic clear_v(); virtual bus_if local_v; local_v=v; local_v=empty_v(); $display("T|frame_null_getter|%0d outer=%0d",local_v==null,v.s); endtask
initial begin v=n; #1 clear_v(); end endmodule
"####,
        &["T|frame_null_getter|1 outer=9"],
    );
}
