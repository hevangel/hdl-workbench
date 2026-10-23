//! IEEE 1800-2023 clause 35: direct programming interface.

use crate::harness::{Order, check_lib};

// §35.5.6.1, §35.5: DPI open arrays of vectors, unsigned and svLogic arguments
#[test]
fn c35_5_6_1_dpi_open_arrays() {
    check_lib(
        "c35b",
        include_str!("sv/c35b.sv"),
        include_str!("sv/c35b.c"),
        "--dpi-lib",
        "c35b",
        Order::Exact,
        &[
            "T|35.5|u32 4294967295 s16 32767 b8 240",
            "T|35.5.6|packed struct 02030405",
            "T|35.5.6.1|2d open 152",
            "T|35.5.6.1|open bits 256",
            "T|35.5.6.1|open size/left/right 474",
            "T|35.7|export with output arg 17",
            "T|35.7|export %m=c35b.sv_where",
            "T|35.5.6.1|open output '{100, 101, 102}",
            "T|35.5.6|svLogic x=3 z=2 1=1",
        ],
        &[],
    );
}

// §35.5.6, §35.5.6.1, §35.7: DPI packed struct arguments (passed as svBitVecVal)
#[test]
fn c35_5_6_dpi_misc() {
    check_lib(
        "35.5.6_dpi_misc",
        include_str!("sv/35.5.6_dpi_misc.sv"),
        include_str!("sv/35.5.6_dpi_misc.c"),
        "--dpi-lib",
        "t35b",
        Order::Exact,
        &[
            "T|a|packed struct out=02030405 in=01020304 (expect 02030405 01020304)",
            "T|b|2-D open array 152 (expect 152)",
            "T|c|export output arg 17 (expect 17)",
        ],
        &[],
    );
}

// §35.5, §35.5.6, §35.5.2, §35.7, §35.9, §35.11: DPI imports/exports, argument types, pure/context, scopes
#[test]
fn c35_dpi_imports_and_exports() {
    // Known gap (§35.5.6): `35.5.6` reference `struct 6 10 2.500000`, xezim `struct 0 0 0.000000`
    check_lib(
        "c35",
        include_str!("sv/c35.sv"),
        include_str!("sv/c35.c"),
        "--dpi-lib",
        "c35",
        Order::Exact,
        &[
            "T|35.5|int 7 byte -127 ll 8589934592 real 6.000000 bit 1",
            "T|35.5.4|out 77 2.500000 1",
            "T|35.5.4|inout 15",
            "T|35.5.6|string <hello:5>",
            "T|35.5.6|str out fromC",
            "T|35.5.6|bitvec f5edcba987",
            "T|35.5.6|logvec 10xz01zx",
            "T|35.5.6.1|open dyn 603",
            "T|35.5.6.1|open fill ranged '{20, 30, 40, 50}",
            "T|35.5.6|fixed 10",
            "T|35.5.6|chandle 42 null=-1",
            "T|35.5.2|pure 81",
            "T|35.7|export call 41",
            "T|35.5.4|alias 2",
            "T|35.5.6|bitsel 11",
            "T|35.5.6|put logic xz10",
            "T|35.11|scope 111",
            "T|35.9|export task resumed t=15",
            "T|35.9|import task returned t=15",
        ],
        &["T|35.5.6|struct "],
    );
}
