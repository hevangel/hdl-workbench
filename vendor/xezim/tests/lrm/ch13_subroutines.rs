//! IEEE 1800-2023 clause 13: tasks and functions.

use crate::harness::{Order, check};

// §13.3: static task variables shared by concurrent calls
#[test]
fn c13_3_static_and_automatic_tasks() {
    // Known gap (§13.3.1): `13.3.1` reference `o1=2 o2=2`, xezim `o1=1 o2=2`
    check(
        "c13_more",
        include_str!("sv/c13_more.sv"),
        "c13x",
        Order::Exact,
        &[
            "T|13.4b|14 6",
            "T|13.4.1d|6",
            "T|13.2a|n=3",
            "T|13.4.2b|10",
            "T|13.5.2g|7 8",
            "T|13.4.3b|8",
            "T|13.4.4|1 3",
            "T|13.5.1c|12",
            "T|13.5.1d|1",
        ],
        &["T|13.3.1|"],
    );
}

// §13.3.1, §13.4.1, §13.2: static task variables shared by concurrent calls, output arguments
#[test]
fn c13_3_static_task_shared_by_calls() {
    // Known gap (§13.3.1): `r1` reference `o1=2 o2=2`, xezim `o1=1 o2=2`
    check(
        "r_c13b",
        include_str!("sv/r_c13b.sv"),
        "r13b",
        Order::Exact,
        &["T|r2|14 6", "T|r3|n=3"],
        &["T|r1|"],
    );
}

// §13.5.1: Pass by value of a dynamic array or queue
#[test]
fn c13_5_1_dynarray_by_value() {
    check(
        "13.5.1_dynarray_by_value",
        include_str!("sv/13.5.1_dynarray_by_value.sv"),
        "fdv",
        Order::Exact,
        &["T|13.5.1d|d='{1, 2} q='{1, 2}"],
        &[],
    );
}

// §13.5.2: Waiting on a ref argument inside a task (@(posedge sig) / wait(sig))
#[test]
fn c13_5_2_ref_arg_event_wait() {
    check(
        "13.5.2_ref_arg_event_wait",
        include_str!("sv/13.5.2_ref_arg_event_wait.sv"),
        "rrw",
        Order::Exact,
        &["T|r1|tt=4 now=4", "T|r2|seen=7"],
        &[],
    );
}

// §13.5.2: ref queue argument shared by concurrently running tasks
#[test]
fn c13_5_2_ref_queue_concurrent() {
    check(
        "13.5.2_ref_queue_concurrent",
        include_str!("sv/13.5.2_ref_queue_concurrent.sv"),
        "rrc",
        Order::Exact,
        &[
            r#"T|r1|'{"B", "A"}"#,
            "T|r2|2",
            "T|r3|during=0",
            "T|r4|after=1",
            "T|r5|101",
        ],
        &[],
    );
}

// §13.5.3: Default argument value that refers to an earlier argument (int b = a + 1)
#[test]
fn c13_5_3_default_refers_other_arg() {
    // Known gap (§13.5.3): `r1` reference `130 127 451`, xezim `130 x x`
    check(
        "13.5.3_default_refers_other_arg",
        include_str!("sv/13.5.3_default_refers_other_arg.sv"),
        "rad",
        Order::Exact,
        &["T|r2|127 421 526"],
        &["T|r1|"],
    );
}

// §13.4.2, §7.10: Recursive function over a queue slice passed as argument (rsum(q[1:$]))
#[test]
fn c13_5_queue_slice_argument() {
    check(
        "13.5_queue_slice_argument",
        include_str!("sv/13.5_queue_slice_argument.sv"),
        "r13a3",
        Order::Exact,
        &["T|r1|3", "T|r2|1", "T|r3|0"],
        &[],
    );
}

// §13.5: queue slice argument recursion crash
#[test]
fn c13_5_queue_slice_argument_recursion_crash() {
    check(
        "13.5_queue_slice_argument_recursion_crash",
        include_str!("sv/13.5_queue_slice_argument_recursion_crash.sv"),
        "r13a2",
        Order::Exact,
        &["T|r1|3 0", "T|r2|10"],
        &[],
    );
}

// §13.5: recursive function over queue slices
#[test]
fn c13_5_queue_slice_recursion() {
    check(
        "r_c13a",
        include_str!("sv/r_c13a.sv"),
        "r13a",
        Order::Exact,
        &["T|r1|10"],
        &[],
    );
}

// §13.5: struct and array arguments by value and by ref
#[test]
fn c13_5_struct_and_array_arguments() {
    check(
        "r_c13c",
        include_str!("sv/r_c13c.sv"),
        "r13c",
        Order::Exact,
        &["T|r1|7 8", "T|r2|8", "T|r3|1 3", "T|r4|12", "T|r5|1"],
        &[],
    );
}

// §3.8, §13.3, §13.4, §13.4.1, §13.4.2, §13.4.3, ...: task/function arguments, defaults, ref, void functions, recursion
#[test]
fn c13_tasks_and_functions() {
    // Known gap (§13.3): `13.3a` reference `1 2`, xezim `1 1`
    // Known gap (§13.4): `13.4a` reference `1 2 1 1`, xezim `1 1 1 1`
    // Known gap (§13.5.3): `13.5.3a` reference `130 240 240 127`, xezim `130 240 240 x`
    // Known gap (§13.5.3): `13.5.3b` reference `451`, xezim `x`
    check(
        "c13_tf",
        include_str!("sv/c13_tf.sv"),
        "c13",
        Order::Exact,
        &[
            "T|13.3b|1 1",
            "T|13.4.1a|0 0",
            "T|13.4.2|3628800",
            "T|13.4.3|W=5 bits=5",
            "T|13.5a|5 6 101",
            "T|13.5.2a|10",
            "T|13.4.1c|5",
            "T|13.5.2b|4",
            "T|13.5.3c|6",
            "T|13.5.4|11",
            "T|13.5.5a|6 24",
            "T|13.5.5b|'{4, 99} 103",
            "T|13.5.5c|'{7, 8, 9} '{-1, 8, 9}",
            "T|13.4.5|3 6 10",
            "T|13.4.4|bgv=7",
            "T|13.5.1|o1 during=11",
            "T|13.5.1b|o1 after=2",
            "T|13.8|15",
            "T|13.5.2c|during g=1",
            "T|13.5.2e|after g=5",
            "T|13.5.2d|during g=6",
            "T|13.5.2f|after g=6",
            "T|13.4.1b|bgv=2",
        ],
        &["T|13.3a|", "T|13.4a|", "T|13.5.3a|", "T|13.5.3b|"],
    );
}
