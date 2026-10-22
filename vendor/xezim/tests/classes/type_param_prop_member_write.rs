//! §8.25: a bare-name member write through an INHERITED property typed by a
//! class type parameter — UVM's `uvm_driver #(REQ)` declares `REQ req;` and a
//! driver does `req.data = vif.rdata;` — was silently dropped. The
//! property's declared type comes back as the parameter's name (`REQ`),
//! which is no class, so the write path never found the object; reads and
//! `this.req.data = ...` were unaffected. The parameter is now resolved
//! through the extends chain to the specialization's class.

use xezim::simulate;

fn out(src: &str) -> Vec<String> {
    let sim = simulate(src, 1000).expect("simulate failed");
    sim.output.iter().map(|o| o.message.clone()).collect()
}

fn expect(o: &[String], want: &[&str]) {
    for w in want {
        assert!(o.iter().any(|l| l == w), "expected {w:?} in:\n{o:#?}");
    }
}

#[test]
fn bare_write_through_type_param_property() {
    let o = out(r#"
class seq_item_base; int id; endclass
class item extends seq_item_base; bit [31:0] data; endclass
class drv_base #(type REQ = seq_item_base); REQ req; endclass
class drv extends drv_base #(item);
  function void f(bit [31:0] v);
    req.data = v;          $display("bare %h", req.data);
    req.id = 7;            $display("id %0d", req.id);
  endfunction
  task t(bit [31:0] v);
    #1 req.data = v;       $display("task %h", this.req.data);
  endtask
endclass
module top;
  initial begin
    drv d = new(); d.req = new();
    d.f(32'hE0);
    $display("outside %h", d.req.data);
    d.t(32'hB2);
  end
endmodule
"#);
    expect(
        &o,
        &["bare 000000e0", "id 7", "outside 000000e0", "task 000000b2"],
    );
}

/// A different process parked with a same-named local of an unrelated class
/// must not redirect the write.
#[test]
fn bare_write_through_type_param_property_with_parked_namesake() {
    let o = out(r#"
class item; bit [31:0] data; endclass
class other; int unused; endclass
class base #(type REQ = int); REQ req; endclass
class parker; task run(); other req; #100; endtask endclass
class drv extends base #(item);
  task t(bit [31:0] v); #1; req.data = v; $display("parked %h", req.data); endtask
endclass
module top;
  initial begin
    drv c = new(); parker p = new();
    c.req = new();
    fork p.run(); join_none
    c.t(32'hC3);
  end
endmodule
"#);
    expect(&o, &["parked 000000c3"]);
}
