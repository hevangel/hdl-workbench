//! `std::randomize()` with targets that are locals of a STATIC function/task.
//!
//! §18.12: the targets of `std::randomize(v, a)` are the variables of the
//! joint solve. A static subroutine local resolves to the scope-qualified name
//! `<fn>.<var>`; `std_randomize_vars` mistook that dot for a packed-struct
//! field and compared the field's parent against only the FIRST dot segment,
//! which is never a target. The guard then returned `NotApplicable` for the
//! whole call, so the call fell back to independent draws that cannot satisfy a
//! joint `a.sum()`, and a satisfiable constraint set reported failure.
//!
//! Every pre-existing local-variable randomize test in this group is
//! `automatic`, where the name carries no scope prefix — hence the gap.

use xezim::simulate;

fn t_lines(src: &str) -> Vec<String> {
    simulate(src, 100)
        .expect("simulate failed")
        .output
        .iter()
        .map(|o| o.message.clone())
        .filter(|m| {
            m.starts_with("T|") || m.starts_with("TEST_") || m.starts_with("CHECK_FAIL")
        })
        .collect()
}

#[test]
fn std_randomize_static_subroutine_locals() {
    assert_eq!(
        t_lines(include_str!("randomize_std_static_subroutine_locals_513.sv")),
        [
            "T|checks=53 failed=0",
            "TEST_PASS",
        ]
    );
}