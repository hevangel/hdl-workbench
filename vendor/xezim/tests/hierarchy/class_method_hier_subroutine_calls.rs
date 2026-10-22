//! §23.6/§23.8/§20.10: a module subroutine named by its full hierarchical
//! path from a CLASS METHOD (`tb.sub.u_h.get_inst_path()`) — functions,
//! tasks, interface functions, generate/array instances, `$root.` paths, from
//! static, parameterized and task methods. Instance subroutines are
//! registered without the top module's name, so the dotted call written in a
//! class body matched nothing and read 0 / an empty string; `%m` inside now
//! names the callee's scope. Issue #320 (reported by Ganesh-TNL). Expected
//! values come from the reference simulator.

use xezim::simulate;

fn t_lines(src: &str) -> Vec<String> {
    simulate(src, 1000)
        .expect("simulate failed")
        .output
        .iter()
        .map(|o| o.message.clone())
        .filter(|m| m.starts_with("T|"))
        .collect()
}

#[test]
fn hier_subroutine_calls_from_class_methods() {
    let expected = [
        "T|mod_ip|tb.m.u_h",
        "T|mod_m|tb.m.u_h.get_m",
        "T|mod_topfn|tb.top_fn",
        "T|cls_const_var|sub_env",
        "T|cls_const_inline|sub_env",
        "T|cls_ret|abc",
        "T|cls_int|42",
        "T|cls_int_inline|42",
        "T|cls_counter|5",
        "T|cls_ip|tb.m.u_h",
        "T|cls_m|tb.m.u_h.get_m",
        "T|cls_static|st",
        "T|cls_arg|q_x",
        "T|cls_task|task_out",
        "T|cls_var|tb.m.u_h",
        "T|cls_cvar|5",
        "T|cls_topfn|tb.top_fn",
        "T|cls_sget|tb.m.u_h",
        "T|cls_task_ip|tb.m.u_h",
    ];
    assert_eq!(
        t_lines(include_str!("class_method_hier_subroutine_calls.sv")),
        expected
    );
}

#[test]
fn hier_subroutine_call_forms_from_class_methods() {
    let expected = [
        "T|root|1",
        "T|gen1|11",
        "T|arr1|2",
        "T|iface|12",
        "T|refq|2 1 2",
        "T|outp|1 10",
        "T|show_m_1|tb.m.u_h.show",
        "T|voidcast|ok",
        "T|nest|101",
        "T|arith|8",
        "T|tbref|TOPS",
        "T|wr|9",
        "T|cond|7",
        "T|ifcall|yes",
        "T|argcall|11",
        "T|pkg|102",
        "T|param|6",
        "T|blk_task|3",
        "T|fork_task|6",
        "T|evt|6",
    ];
    assert_eq!(
        t_lines(include_str!("class_method_hier_subroutine_forms.sv")),
        expected
    );
}

/// Inside a constructor, nested in another call or `$sformatf`, in loops,
/// comparisons and method chains.
#[test]
fn hier_subroutine_calls_in_class_expressions() {
    let expected = [
        "T|ctor|8",
        "T|nested|<tb.m.u_h>",
        "T|nested2|12",
        "T|loop|22",
        "T|cmp|1",
        "T|len|8",
    ];
    assert_eq!(
        t_lines(include_str!("class_method_hier_subroutine_exprs.sv")),
        expected
    );
}
