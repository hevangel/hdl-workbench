//! IEEE 1800-2023 clause 29: user-defined primitives.

use crate::harness::{Order, check};

// §29.3, §29.4, §29.5, §29.6, §29.7: combinational and sequential UDPs, initial statements, edge rows
#[test]
fn c29_udps() {
    check(
        "c29_udp",
        include_str!("sv/c29_udp.sv"),
        "c29_udp",
        Order::Exact,
        &[
            "T|29.6|init q=1 qr=0",
            "T|29.3|s=0 a=0 b=0 m=0 nm=x",
            "T|29.3|s=0 a=0 b=1 m=0 nm=x",
            "T|29.3|s=0 a=0 b=x m=0 nm=x",
            "T|29.3|s=0 a=1 b=0 m=1 nm=1",
            "T|29.3|s=0 a=1 b=1 m=1 nm=1",
            "T|29.3|s=0 a=1 b=x m=1 nm=1",
            "T|29.3|s=0 a=x b=0 m=x nm=x",
            "T|29.3|s=0 a=x b=1 m=x nm=x",
            "T|29.3|s=0 a=x b=x m=x nm=x",
            "T|29.3|s=1 a=0 b=0 m=0 nm=x",
            "T|29.3|s=1 a=0 b=1 m=1 nm=x",
            "T|29.3|s=1 a=0 b=x m=x nm=x",
            "T|29.3|s=1 a=1 b=0 m=0 nm=1",
            "T|29.3|s=1 a=1 b=1 m=1 nm=1",
            "T|29.3|s=1 a=1 b=x m=x nm=1",
            "T|29.3|s=1 a=x b=0 m=0 nm=x",
            "T|29.3|s=1 a=x b=1 m=1 nm=x",
            "T|29.3|s=1 a=x b=x m=x nm=x",
            "T|29.3|s=x a=0 b=0 m=0 nm=x",
            "T|29.3|s=x a=0 b=1 m=x nm=x",
            "T|29.3|s=x a=0 b=x m=x nm=x",
            "T|29.3|s=x a=1 b=0 m=x nm=1",
            "T|29.3|s=x a=1 b=1 m=1 nm=1",
            "T|29.3|s=x a=1 b=x m=x nm=1",
            "T|29.3|s=x a=x b=0 m=x nm=x",
            "T|29.3|s=x a=x b=1 m=x nm=x",
            "T|29.3|s=x a=x b=x m=x nm=x",
            "T|29.4|latch 0",
            "T|29.4|latch hold 0",
            "T|29.4|latch en=x x",
            "T|29.5|dff q=0 qr=0",
            "T|29.5|d change no clk q=0 qr=0",
            "T|29.5|posedge q=1 qr=1",
            "T|29.5|1->x q=x qr=x",
            "T|29.5|0->x d=0 q=x qr=x",
            "T|29.5|rst qr=0",
            "T|29.7|delayed md=0",
        ],
        &[],
    );
}
