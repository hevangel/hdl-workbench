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
fn vif_class_assoc() {
    check(
        r####"interface bus_if(input logic [7:0] s);
 logic [7:0] data;
 function automatic int read_value(); return s; endfunction
 task automatic put(input int n); data=n; endtask
 modport mp(input s, output data, import read_value, put);
endinterface

module top; logic [7:0] a=5, b=9; bus_if m(a); bus_if n(b);
class holder; virtual bus_if v[string]; endclass holder o; virtual bus_if local_v;
initial begin o=new; o.v["chosen"]=n; local_v=m; local_v=o.v["chosen"]; #1 $display("T|class_assoc|%0d %0d",o.v["chosen"].s,local_v.s); end endmodule
"####,
        &["T|class_assoc|9 9"],
    );
}

#[test]
fn vif_class_assoc_keys() {
    check(
        r####"interface bus_if(input logic [7:0] s);
 logic [7:0] data;
 function automatic int read_value(); return s; endfunction
 task automatic put(input int n); data=n; endtask
 modport mp(input s, output data, import read_value, put);
endinterface

module top; logic [7:0] a=5, b=9; bus_if m(a); bus_if n(b);
class holder; virtual bus_if v[string]; endclass holder o; virtual bus_if local_v;
initial begin o=new; o.v["first"]=m; o.v["second"]=n; #1 local_v=o.v["first"]; $display("T|class_assoc_keys|%0d %0d copy=%0d",o.v["first"].s,o.v["second"].s,local_v.s); end endmodule
"####,
        &["T|class_assoc_keys|5 9 copy=5"],
    );
}

#[test]
fn vif_class_assoc_methods() {
    check(
        r####"interface bus_if(input logic [7:0] s);
 logic [7:0] data;
 function automatic int read_value(); return s; endfunction
 task automatic put(input int n); data=n; endtask
 modport mp(input s, output data, import read_value, put);
endinterface

module top; logic [7:0] a=5, b=9; bus_if m(a); bus_if n(b);
class holder; virtual bus_if v[string]; function void set_v(string k,virtual bus_if x); v[k]=x; endfunction function int read_v(string k); return v[k].s; endfunction function virtual bus_if get_v(string k); return v[k]; endfunction endclass holder o; virtual bus_if local_v;
initial begin o=new; o.set_v("first",m); o.set_v("second",n); #1 local_v=o.get_v("second"); $display("T|class_assoc_methods|%0d %0d copy=%0d",o.read_v("first"),o.read_v("second"),local_v.s); end endmodule
"####,
        &["T|class_assoc_methods|5 9 copy=9"],
    );
}

#[test]
fn vif_class_assoc_inherited() {
    check(
        r####"interface bus_if(input logic [7:0] s);
 logic [7:0] data;
 function automatic int read_value(); return s; endfunction
 task automatic put(input int n); data=n; endtask
 modport mp(input s, output data, import read_value, put);
endinterface

module top; logic [7:0] a=5, b=9; bus_if m(a); bus_if n(b);
class parent_holder; virtual bus_if.mp v[string]; endclass class holder extends parent_holder; endclass holder o; virtual bus_if.mp local_v; string key;
initial begin o=new; key="first"; o.v[key]=m; key="second"; o.v[key]=n; #1 local_v=o.v[key]; $display("T|class_assoc_inherited|%0d %0d copy=%0d",o.v["first"].s,o.v[key].s,local_v.s); end endmodule
"####,
        &["T|class_assoc_inherited|5 9 copy=9"],
    );
}
