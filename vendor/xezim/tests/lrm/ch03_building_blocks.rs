//! IEEE 1800-2023 clause 3: design and verification building blocks.

use crate::harness::{Order, check};

// §3.3, §3.5, §3.9, §3.11, §3.14.1, §3.14.2, ...: modules, interfaces, packages, UDPs, time units and rounding
#[test]
fn c3_building_blocks() {
    check(
        "c03",
        include_str!("sv/c03.sv"),
        "c03",
        Order::Exact,
        &["T|3.3|o=5 pv=7", "T|3.12|u=1 t=2000 rt=1.500"],
        &[],
    );
}
