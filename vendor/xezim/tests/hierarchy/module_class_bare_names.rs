//! §23.8: a class declared inside a module resolves BARE names — the
//! module's functions, tasks and variables — in the instance its method runs
//! for, before any same-named declaration of the top module. Calls went to
//! the top's namesake; with two instances, variables read the first
//! instance's (the per-AST-node name cache and the compiled-method plan both
//! bound the name once). Found while probing issue #320 (reported by
//! Ganesh-TNL). Expected values come from the reference simulator.

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
fn module_class_bare_names_resolve_per_instance() {
    let expected = [
        "T|b_1|3",
        "T|v_1|10",
        "T|w_1|11",
        "T|tk_1|5",
        "T|m_1|tb.m1.mid_m",
        "T|sb_1|3",
        "T|b_2|6",
        "T|v_2|20",
        "T|w_2|21",
        "T|tk_2|10",
        "T|m_2|tb.m2.mid_m",
        "T|sb_2|6",
    ];
    assert_eq!(
        t_lines(include_str!("module_class_bare_names.sv")),
        expected
    );
}

/// §6.21/§9.3.1: a variable declared in an `initial` block of a child
/// instance is that block's own, not a same-named block local of the top.
#[test]
fn child_initial_block_locals_are_distinct() {
    const SRC: &str = r#"
module mid;
  initial begin
    int c;
    c = 5;
    #1 $display("T|mid_noinit|%0d", c);
  end
  initial begin : nb
    int d = 6;
    #1 $display("T|mid_named|%0d", d);
  end
endmodule
module tb;
  mid m();
  initial begin
    int c;
    c = 7;
    #2 $display("T|tb_noinit|%0d", c);
  end
  initial begin : nb
    int d = 8;
    #2 $display("T|tb_named|%0d", d);
    $finish;
  end
endmodule
"#;
    let expected = [
        "T|mid_noinit|5",
        "T|mid_named|6",
        "T|tb_noinit|7",
        "T|tb_named|8",
    ];
    assert_eq!(t_lines(SRC), expected);
}

/// The same collision through class handles: the child's `h` read the
/// top's object.
#[test]
fn child_initial_block_class_handles_are_distinct() {
    const SRC: &str = r#"
class A; int v = 1; endclass
class B; int v = 2; endclass
module mid;
  initial begin
    int c = 5;
    A h = new;
    #1 $display("T|mid_int|%0d", c);
    $display("T|mid_h|%0d", h.v);
  end
endmodule
module tb;
  mid m();
  initial begin
    int c = 7;
    B h = new;
    #2 $display("T|tb_int|%0d", c);
    $display("T|tb_h|%0d", h.v);
    $finish;
  end
endmodule
"#;
    let expected = ["T|mid_int|5", "T|mid_h|1", "T|tb_int|7", "T|tb_h|2"];
    assert_eq!(t_lines(SRC), expected);
}
