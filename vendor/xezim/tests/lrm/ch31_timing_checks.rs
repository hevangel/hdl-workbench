//! IEEE 1800-2023 clause 31: timing checks.

use crate::harness::{Order, check};

// §31.3.1, §31.3.2, §31.3.3, §31.3.4, §31.3.5, §31.4.1, ...: setup/hold/setuphold/recovery/removal/width/period/nochange
#[test]
fn c31_timing_checks() {
    // Timing checks that fire in the same time step report from separate notifier processes; IEEE 1800 §4.7 leaves their order open.
    check(
        "c31_tc",
        include_str!("sv/c31_tc.sv"),
        "c31_tc",
        Order::Multiset,
        &[
            "T|31.3.3|setuphold notifier t=40000 0",
            "T|31.3.1|setup notifier t=40000 0",
            "T|31.3.3|setuphold notifier t=41000 1",
            "T|31.4.5|nochange notifier t=41000 0",
            "T|31.4.4|width notifier t=43000 0",
            "T|31.4.3|period notifier t=50000 0",
            "T|31.3.5|recovery notifier t=67000 0",
            "T|31.3.5|recovery notifier t=89000 1",
        ],
        &[],
    );
}

// §31.5, §31.7, §31.3.6: timing check conditions, notifiers, $fullskew
#[test]
fn c31_timing_checks_more() {
    // Known gap (§31.4.2): `31.4.2`: xezim misses `fullskew notifier t=2000`
    // Timing checks that fire in the same time step report from separate notifier processes; IEEE 1800 §4.7 leaves their order open.
    check(
        "c31_more",
        include_str!("sv/c31_more.sv"),
        "c31_more",
        Order::Multiset,
        &[
            "T|31.4.2|fullskew notifier t=12000",
            "T|31.4.2|timeskew notifier t=12000",
            "T|31.7|cond setup notifier t=25000",
            "T|31.5|edge hold notifier t=25500",
            "T|31.4.2|fullskew notifier t=27000",
            "T|31.4.2|timeskew notifier t=27000",
            "T|31.4.2|fullskew notifier t=30000",
            "T|31.4.2|fullskew notifier t=32500",
            "T|31.4.2|timeskew notifier t=32500",
            "T|31.3.6|recrem notifier t=36700",
            "T|31.4.2|fullskew notifier t=38700",
            "T|31.4.2|timeskew notifier t=38700",
        ],
        &["T|31.4.2|fullskew notifier t=2000"],
    );
}
