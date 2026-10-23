//! IEEE 1800-2023 clause 34: protected envelopes.

use crate::harness::{Order, check};

// §34: Protected envelopes: unencrypted `pragma protect begin/end region
#[test]
fn c34_protect_region_dropped() {
    // Known gap (§34): `34`: xezim misses `inside protect region`
    check(
        "34_protect_region_dropped",
        include_str!("sv/34_protect_region_dropped.sv"),
        "c34_prot",
        Order::Exact,
        &["T|34|before", "T|34|after"],
        &["T|34|inside protect region"],
    );
}
