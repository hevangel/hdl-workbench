//! IEEE 1800-2023 clause 27: generate constructs.

use crate::harness::{Order, check};

// §27.4: loop generate constructs
#[test]
fn c27_4_loop_generate() {
    check(
        "c27_more",
        include_str!("sv/c27_more.sv"),
        "c27x",
        Order::Exact,
        &[
            "T|27.4d|'{0, 77, 6, 9}",
            "T|27.4e|2 9",
            "T|27.4f|4 9",
            "T|27.5c|200",
            "T|27.4g|1 11",
            "T|27.4h|6 4 2",
            "T|23.10.1b|77",
        ],
        &[],
    );
}
