//! A `for` statement that fails to compile to bytecode (its init, condition or
//! step bails) must not leave its break/continue frame behind: the enclosing
//! loop's `break`/`continue` would then be patched into the wrong frame and
//! stay `Jump(0)`, hanging the simulation. Each design self-checks and must
//! print `T|LEAK_PASS`, as on the reference simulator (IEEE 1800-2023 §12.8).

fn lines(src: &str) -> Vec<String> {
    let sim = xezim::simulate(src, 100).expect("simulation must finish");
    sim.output
        .iter()
        .map(|l| l.message.clone())
        .filter(|m| m.starts_with("T|"))
        .collect()
}

#[test]
fn inner_for_with_task_call_keeps_the_outer_while_frame() {
    assert_eq!(
        lines(include_str!("loop_bail_while_for_task.sv")),
        vec!["T|visits=4 edges=2 acc=4", "T|LEAK_PASS"]
    );
}

#[test]
fn inner_long_for_keeps_the_outer_while_frame() {
    assert_eq!(
        lines(include_str!("loop_bail_while_for_function.sv")),
        vec!["T|visits=4 edges=2 acc=19800", "T|LEAK_PASS"]
    );
}

#[test]
fn nested_loops_keep_their_frames() {
    let out = lines(include_str!("loop_bail_nested.sv"));
    assert_eq!(
        out.last().map(String::as_str),
        Some("T|LEAK_PASS"),
        "{:?}",
        out
    );
}
