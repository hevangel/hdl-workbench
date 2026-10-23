//! IEEE 1800-2023 clause 30: specify blocks.

use crate::harness::{Order, check};

// §30.4.3: Edge-sensitive path delay to a procedurally assigned `output reg`
#[test]
fn c30_4_3_edge_path_to_output_reg() {
    // Known gap (§30.4.3): `a` reference `t=13000 q1(output reg)=1 (expect t=13)`, xezim `t=10000 q1(output reg)=1 (expect t=13)`
    check(
        "30.4.3_edge_path_to_output_reg",
        include_str!("sv/30.4.3_edge_path_to_output_reg.sv"),
        "t30_4",
        Order::Exact,
        &["T|b|t=13000 q2(output wire)=1 (expect t=13)"],
        &["T|a|"],
    );
}

// §30.7.4: showcancelled / pulsestyle_ondetect
#[test]
fn c30_7_4_showcancelled() {
    // Known gap (§30.7.4): `a`: xezim misses 4 reference lines
    check(
        "30.7.4_showcancelled",
        include_str!("sv/30.7.4_showcancelled.sv"),
        "t30_7",
        Order::Exact,
        &["T|a|t=3000 y3=0"],
        &[
            "T|a|t=21000 y3=x",
            "T|a|t=25000 y3=0",
            "T|a|t=46000 y3=1",
            "T|a|t=47000 y3=0",
        ],
    );
}

// §30.7: pulse filtering and showcancelled
#[test]
fn c30_7_pulse_filtering() {
    // Known gap (§30.7.4): `30.7.4`: xezim misses 6 reference lines
    check(
        "c30_pulse",
        include_str!("sv/c30_pulse.sv"),
        "c30_pulse",
        Order::Exact,
        &[
            "T|30.7.4|t=3000 y3=0",
            "T|30.7|t=5000 y2=0",
            "T|30.7|t=5000 y1=0",
            "T|30.7.4|t=69000 y3=1",
            "T|30.7|t=69000 y2=1",
            "T|30.7|t=69000 y1=1",
            "T|30.7.4|t=77000 y3=0",
            "T|30.7|t=79000 y2=0",
            "T|30.7|t=79000 y1=0",
        ],
        &[
            "T|30.7.4|t=21000 y3=x",
            "T|30.7.4|t=25000 y3=0",
            "T|30.7.4|t=46000 y3=1",
            "T|30.7.4|t=47000 y3=0",
            "T|30.7.4|t=95000 y3=x",
            "T|30.7.4|t=99000 y3=0",
        ],
    );
}

// §30.3, §30.4, §30.4.3, §30.4.4: specify paths, edge-sensitive paths, state-dependent paths
#[test]
fn c30_specify_paths() {
    check(
        "c30_spec",
        include_str!("sv/c30_spec.sv"),
        "c30_spec",
        Order::Exact,
        &[
            "T|30.4|t=1500 z=1",
            "T|30.4|t=3000 y=0",
            "T|30.4.4|t=7000 w=0",
            "T|30.4|t=22000 y=1",
            "T|30.4|t=22500 z=0",
            "T|30.4.4|t=27000 w=1",
            "T|30.4|t=41500 z=1",
            "T|30.4|t=43000 y=0",
            "T|30.4.4|t=47000 w=0",
            "T|30.4|t=82000 y=1",
            "T|30.4|t=82500 z=0",
            "T|30.4.4|t=87000 w=1",
            "T|30.4.3|t=110000 q=1",
            "T|30.4.3|t=136000 q=0",
            "T|30.4|t=174000 y=0",
            "T|30.4.4|t=178000 w=0",
            "T|30.4|t=194000 y=1",
            "T|30.4.4|t=198000 w=1",
            "T|30.4|t=211500 z=1",
            "T|30.4|t=213000 y=0",
            "T|30.4.4|t=217000 w=0",
            "T|30.4|t=232000 y=1",
            "T|30.4|t=232500 z=0",
            "T|30.4.4|t=237000 w=1",
        ],
        &[],
    );
}
