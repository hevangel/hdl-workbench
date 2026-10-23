//! IEEE 1800 §25.9–25.10 and §23.11: reference-checked sibling regressions.
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
    .expect("simulate sibling regression");
    let mut lines: Vec<_> = sim
        .output
        .iter()
        .map(|o| o.message.trim_end())
        .filter(|s| s.starts_with("T|"))
        .collect();
    let mut expected = expected.to_vec();
    lines.sort();
    expected.sort();
    assert_eq!(lines, expected);
}

#[test]
fn vif_assoc_delete() {
    check(
        r####"interface bus_if(input int s); int data; endinterface
module top; bus_if m(5), n(9); virtual bus_if a[string]; initial begin #1; a["k"]=m; a.delete("k"); $display("T|null=%0d exists=%0d",a["k"]==null,a.exists("k")); $finish; end endmodule
"####,
        &["T|null=1 exists=0"],
    );
}

#[test]
fn vif_assoc_delete_all() {
    check(
        r####"interface bus_if(input int s); int data; endinterface
module top; bus_if m(5), n(9); virtual bus_if a[string]; initial begin #1; a["k"]=m; a["x"]=n; a.delete(); $display("T|null=%0d %0d size=%0d",a["k"]==null,a["x"]==null,a.num()); $finish; end endmodule
"####,
        &["T|null=1 1 size=0"],
    );
}

#[test]
fn vif_dynamic_resize() {
    check(
        r####"interface bus_if(input int s); int data; endinterface
module top; bus_if m(5), n(9); virtual bus_if a[]; initial begin #1; a=new[1]; a[0]=m; a=new[1]; $display("T|null=%0d size=%0d",a[0]==null,a.size()); $finish; end endmodule
"####,
        &["T|null=1 size=1"],
    );
}

#[test]
fn vif_dynamic_preserve() {
    check(
        r####"interface bus_if(input int s); int data; endinterface
module top; bus_if m(5), n(9); virtual bus_if a[]; initial begin #1; a=new[1]; a[0]=m; a=new[2](a); $display("T|old=%0d null=%0d",a[0].s,a[1]==null); $finish; end endmodule
"####,
        &["T|old=5 null=1"],
    );
}

#[test]
fn vif_queue_pop() {
    check(
        r####"interface bus_if(input int s); int data; endinterface
module top; bus_if m(5), n(9); virtual bus_if q[$]; virtual bus_if v; initial begin #1; q.push_back(m); q.push_back(n); q[0]=m; v=q.pop_front(); $display("T|popped=%0d next=%0d size=%0d",v.s,q[0].s,q.size()); $finish; end endmodule
"####,
        &["T|popped=5 next=9 size=1"],
    );
}

#[test]
fn vif_queue_delete() {
    check(
        r####"interface bus_if(input int s); int data; endinterface
module top; bus_if m(5), n(9); virtual bus_if q[$]; initial begin #1; q.push_back(m); q.push_back(n); q[0]=m; q.delete(0); $display("T|next=%0d size=%0d",q[0].s,q.size()); $finish; end endmodule
"####,
        &["T|next=9 size=1"],
    );
}

#[test]
fn vif_queue_delete_all() {
    check(
        r####"interface bus_if(input int s); int data; endinterface
module top; bus_if m(5), n(9); virtual bus_if q[$]; initial begin #1; q.push_back(m); q[0]=m; q.delete(); q.push_back(n); $display("T|next=%0d size=%0d",q[0].s,q.size()); $finish; end endmodule
"####,
        &["T|next=9 size=1"],
    );
}

#[test]
fn vif_assoc_null() {
    check(
        r####"interface bus_if(input int s); int data; endinterface
module top; bus_if m(5), n(9); virtual bus_if a[string]; initial begin #1; a["k"]=m; a["k"]=null; $display("T|null=%0d exists=%0d",a["k"]==null,a.exists("k")); $finish; end endmodule
"####,
        &["T|null=1 exists=1"],
    );
}

#[test]
fn vif_numeric_negative_key() {
    check(
        r####"interface bus_if(input int s); int data; endinterface
module top; bus_if m(5), n(9); virtual bus_if a[int]; virtual bus_if v; initial begin #1; a[-3]=n; v=a[-3]; $display("T|direct=%0d copy=%0d",a[-3].s,v.s); $finish; end endmodule
"####,
        &["T|direct=9 copy=9"],
    );
}

#[test]
fn vif_class_assoc_delete() {
    check(
        r####"interface bus_if(input int s); int data; endinterface
module top; bus_if m(5), n(9); class holder; virtual bus_if a[string]; endclass holder o; initial begin #1; o=new; o.a["k"]=m; o.a.delete("k"); $display("T|null=%0d exists=%0d",o.a["k"]==null,o.a.exists("k")); $finish; end endmodule
"####,
        &["T|null=1 exists=0"],
    );
}

#[test]
fn vif_class_dynamic_resize() {
    check(
        r####"interface bus_if(input int s); int data; endinterface
module top; bus_if m(5), n(9); class holder; virtual bus_if a[]; endclass holder o; initial begin #1; o=new; o.a=new[1]; o.a[0]=m; o.a=new[1]; $display("T|null=%0d",o.a[0]==null); $finish; end endmodule
"####,
        &["T|null=1"],
    );
}

#[test]
fn vif_fixed_array_copy() {
    check(
        r####"interface bus_if(input int s); int data; endinterface
module top; bus_if m(5), n(9); virtual bus_if a[2], b[2]; initial begin #1; a[0]=m; a[1]=n; b=a; $display("T|a=%0d b=%0d",b[0].s,b[1].s); $finish; end endmodule
"####,
        &["T|a=5 b=9"],
    );
}

#[test]
fn vif_queue_copy() {
    check(
        r####"interface bus_if(input int s); int data; endinterface
module top; bus_if m(5), n(9); virtual bus_if a[$], b[$]; initial begin #1; a.push_back(m); a.push_back(n); b=a; $display("T|a=%0d b=%0d",b[0].s,b[1].s); $finish; end endmodule
"####,
        &["T|a=5 b=9"],
    );
}

#[test]
fn vif_ref_argument() {
    check(
        r####"interface bus_if(input int s); int data; endinterface
module top; bus_if m(5), n(9); virtual bus_if v; task automatic swap_v(ref virtual bus_if x, input virtual bus_if y); x=y; endtask initial begin #1; v=m; swap_v(v,n); $display("T|v=%0d",v.s); $finish; end endmodule
"####,
        &["T|v=9"],
    );
}

#[test]
fn vif_output_argument() {
    check(
        r####"interface bus_if(input int s); int data; endinterface
module top; bus_if m(5), n(9); virtual bus_if v; task automatic get_v(output virtual bus_if x); x=n; endtask initial begin #1; v=m; get_v(v); $display("T|v=%0d",v.s); $finish; end endmodule
"####,
        &["T|v=9"],
    );
}

#[test]
fn vif_queue_null_control() {
    check(
        r####"interface bus_if(input int s); int data; endinterface
module top; bus_if m(5), n(9); virtual bus_if q[$]; virtual bus_if v; initial begin #1; q.push_back(null); v=m; v=q[0]; $display("T|null=%0d size=%0d",v==null,q.size()); $finish; end endmodule
"####,
        &["T|null=1 size=1"],
    );
}
