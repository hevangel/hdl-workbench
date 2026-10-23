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
fn vif_direct() {
    check(
        r####"interface bus_if(input logic [7:0] s);
 logic [7:0] data;
 function automatic int read_value(); return s; endfunction
 task automatic put(input int n); data=n; endtask
 modport mp(input s, output data, import read_value, put);
endinterface
package p; virtual bus_if v; task show(); $display("T|direct|%0d",v.s); endtask endpackage
module top; logic [7:0] a=5, b=9; bus_if m(a); bus_if n(b); import p::*;

initial begin p::v=m; #1 show(); end endmodule
"####,
        &["T|direct|5"],
    );
}

#[test]
fn vif_explicit() {
    check(
        r####"interface bus_if(input logic [7:0] s);
 logic [7:0] data;
 function automatic int read_value(); return s; endfunction
 task automatic put(input int n); data=n; endtask
 modport mp(input s, output data, import read_value, put);
endinterface
package p; virtual interface bus_if v; task show(); $display("T|explicit|%0d",v.s); endtask endpackage
module top; logic [7:0] a=5, b=9; bus_if m(a); bus_if n(b); import p::*;

initial begin p::v=m; #1 show(); end endmodule
"####,
        &["T|explicit|5"],
    );
}

#[test]
fn vif_typedef() {
    check(
        r####"interface bus_if(input logic [7:0] s);
 logic [7:0] data;
 function automatic int read_value(); return s; endfunction
 task automatic put(input int n); data=n; endtask
 modport mp(input s, output data, import read_value, put);
endinterface
package p; typedef virtual bus_if handle_t; handle_t v; task show(); $display("T|typedef|%0d",v.s); endtask endpackage
module top; logic [7:0] a=5, b=9; bus_if m(a); bus_if n(b); import p::*;

initial begin p::v=m; #1 show(); end endmodule
"####,
        &["T|typedef|5"],
    );
}

#[test]
fn vif_alias_chain() {
    check(
        r####"interface bus_if(input logic [7:0] s);
 logic [7:0] data;
 function automatic int read_value(); return s; endfunction
 task automatic put(input int n); data=n; endtask
 modport mp(input s, output data, import read_value, put);
endinterface
package p; typedef virtual bus_if handle_t; typedef handle_t alias_t; alias_t v; task show(); $display("T|alias_chain|%0d",v.s); endtask endpackage
module top; logic [7:0] a=5, b=9; bus_if m(a); bus_if n(b); import p::*;

initial begin p::v=m; #1 show(); end endmodule
"####,
        &["T|alias_chain|5"],
    );
}

#[test]
fn vif_modport() {
    check(
        r####"interface bus_if(input logic [7:0] s);
 logic [7:0] data;
 function automatic int read_value(); return s; endfunction
 task automatic put(input int n); data=n; endtask
 modport mp(input s, output data, import read_value, put);
endinterface
package p; virtual bus_if.mp v; task show(); $display("T|modport|%0d",v.s); endtask endpackage
module top; logic [7:0] a=5, b=9; bus_if m(a); bus_if n(b); import p::*;

initial begin p::v=m; #1 show(); end endmodule
"####,
        &["T|modport|5"],
    );
}

#[test]
fn vif_wildcard() {
    check(
        r####"interface bus_if(input logic [7:0] s);
 logic [7:0] data;
 function automatic int read_value(); return s; endfunction
 task automatic put(input int n); data=n; endtask
 modport mp(input s, output data, import read_value, put);
endinterface
package p; virtual bus_if v; endpackage
module top; logic [7:0] a=5, b=9; bus_if m(a); bus_if n(b); import p::*;

initial begin v=m; #1 $display("T|wildcard|%0d",v.s); end endmodule
"####,
        &["T|wildcard|5"],
    );
}

#[test]
fn vif_explicit_import() {
    check(
        r####"interface bus_if(input logic [7:0] s);
 logic [7:0] data;
 function automatic int read_value(); return s; endfunction
 task automatic put(input int n); data=n; endtask
 modport mp(input s, output data, import read_value, put);
endinterface
package p; virtual bus_if v; endpackage
module top; logic [7:0] a=5, b=9; bus_if m(a); bus_if n(b); import p::v;

initial begin v=m; #1 $display("T|explicit_import|%0d",v.s); end endmodule
"####,
        &["T|explicit_import|5"],
    );
}

#[test]
fn vif_qualified_copy() {
    check(
        r####"interface bus_if(input logic [7:0] s);
 logic [7:0] data;
 function automatic int read_value(); return s; endfunction
 task automatic put(input int n); data=n; endtask
 modport mp(input s, output data, import read_value, put);
endinterface
package p; virtual bus_if v; endpackage
module top; logic [7:0] a=5, b=9; bus_if m(a); bus_if n(b);
virtual bus_if local_v;
initial begin p::v=m; local_v=p::v; #1 $display("T|qualified_copy|%0d %0d %0d",p::v.s,local_v.s,p::v==local_v); p::v=null; $display("T|qualified_copy|null=%0d copy=%0d",p::v==null,local_v.s); end endmodule
"####,
        &["T|qualified_copy|5 5 1", "T|qualified_copy|null=1 copy=5"],
    );
}

#[test]
fn vif_package_calls() {
    check(
        r####"interface bus_if(input logic [7:0] s);
 logic [7:0] data;
 function automatic int read_value(); return s; endfunction
 task automatic put(input int n); data=n; endtask
 modport mp(input s, output data, import read_value, put);
endinterface
package p; virtual bus_if v; function automatic virtual bus_if get_v(); return v; endfunction task automatic set_v(input virtual bus_if x); v=x; endtask task automatic show(); $display("T|package_calls|%0d %0d",v.s,v.read_value()); v.put(17); endtask endpackage
module top; logic [7:0] a=5, b=9; bus_if m(a); bus_if n(b);
virtual bus_if local_v;
initial begin p::set_v(m); #1 p::show(); local_v=p::get_v(); $display("T|package_calls|copy=%0d write=%0d",local_v.s,m.data); end endmodule
"####,
        &["T|package_calls|5 5", "T|package_calls|copy=5 write=17"],
    );
}

#[test]
fn vif_two_packages() {
    check(
        r####"interface bus_if(input logic [7:0] s);
 logic [7:0] data;
 function automatic int read_value(); return s; endfunction
 task automatic put(input int n); data=n; endtask
 modport mp(input s, output data, import read_value, put);
endinterface
package p; virtual bus_if v; task show(); $display("T|two_packages|p=%0d",v.s); endtask endpackage package q; virtual bus_if v; task show(); $display("T|two_packages|q=%0d",v.s); endtask endpackage
module top; logic [7:0] a=5, b=9; bus_if m(a); bus_if n(b);

initial begin p::v=m; q::v=n; #1 p::show(); q::show(); $display("T|two_packages|direct=%0d %0d",p::v.s,q::v.s); end endmodule
"####,
        &[
            "T|two_packages|p=5",
            "T|two_packages|q=9",
            "T|two_packages|direct=5 9",
        ],
    );
}

#[test]
fn vif_module_shadow() {
    check(
        r####"interface bus_if(input logic [7:0] s);
 logic [7:0] data;
 function automatic int read_value(); return s; endfunction
 task automatic put(input int n); data=n; endtask
 modport mp(input s, output data, import read_value, put);
endinterface
package p; virtual bus_if v; task show(); $display("T|module_shadow|p=%0d",v.s); endtask endpackage
module top; logic [7:0] a=5, b=9; bus_if m(a); bus_if n(b); import p::*;
virtual bus_if v;
initial begin p::v=m; v=n; #1 p::show(); $display("T|module_shadow|module=%0d package=%0d",v.s,p::v.s); end endmodule
"####,
        &["T|module_shadow|p=5", "T|module_shadow|module=9 package=5"],
    );
}

#[test]
fn vif_block_shadow() {
    check(
        r####"interface bus_if(input logic [7:0] s);
 logic [7:0] data;
 function automatic int read_value(); return s; endfunction
 task automatic put(input int n); data=n; endtask
 modport mp(input s, output data, import read_value, put);
endinterface
package p; virtual bus_if v; endpackage
module top; logic [7:0] a=5, b=9; bus_if m(a); bus_if n(b); import p::*;

initial begin virtual bus_if v; p::v=m; v=n; #1 $display("T|block_shadow|local=%0d package=%0d",v.s,p::v.s); end endmodule
"####,
        &["T|block_shadow|local=9 package=5"],
    );
}

#[test]
fn vif_fixed_array() {
    check(
        r####"interface bus_if(input logic [7:0] s);
 logic [7:0] data;
 function automatic int read_value(); return s; endfunction
 task automatic put(input int n); data=n; endtask
 modport mp(input s, output data, import read_value, put);
endinterface
package p; virtual bus_if v[2]; task show(); $display("T|fixed_array|%0d %0d",v[0].s,v[1].s); endtask endpackage
module top; logic [7:0] a=5, b=9; bus_if m(a); bus_if n(b); import p::*;

initial begin p::v[0]=m; p::v[1]=n; #1 show(); $display("T|fixed_array|qualified=%0d",p::v[1].s); end endmodule
"####,
        &["T|fixed_array|5 9", "T|fixed_array|qualified=9"],
    );
}

#[test]
fn vif_queue() {
    check(
        r####"interface bus_if(input logic [7:0] s);
 logic [7:0] data;
 function automatic int read_value(); return s; endfunction
 task automatic put(input int n); data=n; endtask
 modport mp(input s, output data, import read_value, put);
endinterface
package p; virtual bus_if v[$]; task show(); $display("T|queue|%0d %0d",v[0].s,v[1].s); endtask endpackage
module top; logic [7:0] a=5, b=9; bus_if m(a); bus_if n(b); import p::*;
virtual bus_if local_v;
initial begin p::v.push_back(m); p::v.push_back(n); #1 show(); local_v=p::v[1]; $display("T|queue|copy=%0d",local_v.s); end endmodule
"####,
        &["T|queue|5 9", "T|queue|copy=9"],
    );
}

#[test]
fn vif_assoc_string() {
    check(
        r####"interface bus_if(input logic [7:0] s);
 logic [7:0] data;
 function automatic int read_value(); return s; endfunction
 task automatic put(input int n); data=n; endtask
 modport mp(input s, output data, import read_value, put);
endinterface
package p; virtual bus_if v[string]; task show(); $display("T|assoc_string|%0d",v["chosen"].s); endtask endpackage
module top; logic [7:0] a=5, b=9; bus_if m(a); bus_if n(b); import p::*;
virtual bus_if local_v;
initial begin p::v["chosen"]=n; #1 show(); local_v=p::v["chosen"]; $display("T|assoc_string|copy=%0d",local_v.s); end endmodule
"####,
        &["T|assoc_string|9", "T|assoc_string|copy=9"],
    );
}

#[test]
fn vif_dynamic_array() {
    check(
        r####"interface bus_if(input logic [7:0] s);
 logic [7:0] data;
 function automatic int read_value(); return s; endfunction
 task automatic put(input int n); data=n; endtask
 modport mp(input s, output data, import read_value, put);
endinterface
package p; virtual bus_if v[]; task show(); $display("T|dynamic_array|%0d %0d",v[0].s,v[1].s); endtask endpackage
module top; logic [7:0] a=5, b=9; bus_if m(a); bus_if n(b); import p::*;

initial begin p::v=new[2]; p::v[0]=m; p::v[1]=n; #1 show(); end endmodule
"####,
        &["T|dynamic_array|5 9"],
    );
}

#[test]
fn vif_task_rebind() {
    check(
        r####"interface bus_if(input logic [7:0] s);
 logic [7:0] data;
 function automatic int read_value(); return s; endfunction
 task automatic put(input int n); data=n; endtask
 modport mp(input s, output data, import read_value, put);
endinterface
package p; virtual bus_if v; task automatic set_v(input virtual bus_if x); v=x; endtask task show(); $display("T|task_rebind|%0d",v.s); endtask endpackage
module top; logic [7:0] a=5, b=9; bus_if m(a); bus_if n(b); import p::*;

initial begin p::v=m; #1 show(); set_v(n); show(); p::v=null; $display("T|task_rebind|null=%0d",p::v==null); end endmodule
"####,
        &["T|task_rebind|5", "T|task_rebind|9", "T|task_rebind|null=1"],
    );
}

#[test]
fn vif_parameterized() {
    check(
        r####"interface bus_if #(parameter W=8)(input logic [W-1:0] s);
 logic [7:0] data;
 function automatic int read_value(); return s; endfunction
 task automatic put(input int n); data=n; endtask
 modport mp(input s, output data, import read_value, put);
endinterface
package p; virtual bus_if #(8) v; task show(); $display("T|parameterized|%0d",v.s); endtask endpackage
module top; logic [7:0] a=5, b=9; bus_if m(a); bus_if n(b); import p::*;

initial begin p::v=m; #1 show(); end endmodule
"####,
        &["T|parameterized|5"],
    );
}

#[test]
fn vif_user_reproducer() {
    check(
        r####"// Minimal repro: package-scope virtual interface declaration.
interface mif (input logic [3:0] s);
endinterface
package p;
    virtual mif v;
    task show();
        $display("pkg vif s=%0d", v.s);
    endtask
endpackage
module top;
    logic [3:0] a = 5;
    mif m (.s(a));
    import p::*;
    initial begin
        p::v = m;
        #1 show();
        $finish;
    end
endmodule
"####,
        &["pkg vif s=5"],
    );
}
