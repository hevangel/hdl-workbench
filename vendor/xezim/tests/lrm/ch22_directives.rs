//! IEEE 1800-2023 clause 22: compiler directives.

use crate::harness::{Order, check, check_files};

// §22.9, §22.11: unconnected_drive, pragma
#[test]
fn c22_9_unconnected_drive_and_22_11_pragma() {
    check(
        "c22_more",
        include_str!("sv/c22_more.sv"),
        "c22m",
        Order::Exact,
        &["T|22.9|y1=1 y0=z", "T|22.5.1i|5", "T|22.11|pragma ignored"],
        &[],
    );
}

// §22.3, §22.4, §22.5.1, §22.5.2, §22.5.3, §22.6, ...: macros, include, ifdef, timescale, default_nettype, line
#[test]
fn c22_compiler_directives() {
    check_files(
        "c22_dir",
        include_str!("sv/c22_dir.sv"),
        "c22top",
        Order::Exact,
        &[
            "T|22.5.1a|3 15 12",
            "T|22.5.1b|hello world",
            "T|22.5.1c|3",
            "T|22.5.1e|42",
            "T|22.5.1f|12 32 14",
            "T|22.5.1g|4",
            "T|22.5.1h|3",
            "T|22.6a|ifdef ok",
            "T|22.6b|ifndef ok",
            "T|22.6c|elsif ok",
            "T|22.5.2|undef ok",
            "T|22.4|77 4",
            "T|22.10|celldefine ok",
            "T|22.14|logic as ident=1",
            "T|22.5.3|undefineall ok",
            "T|22.7|10 1.300000",
            "T|22.13|c22_dir.sv 48",
            "T|22.12|fake.sv 500",
            "T|22.8|y=1",
        ],
        &[],
        &[],
        &["c22_inc.svh"],
    );
}
