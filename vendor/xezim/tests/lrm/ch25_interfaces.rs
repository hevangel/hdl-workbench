//! IEEE 1800-2023 clause 25: interfaces.

use crate::harness::{Order, check};

// §25.8: parameterized interfaces
#[test]
fn c25_8_parameterized_interfaces() {
    // Known gap (§25.10): `25.10b` reference `-5 300 4`, xezim `65531 300 4`
    // Known gap (§25.9): `25.9g` reference `-5`, xezim `65531`
    check(
        "c25_more",
        include_str!("sv/c25_more.sv"),
        "c25x",
        Order::Exact,
        &["T|25.3.5|127 1", "T|25.3.4|16 300", "T|25.3.6|2 0"],
        &["T|25.10b|", "T|25.9g|"],
    );
}

// §25.10: an omitted virtual-interface argument binds the formal's DEFAULT
// (also §13.5.3) — both for reading an `input` vif and for copying an
// `inout` binding back to the caller, across tasks, functions and class
// methods. Regression for #469.
#[test]
fn c25_9_vif_omitted_default() {
    check(
        "c25_9_vif_default",
        include_str!("sv/c25_9_vif_default.sv"),
        "t",
        Order::Exact,
        &[
            "T|25.9.vifdef|fn-read 42",
            "T|25.9.vifdef|task-read 42",
            "T|25.9.vifdef|inout-read 43",
            "T|25.9.vifdef|after-inout 43",
            "T|25.9.vifdef|method-read 44",
        ],
        &[],
    );
}

// §14.9: clocking block reached through a virtual interface
#[test]
fn c25_9_virtual_interface_clocking() {
    check(
        "r_vifcb",
        include_str!("sv/r_vifcb.sv"),
        "rvc",
        Order::Exact,
        &[
            "T|r1|t=5 vc.cb.data=77 b.cb.data=77",
            "T|r2|t=15 vc.cb.data=12 b.cb.data=12",
        ],
        &[],
    );
}

// §25.3, §25.4, §25.5, §25.5.4, §25.7, §25.9: interfaces, modports, tasks in interfaces, virtual interfaces
#[test]
fn c25_interfaces() {
    // Known gap (§25.9): `25.9a` reference `vif data=5a valid=1 mon=5a`, xezim `vif data=6b valid=1 mon=6b`
    check(
        "c25_if",
        include_str!("sv/c25_if.sv"),
        "c25",
        Order::Exact,
        &[
            "T|25.7a|recv 6b t=5",
            "T|25.7a|recv 6b t=15",
            "T|25.5.4|lo=b",
            "T|25.3.3|generic data=beef",
            "T|25.7b|cnt=2 width=8 w16=16",
            "T|25.3|si.b=4",
            "T|25.10|16",
            "T|25.9b|1 1",
            "T|25.9c|77",
            "T|25.9d|1",
            "T|25.9e|9",
            "T|25.9f|cb data=77",
        ],
        &["T|25.9a|"],
    );
}
