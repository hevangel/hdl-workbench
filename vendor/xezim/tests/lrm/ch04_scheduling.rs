//! IEEE 1800-2023 clause 4: scheduling semantics.

use crate::harness::{Order, check};

// §4.4.2.1: event regions: active, inactive, NBA, observed, reactive, postponed
#[test]
fn c4_4_2_event_regions() {
    // Known gap (§4.4.2.6): `4.4.2.6a`: 2 reference lines differ from xezim's 2, e.g. reference `prog t=15 q=1`, xezim `prog t=15 q=0`
    check(
        "c04_regions",
        include_str!("sv/c04_regions.sv"),
        "c04r",
        Order::Exact,
        &[
            "T|4.4.2.1|t=5 sampled=0 now=0",
            "T|4.4.2.2a|mod t=5 q=0",
            "T|4.4.2.6a|prog t=5 q=0",
            "T|4.4.2.1|t=15 sampled=0 now=0",
            "T|4.4.2.2a|mod t=15 q=0",
            "T|4.4.2.1|t=25 sampled=1 now=1",
            "T|4.4.2.2a|mod t=25 q=1",
            "T|4.4.2.3c|r after #0 = 0",
            "T|4.4.2.9c|strobe s=0",
            "T|4.4.2.4|woke=60",
        ],
        &["T|4.4.2.6a|prog t=15 q=", "T|4.4.2.6a|prog t=25 q="],
    );
}

// §4.4.2.2, §4.4.2.3, §4.4.2.4, §4.4.2.9, §4.6, §4.9.4, ...: active/inactive/NBA/postponed ordering, $strobe, $monitor, last NBA wins
#[test]
fn c4_scheduling_order() {
    check(
        "c04_sched",
        include_str!("sv/c04_sched.sv"),
        "c04",
        Order::Exact,
        &[
            "T|4.4.2.3|after#0 a=0",
            "T|4.4.2.3b|after2#0 a=0",
            "T|4.4.2.9b|display b=0",
            "T|4.4.2.9|strobe b=1",
            "T|4.6|c=1",
            "T|4.4.2.9m|t=20 q=0",
            "T|4.4.2.9m|t=21 q=3",
            "T|4.4.2.9m|t=22 q=5",
            "T|4.4.2.9m|t=24 q=6",
            "T|4.4.2.9m|t=25 q=7",
            "T|4.6s|'{1, 2, 3}",
            "T|4.5|w=1",
            "T|4.5b|w=0",
            "T|4.6d|r1=1",
            "T|4.9nba|s1=2 s2=1",
            "T|9.2.3|final t=63",
        ],
        &[],
    );
}
