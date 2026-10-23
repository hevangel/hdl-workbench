//! §13.3 — a class-typed FORMAL keeps its declared type across a timing
//! control, whatever other processes are doing.
//!
//! `var_class_types` is keyed by BARE variable name and shared by every
//! process; `local_type_stack` is the per-frame overlay that exists to stop
//! one scope's name from deciding another's type. Three paths wrote only the
//! global: `bind_task_frame` (via `register_formal_type_metadata`),
//! `exec_method_in_class_hierarchy` (which recorded during the port loop, i.e.
//! into the CALLER's frame, where `local_class_type_of` never looks), and
//! `class_declared_type_of` (what `$typename` reads).
//!
//! So while another process sat suspended in a task declaring `other item;`,
//! a resumed `task get(base src, output pkt item)` saw its formal as `other`
//! (issue #239): `$cast` into it failed for a compatible object — and, worse,
//! SUCCEEDED for an incompatible one.

use xezim::simulate;

fn lines(src: &str, tag: &str) -> Vec<String> {
    let sim = simulate(src, 1000).expect("sim");
    sim.output
        .iter()
        .filter(|o| o.message.starts_with(tag))
        .map(|o| o.message.clone())
        .collect()
}

/// `dir` is the formal's direction syntax (`output pkt` / `ref pkt`).
fn reported_case(dir: &str, fork_other: bool) -> Vec<String> {
    let fork_line = if fork_other {
        "    fork u.run(); join_none\n"
    } else {
        ""
    };
    let src = format!(
        "class base; endclass\n\
class pkt extends base; endclass\n\
class other; endclass\n\
class consumer;\n\
  task get(base src, {dir} item);\n\
    #1;\n\
    $display(\"T %0s\", $typename(item));\n\
    if (!$cast(item, src)) $display(\"T cast-failed\");\n\
  endtask\n\
endclass\n\
class other_user;\n\
  task run(); other item; #100; endtask\n\
endclass\n\
module top;\n\
  initial begin\n\
    consumer c = new();\n\
    other_user u = new();\n\
    pkt p = new(), r;\n\
{fork_line}\
    c.get(p, r);\n\
    $display(\"T r-%0s\", r == null ? \"null\" : \"nonnull\");\n\
    $finish;\n\
  end\n\
endmodule"
    );
    lines(&src, "T ")
}

#[test]
fn output_formal_keeps_its_class_across_a_delay() {
    let out = reported_case("output pkt", true);
    assert_eq!(
        out,
        vec!["T class pkt", "T r-nonnull"],
        "a suspended process's same-named local must not retype the formal"
    );
}

#[test]
fn ref_formal_keeps_its_class_across_a_delay() {
    let out = reported_case("ref pkt", true);
    assert_eq!(out, vec!["T class pkt", "T r-nonnull"]);
}

/// Control: without the competing process the case always worked, because the
/// shared global still held the formal's own entry. It must keep working.
#[test]
fn formal_class_is_correct_without_a_competing_process() {
    let out = reported_case("output pkt", false);
    assert_eq!(out, vec!["T class pkt", "T r-nonnull"]);
}

/// The dangerous half: a cast of an INCOMPATIBLE object into the formal must
/// still fail. While the formal read `other`, casting an `other` into a `pkt`
/// formal succeeded silently — corrupting the handle instead of reporting.
#[test]
fn incompatible_cast_into_the_formal_still_fails_after_a_delay() {
    let src = "\
class base; endclass\n\
class pkt extends base; endclass\n\
class other; endclass\n\
class consumer;\n\
  task get(base src, output pkt item);\n\
    other bad = new();\n\
    if (!$cast(item, bad)) $display(\"T pre-failed\");\n\
    #1;\n\
    if (!$cast(item, bad)) $display(\"T post-failed\");\n\
  endtask\n\
endclass\n\
class other_user;\n\
  task run(); other item; #100; endtask\n\
endclass\n\
module top;\n\
  initial begin\n\
    consumer c = new();\n\
    other_user u = new();\n\
    pkt p = new(), r;\n\
    fork u.run(); join_none\n\
    c.get(p, r);\n\
    $finish;\n\
  end\n\
endmodule";
    let out = lines(src, "T ");
    assert_eq!(
        out,
        vec!["T pre-failed", "T post-failed"],
        "an `other` must never cast into a `pkt` formal, before or after the delay"
    );
}

/// A LOCAL of the resuming task was already frame-scoped; keep it that way so
/// the formal fix did not come at the local's expense.
#[test]
fn local_of_the_resuming_task_is_unaffected() {
    let src = "\
class pkt; endclass\n\
class other; endclass\n\
class consumer;\n\
  task get();\n\
    pkt item = new();\n\
    #1;\n\
    $display(\"T %0s\", $typename(item));\n\
  endtask\n\
endclass\n\
class other_user;\n\
  task run(); other item; #100; endtask\n\
endclass\n\
module top;\n\
  initial begin\n\
    consumer c = new();\n\
    other_user u = new();\n\
    fork u.run(); join_none\n\
    c.get();\n\
    $finish;\n\
  end\n\
endmodule";
    assert_eq!(lines(src, "T "), vec!["T class pkt"]);
}
