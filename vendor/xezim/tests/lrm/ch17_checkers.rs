//! IEEE 1800-2023 clause 17: checkers.

use crate::harness::{Order, check};

// §17.3, §17.5: Checker instantiation at module level; assertion inside checker
#[test]
fn c17_7_checker_always_ff() {
    // Known gap (§17.7): `17.7` reference `nreq = 3`, xezim `nreq = 0`
    check(
        "17.7_checker_always_ff",
        include_str!("sv/17.7_checker_always_ff.sv"),
        "c17_simple",
        Order::Exact,
        &["T|17.2|req_ack F t=65"],
        &["T|17.7|"],
    );
}

// §17.8, §17.3.2: functions and covergroups in checkers
#[test]
fn c17_checkers_more() {
    // Known gap (§17.8): `17.8`: xezim misses 3 reference lines
    // Known gap (§17.6): `17.6` reference `cg cov=100.00 cnt=2`, xezim `cg cov=0.00 cnt=0`
    check(
        "c17_more",
        include_str!("sv/c17_more.sv"),
        "c17_more",
        Order::Exact,
        &["T|17.3.2|inner F t=25"],
        &["T|17.8|", "T|17.6|"],
    );
}
