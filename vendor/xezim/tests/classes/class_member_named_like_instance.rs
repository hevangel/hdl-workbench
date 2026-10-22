//! #271: a class member read through a handle (`t.irq`), through `this`, or
//! bare inside a method must never resolve to a same-named hierarchical
//! instance (§8.5, §23.6, §23.9). An interface instance `irq` turned the
//! scalar compare `t.irq != |(t.pending & t.enable)` into a virtual-interface
//! binding compare. Expected values are the reference simulator's.

use xezim::simulate;

fn t_lines(src: &str) -> Vec<String> {
    let sim = simulate(src, 100).expect("simulate failed");
    sim.output
        .iter()
        .filter_map(|l| l.message.strip_prefix("T|").map(str::to_string))
        .collect()
}

#[test]
fn member_compare_ignores_same_named_interface_instance() {
    let src = r#"
interface flag_if; endinterface
module top;
 flag_if irq();
 class item;
  bit irq;
  bit [1:0] pending, enable;
 endclass
 class irq_probe;
  function void check(item t);
   $display("IRQ %b PENDING %b ENABLE %b EXPECTED %b MISMATCH %b", t.irq,t.pending,t.enable,|(t.pending&t.enable),t.irq != |(t.pending&t.enable));
   if(t.irq != |(t.pending & t.enable)) $fatal(1,"IRQ false mismatch");
  endfunction
 endclass
 item t;
 irq_probe c;
 initial begin
  t=new; c=new;
  c.check(t);
  t.pending=2'b11;t.enable=2'b01;t.irq=1;
  c.check(t);
  $display("IRQ_CHECK_PASS"); $finish;
 end
endmodule
"#;
    let sim = simulate(src, 100).expect("simulate failed");
    let msgs: Vec<&str> = sim.output.iter().map(|l| l.message.as_str()).collect();
    assert!(
        msgs.contains(&"IRQ 0 PENDING 00 ENABLE 00 EXPECTED 0 MISMATCH 0"),
        "{msgs:?}"
    );
    assert!(
        msgs.contains(&"IRQ 1 PENDING 11 ENABLE 01 EXPECTED 1 MISMATCH 0"),
        "{msgs:?}"
    );
    assert!(msgs.contains(&"IRQ_CHECK_PASS"), "{msgs:?}");
}

#[test]
fn members_named_like_instances_nets_and_tasks() {
    // Interface instance, module instance, net and task names shared with
    // class members; reads, writes, `this.` and bare names in methods.
    let got = t_lines(include_str!("member_named_like_instance.sv"));
    let want = [
        "IRQ 0 PENDING 00 ENABLE 00 EXPECTED 0 MISMATCH 0",
        "c1 0 bare=0 this=0",
        "IRQ 1 PENDING 11 ENABLE 01 EXPECTED 1 MISMATCH 0",
        "c2 0 bare=0 this=0",
        "IRQ 0 PENDING 11 ENABLE 01 EXPECTED 1 MISMATCH 1",
        "c3 1 bare=1 this=1",
        "eq t.irq==u.irq 1  t.irq!=u.irq 0  t.irq===u.irq 1",
        "rd sub=3 w=01 go=9 cnt=2",
        "cmp sub==3 1 sub!=3 0 w==1 1 w!=1 0 go==9 1 cnt==2 1 cnt!=5 1",
        "cmp_hier sub.v==A 1 w==2 1 cnt==5 1 irq.sig 1",
        "cmp_mixed 0 1",
        "show irq=1 sub=3 w=01 go=7 eq_irq=1 eq_sub=1 eq_w=1 eq_go=1",
        "show_this irq=1 sub=3 w=01 go=7 ne_irq=0 ne_sub=0 ne_w=0 ne_go=0",
        "u irq=1 sub=3 w=01 go=7",
        "u_vs_t irq 1 sub 1 w 1 go 1",
        "module task go called",
        "still w=10 cnt=5 sub.v=a",
        "IRQ 0 PENDING 11 ENABLE 01 EXPECTED 1 MISMATCH 1",
        "branch mismatch",
        "done",
    ];
    assert_eq!(got, want);
}

#[test]
fn vif_compares_keep_binding_semantics_beside_same_named_members() {
    // A vif property named like the instance still compares by binding; a
    // plain local copied from a same-named member compares by value.
    let got = t_lines(include_str!("member_named_like_vif_instance.sv"));
    let want = [
        "vif_null 1 1",
        "vif_bound 0 1 0 1",
        "vif_same 1 0",
        "local_copy0 0",
        "local_copy1 1 bare_vs 1",
        "copy_bare0 1",
        "copy_bare1 0",
        "mixed 0 1",
        "mixed2 1 0",
        "hier 1 1",
    ];
    assert_eq!(got, want);
}
