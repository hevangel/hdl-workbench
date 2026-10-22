//! IEEE 1800-2023 clause 12: procedural programming statements.

use crate::harness::{Order, check};

// §12.4.2, §12.5.3: unique/priority-if violation reports (overlap, no match)
#[test]
fn c12_4_2_unique_priority_no_violation_report() {
    check(
        "12.4.2_unique_priority_no_violation_report",
        include_str!("sv/12.4.2_unique_priority_no_violation_report.sv"),
        "fun",
        Order::Exact,
        &["T|12.4.2|done r=1"],
        &[],
    );
}

// §12.4, §12.4.2, §12.5, §12.5.1, §12.5.3, §12.5.4, ...: if/case forms, loops, jump statements, pattern matching
#[test]
fn c12_procedural_statements() {
    // Known gap (§12.7.5): `12.7.5d` reference `00 10 11`, xezim `031 030 029 028 027 026 025 024 023 022 021 020 019 018 017 016 015...`
    check(
        "c12_proc",
        include_str!("sv/c12_proc.sv"),
        "c12",
        Order::Exact,
        &[
            "T|12.4a|2",
            "T|12.4b|1",
            "T|12.4.2a|1",
            "T|12.4.2b|1",
            "T|12.4.2c|1",
            "T|12.4.2d|1",
            "T|12.5a|1",
            "T|12.5b|1",
            "T|12.5c|1",
            "T|12.5d|1",
            "T|12.5e|1",
            "T|12.5f|1",
            "T|12.5g|2",
            "T|12.5h|2",
            "T|12.5i|10",
            "T|12.5j|2",
            "T|12.5.3a|2",
            "T|12.5.3b|2",
            "T|12.5.3c|7",
            "T|12.5.3d|7",
            "T|12.5.4a|2",
            "T|12.5.4b|1",
            "T|12.5.4c|2",
            "T|12.7.1|4",
            "T|12.7.2a|5",
            "T|12.7.2b|0",
            "T|12.7.2c|0",
            "T|12.7.3|6",
            "T|12.7.4|11",
            "T|12.7.5a|00,01,02,10,11,12, '{'{0, 1, 2}, '{10, 11, 12}}",
            "T|12.7.5b|321",
            "T|12.7.5c|amz",
            "T|12.7.5e|'{14, 16}",
            "T|12.7.5f|31 30 21 20 11 10 01 00",
            "T|12.7.5g|0 1",
            "T|12.7.6|4",
            "T|12.8a|12",
            "T|12.8b|3",
            "T|12.8c|5",
            "T|12.8d|10",
            "T|12.8e|ret",
            "T|12.6.2|B=9",
        ],
        &["T|12.7.5d|"],
    );
}
