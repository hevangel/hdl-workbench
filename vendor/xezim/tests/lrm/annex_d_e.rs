//! IEEE 1800-2023 Annexes D and E: optional system tasks and compiler directives.

use crate::harness::{Order, check};

// §D, §E: Optional system tasks: $countdrivers, $sreadmemh
#[test]
fn annex_d_countdrivers_sreadmemh_delaymode() {
    // Known gap (§D.2): `D.2` reference `countdrivers ret=1 nd=2 n0=0 n1=2 nx=0`, xezim `countdrivers ret=0 nd=0 n0=0 n1=0 nx=0`
    // Known gap (§D.17): `D.17` reference `sreadmemh 0a 0b 0c 0d`, xezim `sreadmemh xx xx xx xx`
    // Known gap (§E): `E` reference `delay_mode_zero y=1 t=2`, xezim `delay_mode_zero y=x t=2`
    check(
        "D_countdrivers_sreadmemh_delaymode",
        include_str!("sv/D_countdrivers_sreadmemh_delaymode.sv"),
        "cAnnexD",
        Order::Exact,
        &["T|D.3|getpattern ok"],
        &["T|D.2|", "T|D.17|", "T|E|"],
    );
}
