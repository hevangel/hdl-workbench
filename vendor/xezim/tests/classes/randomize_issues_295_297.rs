//! `randomize` on wide-element sums, packed-struct arrays and task-local
//! collections (issues #295, #296 and #297, reported by Ganesh-TNL).
//!
//! - #295, §7.12.3: `arr.sum()` is computed in the element type, so with
//!   16-bit elements `sum() == 3` also holds for elements adding up to
//!   `3 + 65536·k`. The solver read the sum as the plain integer total and
//!   could only guess the last element once that total could wrap; the sum
//!   is now a variable tied to its terms by a carry.
//! - #296, §7.2.1/§7.4: a packed-struct array local to a task or function
//!   had no field layout, so `a[i].d = …` was dropped and `a[i].d` read 0 —
//!   the acceptance check then rejected every correct solution.
//! - #297, §18.12: a dynamic array or queue local to an automatic task is
//!   stored under a per-call key, so the joint solver did not know the name
//!   the constraints use.
//!
//! Expected values come from the reference simulator. Random values differ
//! between simulators, so the probes print whether the constraints hold.

use xezim::simulate;

fn lines(src: &str) -> Vec<String> {
    simulate(src, 100)
        .expect("simulate failed")
        .output
        .iter()
        .map(|o| o.message.clone())
        .collect()
}

fn t_lines(src: &str) -> Vec<String> {
    lines(src)
        .into_iter()
        .filter(|m| m.starts_with("T|"))
        .collect()
}

fn assert_pass(out: &[String]) {
    assert!(
        out.iter().any(|m| m.contains("TEST_PASS")),
        "no TEST_PASS: {out:?}"
    );
    assert!(
        !out.iter().any(|m| m.contains("FAIL")),
        "failure reported: {out:?}"
    );
}

#[test]
fn issue_295_sum_of_wide_elements() {
    assert_pass(&lines(include_str!("randomize_issue_295_wide_sum.sv")));
}

#[test]
fn issue_296_struct_array_field_constraints_in_task_and_function() {
    assert_pass(&lines(include_str!("randomize_issue_296_struct_locals.sv")));
}

#[test]
fn issue_297_task_local_arrays_in_joint_solve() {
    let out = lines(include_str!("randomize_issue_297_task_locals.sv"));
    assert_pass(&out);
    assert!(out.iter().any(|m| m == "OK t1 task-local: size=5 sum=3"));
    assert!(out.iter().any(|m| m.starts_with("OK t4: num_reprog_regs=")));
}

/// §7.12.3: a reduction with a `with` clause has the type of the clause's
/// expression (its width and signedness); without one, the element type.
#[test]
fn reduction_result_type() {
    let expected = [
        "T|a 124 -56 200",
        "T|b 44 300",
        "T|c 1 -2",
        "T|d 1 1 1",
        "T|e 105 13",
        "T|f 3 15 12",
        "T|g 8 16",
        "T|h 44 44",
        "T|i 0",
        "T|j 11 20",
        "T|k 16",
    ];
    assert_eq!(
        t_lines(include_str!("reduction_with_result_type.sv")),
        expected
    );
}

/// Sums wrapping at 8, 16 and 32 bits, signed elements, a `with` cast, a
/// sum inside a set, sizes tied to a drawn scalar or a range, and the other
/// reductions. `c` and `p` have no solution.
#[test]
fn sum_constraint_neighbours() {
    let expected: Vec<String> = "abcdefghijklmnop"
        .chars()
        .map(|c| match c {
            'c' | 'p' => format!("T|{c} r=0"),
            _ => format!("T|{c} r=1 ok=1"),
        })
        .collect();
    assert_eq!(
        t_lines(include_str!("randomize_sum_neighbours.sv")),
        expected
    );
}

/// `foreach` field constraints on packed-struct arrays in a class, a class
/// task and function, the module, an automatic task (also forked twice), a
/// static task, and a loop of solves in one call.
#[test]
fn struct_field_constraints_in_every_scope() {
    let expected = [
        "T|class r=1 ok=1",
        "T|ctask r=1 ok=1",
        "T|cfunc r=1 ok=1",
        "T|module r=1 ok=1",
        "T|tauto r=1 ok=1",
        "T|tauto r=1 ok=1",
        "T|tstatic r=1 ok=1",
        "T|tnest done",
        "T|tauto r=1 ok=1",
        "T|tauto r=1 ok=1",
    ];
    assert_eq!(
        t_lines(include_str!("randomize_struct_field_scopes.sv")),
        expected
    );
}

/// Field writes and reads on packed-struct collections local to a
/// function, an automatic task and a block.
#[test]
fn local_struct_array_fields() {
    let expected = [
        "T|i 4",
        "T|f 5 1 6 7",
        "T|f chk=1",
        "T|t 9 1 171",
        "T|t chk=1",
        "T|t 9 1 171",
        "T|t chk=1",
    ];
    assert_eq!(
        t_lines(include_str!("local_struct_array_fields.sv")),
        expected
    );
}

/// The #297 `t4` shape 300 times: no violation, and the `dist {1 := 2,
/// 0 := 1}` on the `bit` elements keeps its weights (the reference: 65% ones,
/// mean size 9.4). The elements were solved as signed 1-bit values, which
/// have no value 1, so every element came out 0.
#[test]
fn task_local_joint_solve_distribution() {
    let out = t_lines(include_str!("randomize_task_local_joint_dist.sv"));
    let [line] = out.as_slice() else {
        panic!("unexpected output: {out:?}");
    };
    let num = |key: &str| -> i64 {
        line.split_whitespace()
            .find_map(|f| f.strip_prefix(key))
            .and_then(|v| v.parse().ok())
            .unwrap_or_else(|| panic!("no {key} in {line}"))
    };
    assert_eq!(num("T|fails="), 0, "{line}");
    assert!((55..=77).contains(&num("ones_pct=")), "{line}");
    assert!((80..=110).contains(&num("avg_n_x10=")), "{line}");
}

/// `size() inside {[3:5]}` redraws a size inside the set; the array's
/// earlier size of 2 was kept.
#[test]
fn size_inside_set_redraws_the_size() {
    let out = t_lines(include_str!("randomize_size_inside_set.sv"));
    assert_eq!(out.len(), 4, "{out:?}");
    for l in &out {
        let n: i64 = l.rsplit("n=").next().unwrap().parse().unwrap();
        assert!(l.contains(" r=1 ") && (3..=5).contains(&n), "{l}");
    }
}
