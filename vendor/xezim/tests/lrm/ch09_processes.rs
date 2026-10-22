//! IEEE 1800-2023 clause 9: processes.

use crate::harness::{Order, check};

// §9.6.2: disable of a named block from inside a fork child of that block
#[test]
fn c9_6_2_disable_block_from_fork() {
    check(
        "9.6.2_disable_block_from_fork",
        include_str!("sv/9.6.2_disable_block_from_fork.sv"),
        "rdf",
        Order::Exact,
        &[r#"T|r1|'{"H"} t=6"#],
        &[],
    );
}

// §9.2.1, §9.2.2.2, §9.2.2.3, §9.2.3, §9.3.1, §9.3.2, ...: always variants, fork/join variants, wait fork, disable fork, process class
#[test]
fn c9_processes() {
    // Known gap (§9.7): `9.7e` reference `KILLED`, xezim `RUNNING`
    check(
        "c09_proc",
        include_str!("sv/c09_proc.sv"),
        "c09",
        Order::Exact,
        &[
            "T|9.2.2.2a|y_comb=0 y_star=x cnt=1",
            "T|9.2.2.2b|y_comb=3 y_star=x",
            "T|9.2.2.2c|y_comb=4 y_star=4",
            "T|9.2.2.3|latch=7",
            "T|9.2.2.3b|latch=7",
            "T|9.2.2.4|ff=8",
            r#"T|9.3.1a|'{"B", "A"} t=19"#,
            r#"T|9.3.1b|'{"D", "after_any"}"#,
            r#"T|9.6.1|'{"D", "after_any", "C"}"#,
            r#"T|9.3.1c|'{"after_none"}"#,
            r#"T|9.3.1d|'{"after_none", "E"}"#,
            r#"T|9.3.2|'{"k0", "k1", "k2"}"#,
            r#"T|9.6.3|'{"G"}"#,
            r#"T|9.6.2|'{"H"}"#,
            "T|9.4.1a|2",
            "T|9.4.1b|0",
            "T|9.4.2a|ev_cnt=2",
            "T|9.4.2b|mb_pos=2",
            "T|9.4.2c|'{2, 1}",
            "T|9.4.2d|edge ok 2",
            "T|9.4.3|woke t=7",
            "T|9.4.5a|ib=2",
            "T|9.4.5b|ib=2",
            "T|9.4.5c|ib=3",
            "T|9.4.5d|ib=5",
            "T|9.4.5e|ib=5",
            "T|9.4.5f|ib=9",
            "T|9.7a|WAITING WAITING",
            "T|9.7b|SUSPENDED",
            "T|9.7c|FINISHED",
            "T|9.7d|KILLED",
            "T|9.7f|await FINISHED",
            "T|9.2.3|final",
        ],
        &["T|9.7e|"],
    );
}

// §9.2.2.1, §9.2.2.2.1, §9.2.2.4, §9.4.2.1, §9.4.3, §9.6.2: always_comb/always_latch/always_ff, event controls, disable
#[test]
fn c9_processes_more() {
    // Known gap (§9.4.2): `9.4.2e` reference `sab_cnt=1`, xezim `sab_cnt=3`
    check(
        "c09_more",
        include_str!("sv/c09_more.sv"),
        "c09x",
        Order::Exact,
        &[
            "T|9.4.2.1|pe=4 ne=4",
            "T|9.2.2.2d|10 10",
            "T|9.2.2.2e|12 12",
            "T|9.2.2.2f|22 22",
            "T|9.2.2.2g|3",
            "T|9.2.2.2h|0",
            "T|9.4.2f|arr_cnt=1",
            "T|9.2.2.1|fo=a",
            "T|9.2.2.4b|cnt=0",
            "T|9.2.2.4c|cnt=1",
            "T|9.4.2g|rcnt=2",
            "T|9.4.3b|dt=0",
            "T|9.6.2b|after disable task t=32 done=0",
            "T|9.6.2c|k=5",
        ],
        &["T|9.4.2e|"],
    );
}
