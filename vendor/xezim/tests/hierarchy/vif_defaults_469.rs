//! IEEE 1800 §13.5.3 / §25.9: omitted vif defaults retain declaration scope.
use xezim::simulate;

fn check(src: &str, expected: &[&str]) {
    let result = simulate(src, 100).expect("vif defaults must simulate");
    let lines: Vec<_> = result
        .output
        .iter()
        .map(|o| o.message.as_str())
        .filter(|s| s.starts_with("T|"))
        .collect();
    assert_eq!(lines, expected);
}

#[test]
fn default_uses_receiving_object() {
    check(
        r#"
interface bus_if; int sig; endinterface
class receiver;
  virtual bus_if v;
  function int read(input virtual bus_if x = v); return x.sig; endfunction
endclass
class caller;
  virtual bus_if v;
  receiver other;
  function int read(); return other.read(); endfunction
endclass
module top;
  bus_if a(), b();
  initial begin
    receiver r = new; caller c = new;
    a.sig = 11; b.sig = 42;
    r.v = b; c.v = a; c.other = r;
    $display("T|receiver %0d", c.read());
  end
endmodule
"#,
        &["T|receiver 42"],
    );
}

#[test]
fn getter_default_evaluated_once() {
    check(
        r#"
interface bus_if; int sig; endinterface
class receiver;
  virtual bus_if v;
  int calls;
  function virtual bus_if pick(); calls++; return v; endfunction
  function int read(input virtual bus_if x = this.pick()); return x.sig; endfunction
endclass
module top;
  bus_if b();
  initial begin
    receiver r = new;
    b.sig = 42; r.v = b;
    $display("T|getter %0d", r.read());
    $display("T|calls %0d", r.calls);
  end
endmodule
"#,
        &["T|getter 42", "T|calls 1"],
    );
}

#[test]
fn earlier_formal_default_and_explicit_null() {
    check(
        r#"
interface bus_if; int sig; endinterface
module top;
  bus_if a(), b();
  virtual bus_if v;
  function automatic int read(input virtual bus_if x = v, input virtual bus_if y = x);
    return y == null ? -1 : y.sig;
  endfunction
  initial begin
    a.sig = 11; b.sig = 42; v = b;
    $display("T|default %0d", read());
    $display("T|actual %0d", read(a));
    $display("T|null %0d", read(null));
  end
endmodule
"#,
        &["T|default 42", "T|actual 11", "T|null -1"],
    );
}

#[test]
fn named_and_sparse_defaults_use_the_receiver() {
    check(
        r#"
interface bus_if;int sig;endinterface
class receiver;
 virtual bus_if v;
 function int read(input int unused=0,input virtual bus_if x=v);return x.sig;endfunction
endclass
class caller;
 virtual bus_if v;receiver other;
 function void run();
 $display("T|named %0d",other.read(.unused(7)));
 $display("T|gap %0d",other.read(8,));
 $display("T|empty %0d",other.read(.x()));
 endfunction
endclass
module top;bus_if a(),b();initial begin
 receiver r=new;caller c=new;a.sig=11;b.sig=42;r.v=b;c.v=a;c.other=r;c.run();
 end endmodule
"#,
        &["T|named 42", "T|gap 42", "T|empty 42"],
    );
}
#[test]
fn blocking_task_defaults_and_copyback_use_the_receiver() {
    check(
        r#"
interface bus_if;int sig;endinterface
class receiver;
 virtual bus_if v;
 task read(input virtual bus_if x=v);#1;$display("T|task %0d",x.sig);endtask
 task keep(inout virtual bus_if x=v);#1;$display("T|inout %0d",x.sig);endtask
endclass
class caller;
 virtual bus_if v;receiver other;
 task run();other.read();other.keep();$display("T|caller %0d",v.sig);$display("T|receiver %0d",other.v.sig);endtask
endclass
module top;bus_if a(),b();initial begin
 receiver r=new;caller c=new;a.sig=11;b.sig=42;r.v=b;c.v=a;c.other=r;c.run();
 end endmodule
"#,
        &["T|task 42", "T|inout 42", "T|caller 11", "T|receiver 42"],
    );
}
#[test]
fn module_function_default_ignores_caller_property() {
    check(
        r#"
interface bus_if;int sig;endinterface
module top;
 bus_if a(),b();virtual bus_if v;
 function automatic int read(input virtual bus_if x=v);return x.sig;endfunction
 class caller;virtual bus_if v;function int run();return read();endfunction endclass
 initial begin caller c=new;a.sig=11;b.sig=42;v=b;c.v=a;$display("T|module %0d",c.run());end
endmodule
"#,
        &["T|module 42"],
    );
}

#[test]
fn default_output_may_target_an_earlier_formal() {
    check(
        r#"
interface bus_if;int sig;endinterface
module top;
 bus_if a(),b();virtual bus_if v,replacement;
 task automatic change(inout virtual bus_if x,inout virtual bus_if y=x);
 #1;y=replacement;
 endtask
 initial begin a.sig=11;b.sig=42;v=a;replacement=b;change(v);$display("T|earlier %0d",v.sig);end
endmodule
"#,
        &["T|earlier 42"],
    );
}
#[test]
fn function_default_copyback_ignores_caller_property() {
    check(
        r#"
interface bus_if;int sig;endinterface
module top;
 bus_if a(),b();virtual bus_if v;
 function automatic void keep(inout virtual bus_if x=v);endfunction
 class caller;
 virtual bus_if v;
 function void run();keep();$display("T|caller %0d",v.sig);endfunction
 endclass
 initial begin caller c=new;a.sig=11;b.sig=42;v=b;c.v=a;c.run();$display("T|module %0d",v.sig);end
endmodule
"#,
        &["T|caller 11", "T|module 42"],
    );
}

#[test]
fn implicit_class_vif_hides_module_vif_but_formal_still_wins() {
    check(
        r#"
interface bus_if;int sig;endinterface
module top;bus_if a(),b();virtual bus_if v;
 class c;
 virtual bus_if v;
 function int read();return v.sig;endfunction
 function int actual(input virtual bus_if v);return v.sig;endfunction
 endclass
 initial begin c h=new;a.sig=11;b.sig=42;v=b;h.v=a;
 $display("T|implicit %0d",h.read());
 $display("T|formal %0d",h.actual(v));
 $display("T|module %0d",v.sig);
 end endmodule
"#,
        &["T|implicit 11", "T|formal 42", "T|module 42"],
    );
}
