//! `randomize() with { ... }` inside a module instance (issues #303, #304
//! and #305, reported by Ganesh-TNL).
//!
//! §18.7 / §18.12: the names of an inline constraint block bind in the scope
//! of the call — for `obj.randomize() with`, after the object's own class
//! scope. Inlining a module instance never rewrote the block (nor an array
//! method's §7.12 `with (...)` clause), so its names stayed bare leaves:
//! - #303: a fixed-size array of the instance was never constrained;
//! - #304: `delays.size()` of the instance's dynamic array read 0 in the
//!   checker, which rejected every solution;
//! - #305: the instance's `std::randomize(delays)` filled the top module's
//!   same-named array instead of its own.
//!
//! Expected values come from the reference simulator. Random values differ
//! between simulators, so the probes print whether the constraints hold.

use xezim::simulate;

fn t_lines(src: &str) -> Vec<String> {
    simulate(src, 100)
        .expect("simulate failed")
        .output
        .iter()
        .map(|o| o.message.clone())
        .filter(|m| m.starts_with("T|"))
        .collect()
}

#[test]
fn issues_303_305_instance_randomize_with() {
    assert_eq!(
        t_lines(include_str!("randomize_instance_scope_issues.sv")),
        [
            "T|303 PASS",
            "T|304 PASS size=16",
            "T|305 child size=16 inrange=1",
            "T|305 top size=0",
        ]
    );
}

/// Two instances of one child, generate scopes, `obj.randomize() with`
/// reading instance variables (and an object member named like one),
/// `local::`, queues and dynamic arrays, and `with` clauses of array methods.
#[test]
fn instance_randomize_with_shapes() {
    assert_eq!(
        t_lines(include_str!("randomize_instance_scope_shapes.sv")),
        [
            "T|top.a fixed ok=1 inrange=1",
            "T|top.a dyn ok=1 size=4 inrange=1",
            "T|top.b fixed ok=1 inrange=1",
            "T|top.b dyn ok=1 size=13 inrange=1",
            "T|top.k.gb[0] g=0 ok=1 inr=1",
            "T|top.k.gb[0] g=0 qok=1 qsize=2 q0=10 qlast=11",
            "T|top.k.gb[1] g=1 ok=1 inr=1",
            "T|top.k.gb[1] g=1 qok=1 qsize=3 q0=20 qlast=22",
            "T|top.k.ib ok=1 a=7 8 9",
            "T|top.c c1 ok=1 in=1 y=1 modx=99",
            "T|top.c c2 ok=1 x=22 a=13 24 35",
            "T|top.c c3 ok=1 x=22",
            "T|top.c c4 ok=1 x=121",
            "T|top.l q ok=1 size=5 sum=20",
            "T|top.l dq ok=1 size=4 d0=-5 d3=-2",
            "T|top.r v ok=1 in=1",
            "T|top.r vw ok=1 v=41 w=86",
            "T|top.r sumwith=10",
            "T|top.r find=2",
            "T|top.r sumv ok=1 v=10",
            "T|top fa0=0 da=0 lim=5 arr0=0 x=3 q=0 v=0 w=0 lo=1000",
        ]
    );
}

/// The same constructs in a task and a function of the instance.
#[test]
fn instance_randomize_with_in_subroutines() {
    assert_eq!(
        t_lines(include_str!("randomize_instance_scope_subroutines.sv")),
        [
            "T|top.k1.go task ok=1 a0=105 a3=108",
            "T|top.k1 total=26",
            "T|top.k1 q=2 100 101",
            "T|top.k2.go task ok=1 a0=105 a3=108",
            "T|top.k2 total=26",
            "T|top.k2 q=2 100 101",
            "T|top a0=0 lim=7",
        ]
    );
}
