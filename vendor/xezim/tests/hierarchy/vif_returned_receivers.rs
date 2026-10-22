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
fn vif_package_call_receiver() {
    check(
        r####"interface bus_if(input logic [7:0] s);
 logic [7:0] data;
 function automatic int read_value(); return s; endfunction
 task automatic put(input int n); data=n; endtask
 modport mp(input s, output data, import read_value, put);
endinterface
package p; virtual bus_if v; int calls; function automatic virtual bus_if get_v(); calls++; return v; endfunction endpackage
module top; logic [7:0] a=5, b=9; bus_if m(a); bus_if n(b); import p::*;

initial begin p::v=m; #1 $display("T|package_call_receiver|%0d calls=%0d",p::get_v().s,p::calls); p::get_v().data=17; $display("T|package_call_receiver|write=%0d calls=%0d",m.data,p::calls); end endmodule
"####,
        &[
            "T|package_call_receiver|5 calls=1",
            "T|package_call_receiver|write=17 calls=2",
        ],
    );
}

#[test]
fn vif_class_call_receiver() {
    check(
        r####"interface bus_if(input logic [7:0] s);
 logic [7:0] data;
 function automatic int read_value(); return s; endfunction
 task automatic put(input int n); data=n; endtask
 modport mp(input s, output data, import read_value, put);
endinterface

module top; logic [7:0] a=5, b=9; bus_if m(a); bus_if n(b);
class holder; virtual bus_if v; int calls; function virtual bus_if get_v(); calls++; return v; endfunction endclass holder o;
initial begin o=new; o.v=n; #1 $display("T|class_call_receiver|%0d calls=%0d",o.get_v().s,o.calls); o.get_v().data=19; $display("T|class_call_receiver|write=%0d calls=%0d",n.data,o.calls); end endmodule
"####,
        &[
            "T|class_call_receiver|9 calls=1",
            "T|class_call_receiver|write=19 calls=2",
        ],
    );
}

#[test]
fn vif_free_call_receiver() {
    check(
        r####"interface bus_if(input logic [7:0] s);
 logic [7:0] data;
 function automatic int read_value(); return s; endfunction
 task automatic put(input int n); data=n; endtask
 modport mp(input s, output data, import read_value, put);
endinterface

module top; logic [7:0] a=5, b=9; bus_if m(a); bus_if n(b);
typedef virtual bus_if handle_t; handle_t v; int calls; function automatic handle_t get_v(); calls++; return v; endfunction
initial begin v=n; #1 $display("T|free_call_receiver|%0d fn=%0d calls=%0d",get_v().s,get_v().read_value(),calls); get_v().put(21); $display("T|free_call_receiver|write=%0d calls=%0d",n.data,calls); end endmodule
"####,
        &[
            "T|free_call_receiver|9 fn=9 calls=2",
            "T|free_call_receiver|write=21 calls=3",
        ],
    );
}

#[test]
fn vif_call_receiver_selects() {
    check(
        r####"interface bus_if(input logic [7:0] s);
 logic [7:0] data;
 function automatic int read_value(); return s; endfunction
 task automatic put(input int n); data=n; endtask
 modport mp(input s, output data, import read_value, put);
endinterface
package p; virtual bus_if.mp v; function automatic virtual bus_if.mp get_v(); return v; endfunction endpackage
module top; logic [7:0] a=5, b=9; bus_if m(a); bus_if n(b); import p::*;

initial begin p::v=n; #1 p::get_v().data=8'ha5; $display("T|call_receiver_selects|bit=%0d slice=%0d",p::get_v().data[7],p::get_v().data[4 +: 4]); p::get_v().data[0]<=0; #1 $display("T|call_receiver_selects|write=%0d",n.data); end endmodule
"####,
        &[
            "T|call_receiver_selects|bit=1 slice=10",
            "T|call_receiver_selects|write=164",
        ],
    );
}

#[test]
fn vif_direct_call_receiver() {
    check(
        r####"interface bus_if(input logic [7:0] s);
 logic [7:0] data;
 function automatic int read_value(); return s; endfunction
 task automatic put(input int n); data=n; endtask
 modport mp(input s, output data, import read_value, put);
endinterface

module top; logic [7:0] a=5, b=9; bus_if m(a); bus_if n(b);
function automatic virtual bus_if get_v(); return m; endfunction
initial begin m.data=7; #1 $display("T|direct_call_receiver|read=%0d",get_v().data); get_v().data=11; $display("T|direct_call_receiver|write=%0d",m.data); get_v().data[0]<=0; #1 $display("T|direct_call_receiver|nba=%0d",m.data); end endmodule
"####,
        &[
            "T|direct_call_receiver|read=7",
            "T|direct_call_receiver|write=11",
            "T|direct_call_receiver|nba=10",
        ],
    );
}
