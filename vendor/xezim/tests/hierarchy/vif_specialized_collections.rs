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
fn vif_static_queue() {
    check(
        r####"interface bus_if(input logic [7:0] s);
 logic [7:0] data;
 function automatic int read_value(); return s; endfunction
 task automatic put(input int n); data=n; endtask
 modport mp(input s, output data, import read_value, put);
endinterface

module top; logic [7:0] a=5, b=9; bus_if m(a); bus_if n(b);
class store; static virtual bus_if v[$]; endclass virtual bus_if local_v;
initial begin store::v.push_back(m); store::v.push_back(n); #1 local_v=store::v[1]; $display("T|static_queue|%0d copy=%0d",store::v[1].s,local_v.s); end endmodule
"####,
        &["T|static_queue|9 copy=9"],
    );
}

#[test]
fn vif_parameterized_queue() {
    check(
        r####"interface bus_if(input logic [7:0] s);
 logic [7:0] data;
 function automatic int read_value(); return s; endfunction
 task automatic put(input int n); data=n; endtask
 modport mp(input s, output data, import read_value, put);
endinterface

module top; logic [7:0] a=5, b=9; bus_if m(a); bus_if n(b);
class store #(type T=int); static T v[$]; static function void put(T x); v.push_back(x); endfunction endclass virtual bus_if local_v;
initial begin store#(virtual bus_if)::put(m); store#(virtual bus_if)::put(n); #1 local_v=store#(virtual bus_if)::v[1]; $display("T|parameterized_queue|%0d copy=%0d",store#(virtual bus_if)::v[1].s,local_v.s); end endmodule
"####,
        &["T|parameterized_queue|9 copy=9"],
    );
}

#[test]
fn vif_queue_typedef_alias() {
    check(
        r####"interface bus_if(input logic [7:0] s);
 logic [7:0] data;
 function automatic int read_value(); return s; endfunction
 task automatic put(input int n); data=n; endtask
 modport mp(input s, output data, import read_value, put);
endinterface

module top; logic [7:0] a=5, b=9; bus_if m(a); bus_if n(b);
typedef virtual bus_if handle_t; class store #(type T=int); static T v[$]; static function void put(T x); v.push_back(x); endfunction endclass typedef store#(handle_t) store_v; virtual bus_if local_v;
initial begin store_v::put(m); store_v::put(n); #1 local_v=store_v::v[1]; $display("T|queue_typedef_alias|%0d copy=%0d",store_v::v[1].s,local_v.s); end endmodule
"####,
        &["T|queue_typedef_alias|9 copy=9"],
    );
}

#[test]
fn vif_queue_named_type() {
    check(
        r####"interface bus_if(input logic [7:0] s);
 logic [7:0] data;
 function automatic int read_value(); return s; endfunction
 task automatic put(input int n); data=n; endtask
 modport mp(input s, output data, import read_value, put);
endinterface

module top; logic [7:0] a=5, b=9; bus_if m(a); bus_if n(b);
class store #(type T=int); static T v[$]; static function void put(T x); v.push_back(x); endfunction endclass virtual bus_if local_v;
initial begin store#(.T(virtual bus_if))::put(m); store#(.T(virtual bus_if))::put(n); #1 local_v=store#(.T(virtual bus_if))::v[1]; $display("T|queue_named_type|%0d copy=%0d",store#(.T(virtual bus_if))::v[1].s,local_v.s); end endmodule
"####,
        &["T|queue_named_type|9 copy=9"],
    );
}

#[test]
fn vif_queue_reorder() {
    check(
        r####"interface bus_if(input logic [7:0] s);
 logic [7:0] data;
 function automatic int read_value(); return s; endfunction
 task automatic put(input int n); data=n; endtask
 modport mp(input s, output data, import read_value, put);
endinterface

module top; logic [7:0] a=5, b=9; bus_if m(a); bus_if n(b);
class store #(type T=int); static T v[$]; static function void put(T x); v.push_back(x); endfunction endclass virtual bus_if local_v;
initial begin store#(virtual bus_if)::put(m); store#(virtual bus_if)::put(n); store#(int)::put(37); #1 local_v=store#(virtual bus_if)::v[1]; store#(virtual bus_if)::v.push_front(n); $display("T|queue_reorder|%0d %0d copy=%0d scalar=%0d",store#(virtual bus_if)::v[0].s,store#(virtual bus_if)::v[1].s,local_v.s,store#(int)::v[0]); end endmodule
"####,
        &["T|queue_reorder|9 5 copy=9 scalar=37"],
    );
}

#[test]
fn vif_queue_static_method() {
    check(
        r####"interface bus_if(input logic [7:0] s);
 logic [7:0] data;
 function automatic int read_value(); return s; endfunction
 task automatic put(input int n); data=n; endtask
 modport mp(input s, output data, import read_value, put);
endinterface

module top; logic [7:0] a=5, b=9; bus_if m(a); bus_if n(b);
class store #(type T=int); static T v[$]; static function void put(T x); v.push_back(x); endfunction static function int read_v(int i); return v[i].s; endfunction static function T get_v(int i); return v[i]; endfunction endclass virtual bus_if local_v;
initial begin store#(virtual bus_if)::put(m); store#(virtual bus_if)::put(n); #1 local_v=store#(virtual bus_if)::get_v(0); $display("T|queue_static_method|%0d copy=%0d",store#(virtual bus_if)::read_v(1),local_v.s); end endmodule
"####,
        &["T|queue_static_method|9 copy=5"],
    );
}
