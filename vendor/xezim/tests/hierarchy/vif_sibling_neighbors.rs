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
fn copy_queue_null() {
    check(
        r####"interface bus_if(input logic[7:0] s); endinterface
module top; logic[7:0] a=5,b=9; bus_if m(a); bus_if n(b);
virtual bus_if x[$],y[$];
initial begin #1; x.push_back(m);x.push_back(null);y.push_back(n);y=x;$display("T|%0d %0d %0d",y[0].s,y[1]==null,y.size()); $finish; end endmodule
"####,
        &["T|5 1 2"],
    );
}

#[test]
fn copy_queue_self() {
    check(
        r####"interface bus_if(input logic[7:0] s); endinterface
module top; logic[7:0] a=5,b=9; bus_if m(a); bus_if n(b);
virtual bus_if x[$];
initial begin #1; x.push_back(m);x.push_back(n);x=x;$display("T|%0d %0d",x[0].s,x[1].s); $finish; end endmodule
"####,
        &["T|5 9"],
    );
}

#[test]
fn copy_queue_empty() {
    check(
        r####"interface bus_if(input logic[7:0] s); endinterface
module top; logic[7:0] a=5,b=9; bus_if m(a); bus_if n(b);
virtual bus_if x[$],y[$];
initial begin #1; y.push_back(m);y=x;$display("T|%0d",y.size()); $finish; end endmodule
"####,
        &["T|0"],
    );
}

#[test]
fn copy_dynamic() {
    check(
        r####"interface bus_if(input logic[7:0] s); endinterface
module top; logic[7:0] a=5,b=9; bus_if m(a); bus_if n(b);
virtual bus_if x[],y[];
initial begin #1; x=new[2];x[0]=n;y=x;x[0]=m;$display("T|%0d %0d %0d",y[0].s,y[1]==null,y.size()); $finish; end endmodule
"####,
        &["T|9 1 2"],
    );
}

#[test]
fn copy_assoc() {
    check(
        r####"interface bus_if(input logic[7:0] s); endinterface
module top; logic[7:0] a=5,b=9; bus_if m(a); bus_if n(b);
virtual bus_if x[string],y[string];
initial begin #1; x["one"]=m;x["two"]=n;y["old"]=n;y=x;x.delete();$display("T|%0d %0d %0d %0d",y["one"].s,y["two"].s,y.num(),y.exists("old")); $finish; end endmodule
"####,
        &["T|5 9 2 0"],
    );
}

#[test]
fn copy_fixed_descending() {
    check(
        r####"interface bus_if(input logic[7:0] s); endinterface
module top; logic[7:0] a=5,b=9; bus_if m(a); bus_if n(b);
virtual bus_if x[3:2],y[1:0];
initial begin #1; x[3]=m;x[2]=n;y=x;$display("T|%0d %0d",y[1].s,y[0].s); $finish; end endmodule
"####,
        &["T|5 9"],
    );
}

#[test]
fn mutate_queue_front() {
    check(
        r####"interface bus_if(input logic[7:0] s); endinterface
module top; logic[7:0] a=5,b=9; bus_if m(a); bus_if n(b);
virtual bus_if x[$];
initial begin #1; x.push_back(m);x[0]=m;x.push_front(n);$display("T|%0d %0d",x[0].s,x[1].s); $finish; end endmodule
"####,
        &["T|9 5"],
    );
}

#[test]
fn mutate_queue_insert() {
    check(
        r####"interface bus_if(input logic[7:0] s); endinterface
module top; logic[7:0] a=5,b=9; bus_if m(a); bus_if n(b);
virtual bus_if x[$];
initial begin #1; x.push_back(m);x[0]=m;x.insert(0,n);$display("T|%0d %0d",x[0].s,x[1].s); $finish; end endmodule
"####,
        &["T|9 5"],
    );
}

#[test]
fn mutate_queue_reverse() {
    check(
        r####"interface bus_if(input logic[7:0] s); endinterface
module top; logic[7:0] a=5,b=9; bus_if m(a); bus_if n(b);
virtual bus_if x[$];
initial begin #1; x.push_back(m);x.push_back(n);x[0]=m;x.reverse();$display("T|%0d %0d",x[0].s,x[1].s); $finish; end endmodule
"####,
        &["T|9 5"],
    );
}

#[test]
fn mutate_assoc_missing() {
    check(
        r####"interface bus_if(input logic[7:0] s); endinterface
module top; logic[7:0] a=5,b=9; bus_if m(a); bus_if n(b);
virtual bus_if x[string];
initial begin #1; x["k"]=n;x.delete("k");$display("T|%0d %0d %0d",x["k"]==null,x["absent"]==null,x.num()); $finish; end endmodule
"####,
        &["T|1 1 0"],
    );
}

#[test]
fn mutate_class_queue() {
    check(
        r####"interface bus_if(input logic[7:0] s); endinterface
module top; logic[7:0] a=5,b=9; bus_if m(a); bus_if n(b);
class holder; virtual bus_if x[$]; endclass holder h;
initial begin #1; h=new;h.x.push_back(m);h.x.push_back(n);h.x[0]=m;h.x.delete(0);$display("T|%0d",h.x[0].s); $finish; end endmodule
"####,
        &["T|9"],
    );
}

#[test]
fn mutate_class_assoc() {
    check(
        r####"interface bus_if(input logic[7:0] s); endinterface
module top; logic[7:0] a=5,b=9; bus_if m(a); bus_if n(b);
class holder; virtual bus_if x[string]; endclass holder h;
initial begin #1; h=new;h.x["k"]=n;h.x.delete("k");h.x["k"]=m;$display("T|%0d %0d",h.x["k"].s,h.x.num()); $finish; end endmodule
"####,
        &["T|5 1"],
    );
}

#[test]
fn mutate_class_resize() {
    check(
        r####"interface bus_if(input logic[7:0] s); endinterface
module top; logic[7:0] a=5,b=9; bus_if m(a); bus_if n(b);
class holder; virtual bus_if x[]; endclass holder h;
initial begin #1; h=new;h.x=new[1];h.x[0]=n;h.x=new[1];$display("T|%0d",h.x[0]==null); $finish; end endmodule
"####,
        &["T|1"],
    );
}

#[test]
fn task_inout() {
    check(
        r####"interface bus_if(input logic[7:0] s); endinterface
module top; logic[7:0] a=5,b=9; bus_if m(a); bus_if n(b);
virtual bus_if v; task automatic change(inout virtual bus_if x);x=n;endtask
initial begin #1; v=m;change(v);$display("T|%0d",v.s); $finish; end endmodule
"####,
        &["T|9"],
    );
}

#[test]
fn task_null_ref() {
    check(
        r####"interface bus_if(input logic[7:0] s); endinterface
module top; logic[7:0] a=5,b=9; bus_if m(a); bus_if n(b);
virtual bus_if v;task automatic clear_v(ref virtual bus_if x);x=null;endtask
initial begin #1; v=m;clear_v(v);$display("T|%0d",v==null); $finish; end endmodule
"####,
        &["T|1"],
    );
}

#[test]
fn task_null_output() {
    check(
        r####"interface bus_if(input logic[7:0] s); endinterface
module top; logic[7:0] a=5,b=9; bus_if m(a); bus_if n(b);
virtual bus_if v;task automatic clear_v(output virtual bus_if x);x=null;endtask
initial begin #1; v=m;clear_v(v);$display("T|%0d",v==null); $finish; end endmodule
"####,
        &["T|1"],
    );
}

#[test]
fn task_input_isolation() {
    check(
        r####"interface bus_if(input logic[7:0] s); endinterface
module top; logic[7:0] a=5,b=9; bus_if m(a); bus_if n(b);
virtual bus_if x;task automatic change(input virtual bus_if x);x=n;endtask
initial begin #1; x=m;change(x);$display("T|%0d",x.s); $finish; end endmodule
"####,
        &["T|5"],
    );
}

#[test]
fn task_nested_shadow() {
    check(
        r####"interface bus_if(input logic[7:0] s); endinterface
module top; logic[7:0] a=5,b=9; bus_if m(a); bus_if n(b);
virtual bus_if v;task automatic inner(ref virtual bus_if x);x=n;endtask task automatic outer(ref virtual bus_if x);inner(x);endtask
initial begin #1; v=m;outer(v);$display("T|%0d",v.s); $finish; end endmodule
"####,
        &["T|9"],
    );
}

#[test]
fn task_class_output() {
    check(
        r####"interface bus_if(input logic[7:0] s); endinterface
module top; logic[7:0] a=5,b=9; bus_if m(a); bus_if n(b);
class holder;virtual bus_if v;endclass holder h;task automatic change(output virtual bus_if x);x=n;endtask
initial begin #1; h=new;h.v=m;change(h.v);$display("T|%0d",h.v.s); $finish; end endmodule
"####,
        &["T|9"],
    );
}

#[test]
fn task_parked_ref() {
    check(
        r####"interface bus_if(input logic[7:0] s); endinterface
module top; logic[7:0] a=5,b=9; bus_if m(a); bus_if n(b);
virtual bus_if v;task automatic change(ref virtual bus_if x);x=n;#2;x=null;endtask
initial begin #1; v=m;fork change(v);begin #1;$display("T|mid=%0d",v.s);end join $display("T|end=%0d",v==null); $finish; end endmodule
"####,
        &["T|mid=9", "T|end=1"],
    );
}

#[test]
fn task_same_name_ref() {
    check(
        r####"interface bus_if(input logic[7:0] s); endinterface
module top; logic[7:0] a=5,b=9; bus_if m(a); bus_if n(b);
virtual bus_if x;task automatic change(ref virtual bus_if x);x=n;endtask
initial begin #1; x=m;change(x);$display("T|%0d",x.s); $finish; end endmodule
"####,
        &["T|9"],
    );
}

#[test]
fn assoc_copy_self() {
    check(
        r####"interface bus_if(input int s);endinterface module top;bus_if m(5),n(9);virtual bus_if x[string]; initial begin #1;x["k"]=m;x=x;$display("T|%0d %0d",x["k"].s,x.num()); $finish;end endmodule"####,
        &["T|5 1"],
    );
}

#[test]
fn assoc_copy_empty() {
    check(
        r####"interface bus_if(input int s);endinterface module top;bus_if m(5),n(9);virtual bus_if x[string],y[string]; initial begin #1;y["old"]=m;y=x;$display("T|%0d %0d",y.num(),y["old"]==null); $finish;end endmodule"####,
        &["T|0 1"],
    );
}

#[test]
fn assoc_copy_numeric() {
    check(
        r####"interface bus_if(input int s);endinterface module top;bus_if m(5),n(9);virtual bus_if x[int],y[int]; initial begin #1;x[-2]=n;y[3]=m;y=x;$display("T|%0d %0d %0d",y[-2].s,y.num(),y.exists(3)); $finish;end endmodule"####,
        &["T|9 1 0"],
    );
}

#[test]
fn assoc_copy_class() {
    check(
        r####"interface bus_if(input int s);endinterface module top;bus_if m(5),n(9);class holder;virtual bus_if x[string];endclass holder a,b; initial begin #1;a=new;b=new;a.x["k"]=n;b.x["old"]=m;b.x=a.x;$display("T|%0d %0d %0d",b.x["k"].s,b.x.num(),b.x.exists("old")); $finish;end endmodule"####,
        &["T|9 1 0"],
    );
}

#[test]
fn assoc_copy_mixed() {
    check(
        r####"interface bus_if(input int s);endinterface module top;bus_if m(5),n(9);class holder;virtual bus_if x[string];endclass holder a;virtual bus_if y[string]; initial begin #1;a=new;a.x["k"]=n;y["old"]=m;y=a.x;$display("T|%0d %0d %0d",y["k"].s,y.num(),y.exists("old")); $finish;end endmodule"####,
        &["T|9 1 0"],
    );
}

#[test]
fn task_output_default() {
    check(
        r####"interface bus_if(input int s);endinterface module top;bus_if m(5),n(9);virtual bus_if v;task automatic clear_v(output virtual bus_if x);endtask initial begin #1;v=m;clear_v(v);$display("T|%0d",v==null); $finish;end endmodule"####,
        &["T|1"],
    );
}

#[test]
fn task_output_seed() {
    check(
        r####"interface bus_if(input int s);endinterface module top;bus_if m(5),n(9);virtual bus_if v;task automatic clear_v(output virtual bus_if x);$display("T|seed=%0d",x==null);x=n;endtask initial begin #1;v=m;clear_v(v);$display("T|end=%0d",v.s); $finish;end endmodule"####,
        &["T|seed=1", "T|end=9"],
    );
}

#[test]
fn task_ref_external() {
    check(
        r####"interface bus_if(input int s);endinterface module top;bus_if m(5),n(9);virtual bus_if v;task automatic observe(ref virtual bus_if x);#2;$display("T|inside=%0d",x.s);endtask initial begin #1;v=m;fork observe(v);begin #1;v=n;end join $display("T|after=%0d",v.s);$finish;end endmodule
"####,
        &["T|inside=9", "T|after=9"],
    );
}

#[test]
fn element_index_once() {
    check(
        r####"interface bus_if(input int s);endinterface module top;bus_if m(5),n(9);virtual bus_if x[2];int i; initial begin #1;x[i++]=m;$display("T|%0d %0d %0d",i,x[0].s,x[1]==null); $finish;end endmodule"####,
        &["T|1 5 1"],
    );
}

#[test]
fn element_string_index_once() {
    check(
        r####"interface bus_if(input int s);endinterface module top;bus_if m(5),n(9);virtual bus_if x[string];int calls;function string key();calls++;return "k";endfunction initial begin #1;x[key()]=m;$display("T|%0d %0d %0d",calls,x["k"].s,x.num()); $finish;end endmodule"####,
        &["T|1 5 1"],
    );
}

#[test]
fn element_class_index_once() {
    check(
        r####"interface bus_if(input int s);endinterface module top;bus_if m(5),n(9);class holder;virtual bus_if x[2];endclass holder h;int i; initial begin #1;h=new;h.x[i++]=m;$display("T|%0d %0d %0d",i,h.x[0].s,h.x[1]==null); $finish;end endmodule"####,
        &["T|1 5 1"],
    );
}

#[test]
fn element_class_out_of_range() {
    check(
        r####"interface bus_if(input int s);endinterface module top;bus_if m(5),n(9);class holder;virtual bus_if x[2];endclass holder h;virtual bus_if v; initial begin #1;h=new;h.x[2]=m;v=h.x[2];$display("T|%0d %0d",v==null,h.x[2]==null); $finish;end endmodule"####,
        &["T|1 1"],
    );
}

#[test]
fn element_queue_spare() {
    check(
        r####"interface bus_if(input int s);endinterface module top;bus_if m(5),n(9);virtual bus_if x[$],v; initial begin #1;x.push_back(m);x.push_back(n);x.pop_back();v=x[1];$display("T|%0d %0d",v==null,x.size()); $finish;end endmodule"####,
        &["T|1 1"],
    );
}

#[test]
fn element_dynamic_spare() {
    check(
        r####"interface bus_if(input int s);endinterface module top;bus_if m(5),n(9);virtual bus_if x[],v; initial begin #1;x=new[2];x[1]=n;x=new[1](x);v=x[1];$display("T|%0d %0d",v==null,x.size()); $finish;end endmodule"####,
        &["T|1 1"],
    );
}

#[test]
fn element_class_queue_append() {
    check(
        r####"interface bus_if(input int s);endinterface module top;bus_if m(5),n(9);class holder;virtual bus_if x[$];endclass holder h;virtual bus_if v; initial begin #1;h=new;h.x[0]=m;v=h.x[0];$display("T|%0d %0d",v==null,h.x.size()); $finish;end endmodule"####,
        &["T|0 1"],
    );
}

/// §8.9/§25.10: collection identity follows the declaring storage.
#[test]
fn static_fixed_object_write() {
    check(
        r####"interface bus_if(input int s);endinterface class holder;static virtual bus_if a[2];endclass module top;bus_if m(5),n(9);holder h;initial begin #1;h=new;h.a[0]=n;$display("T|%0d %0d",holder::a[0].s,h.a[0].s); $finish;end endmodule"####,
        &["T|9 9"],
    );
}

/// §8.9/§25.10: collection identity follows the declaring storage.
#[test]
fn static_queue_object_append() {
    check(
        r####"interface bus_if(input int s);endinterface class holder;static virtual bus_if a[$];endclass module top;bus_if m(5),n(9);holder h;initial begin #1;h=new;h.a[0]=n;$display("T|%0d %0d %0d",holder::a[0].s,h.a[0].s,holder::a.size()); $finish;end endmodule"####,
        &["T|9 9 1"],
    );
}

/// §8.9/§25.10: collection identity follows the declaring storage.
#[test]
fn static_inherited_object_write() {
    check(
        r####"interface bus_if(input int s);endinterface class base;static virtual bus_if a[2];endclass class holder extends base;endclass module top;bus_if m(5),n(9);holder h;initial begin #1;h=new;h.a[0]=n;$display("T|%0d %0d",base::a[0].s,holder::a[0].s); $finish;end endmodule"####,
        &["T|9 9"],
    );
}

/// §8.9/§25.10: collection identity follows the declaring storage.
#[test]
fn static_fixed_method_null() {
    check(
        r####"interface bus_if(input int s);endinterface class holder;static virtual bus_if a[2];static function void clear_v();a[0]=null;endfunction endclass module top;bus_if m(5),n(9);holder h;initial begin #1;h=new;h.a[0]=n;holder::clear_v();$display("T|%0d %0d",holder::a[0]==null,h.a[0]==null); $finish;end endmodule"####,
        &["T|1 1"],
    );
}

/// §8.9/§25.10: collection identity follows the declaring storage.
#[test]
fn static_queue_object_shift() {
    check(
        r####"interface bus_if(input int s);endinterface class holder;static virtual bus_if a[$];endclass module top;bus_if m(5),n(9);holder h;initial begin #1;h=new;h.a.push_back(m);h.a.push_back(n);h.a[0]=m;h.a.pop_front();$display("T|%0d %0d",holder::a[0].s,h.a[0].s); $finish;end endmodule"####,
        &["T|9 9"],
    );
}

/// §8.9/§25.10: collection identity follows the declaring storage.
#[test]
fn assoc_bracket_string_key() {
    check(
        r####"interface bus_if(input int s);endinterface class holder;virtual bus_if a[string];endclass module top;bus_if m(5),n(9);holder h;initial begin #1;h=new;h.a["k[x]"]=n;$display("T|%0d %0d",h.a["k[x]"].s,h.a.num()); $finish;end endmodule"####,
        &["T|9 1"],
    );
}
