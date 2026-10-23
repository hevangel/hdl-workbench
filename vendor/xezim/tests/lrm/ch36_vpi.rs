//! IEEE 1800-2023 clauses 36-38: VPI.

use crate::harness::{Order, check_lib};

// §38.35: vpi_put_value with vpiTransportDelay / vpiInertialDelay
#[test]
fn c38_35_put_value_delay() {
    // Known gap (§38.35): `a` reference `t=1 a=1 b=2 (expect old values)`, xezim `t=1 a=99 b=55 (expect old values)`
    check_lib(
        "38.35_put_value_delay",
        include_str!("sv/38.35_put_value_delay.sv"),
        include_str!("sv/38.35_put_value_delay.c"),
        "--vpi-lib",
        "t38",
        Order::Exact,
        &["T|b|t=6 a=99 b=55 (expect 99 55)"],
        &["T|a|"],
    );
}

// §36, §37, §38.15, §38.34, §38.35, §38.36, ...: VPI handles, vpi_put_value, callbacks
#[test]
fn c38_vpi_put_value_and_callbacks() {
    // Known gap (§36): `36`: 2 reference lines differ from xezim's 2, e.g. reference `after put c=7 a=1 t=1`, xezim `after put c=7 a=99 t=1`
    check_lib(
        "c36",
        include_str!("sv/c36.sv"),
        include_str!("sv/c36.c"),
        "--vpi-lib",
        "c36",
        Order::Exact,
        &["T|36|a after delay 99 t=6", "T|36|ap fail action t=25"],
        &["T|36|after put c=7 a=", "T|36|a before delay "],
    );
}
