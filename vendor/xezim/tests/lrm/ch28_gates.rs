//! IEEE 1800-2023 clause 28: gate-level and switch-level modeling.

use crate::harness::{Order, check};

// §28.12: Strength resolution between gates of different strength
#[test]
fn c28_12_gate_strength_conflict() {
    // Known gap (§28.12): `a` reference `a=0 b=1 w5=1/St1 w8=1/St1 (strong 1 should win)`, xezim `a=0 b=1 w5=x/StX w8=1/We1 (strong 1 should win)`
    // Known gap (§28.12): `b` reference `a=1 b=0 w5=0/St0 w8=0/St0 (strong 0 should win)`, xezim `a=1 b=0 w5=x/StX w8=0/We0 (strong 0 should win)`
    check(
        "28.12_gate_strength_conflict",
        include_str!("sv/28.12_gate_strength_conflict.sv"),
        "t28_12",
        Order::Exact,
        &["T|c|pullup vs pull0 pu=x/PuX (equal strength -> x)"],
        &["T|a|", "T|b|"],
    );
}

// §28.16: gate and net delays
#[test]
fn c28_16_gate_delays() {
    // Known gap (§28.16): `28.16` reference `t=45000 o3=z`, xezim `t=42000 o3=z`
    check(
        "c28_delay",
        include_str!("sv/c28_delay.sv"),
        "c28_delay",
        Order::Exact,
        &[
            "T|28.16|t=2000 o5=1",
            "T|28.16|t=2000 o4(typ)=0",
            "T|28.16|t=2000 o3=0",
            "T|28.16|t=3000 o1=0",
            "T|28.16|t=5000 o2=0",
            "T|28.16|t=11000 o3=1",
            "T|28.16|t=12000 o5=0",
            "T|28.16|t=12000 o4(typ)=1",
            "T|28.16|t=12000 o2=1",
            "T|28.16|t=13000 o1=1",
            "T|28.16|t=22000 o5=1",
            "T|28.16|t=22000 o4(typ)=0",
            "T|28.16|t=22000 o3=0",
            "T|28.16|t=23000 o1=0",
            "T|28.16|t=25000 o2=0",
            "T|28.16|t=31000 o3=1",
            "T|28.16|t=33000 o3=0",
            "T|28.16|t=53000 o3=0",
            "T|28.16|t=62000 o3=x",
            "T|28.16|t=63000 o5=x",
            "T|28.16|t=63000 o4(typ)=x",
            "T|28.16|t=63000 o2=x",
            "T|28.16|t=64000 o1=x",
            "T|28.16|t=72000 o3=1",
            "T|28.16|t=73000 o5=0",
            "T|28.16|t=73000 o4(typ)=1",
            "T|28.16|t=73000 o2=1",
            "T|28.16|t=74000 o1=1",
        ],
        &["T|28.16|t=4"],
    );
}

// §28.16: Three-delay turn-off delay on bufif
#[test]
fn c28_16_turnoff_delay() {
    // Known gap (§28.16): `a` reference `t=14 o3=z (turn-off delay 4: z expected at 14)`, xezim `t=11 o3=z (turn-off delay 4: z expected at 14)`
    check(
        "28.16_turnoff_delay",
        include_str!("sv/28.16_turnoff_delay.sv"),
        "t28_16",
        Order::Exact,
        &["T|a|t=1 o3=1 (turn-off delay 4: z expected at 14)"],
        &[
            "T|a|t=14 o3=z (turn-off delay 4: z expected at 14)",
            "T|a|t=11 o3=z (turn-off delay 4: z expected at 14)",
        ],
    );
}

// §28.3.6: trireg charge storage
#[test]
fn c28_3_6_trireg() {
    // Known gap (§28.13): `28.13`: 2 reference lines differ from xezim's 2, e.g. reference `stored tr1=Me1 tr2=Sm1 tr3=La1`, xezim `stored tr1=St1 tr2=St1 tr3=St1`
    check(
        "c28_trireg",
        include_str!("sv/c28_trireg.sv"),
        "c28_trireg",
        Order::Exact,
        &[
            "T|28.13|driven tr1=St1 tr2=St1 tr3=St1",
            "T|28.3.6|array bus=1010",
        ],
        &["T|28.13|stored tr1=", "T|28.13|after decay tr3="],
    );
}

// §28.7: MOS and bidirectional switches
#[test]
fn c28_7_mos_switches() {
    // Known gap (§28.7): `28.7`: 14 reference lines differ from xezim's 14, e.g. reference `d=0 g=0 nmos=z/HiZ pmos=0/St0 cmos=z/HiZ rnmos=HiZ rpmos=Pu0 rcmos=HiZ`, xezim `d=0 g=0 nmos=z/HiZ pmos=0/St0 cmos=z/HiZ rnmos=HiZ rpmos=St0 rcmos=HiZ`
    // Known gap (§28.8): `28.8`: 12 reference lines differ from xezim's 12, e.g. reference `d=0 en=0 tran=0/St0 tif1=z/HiZ tif0=0/St0 rtran=Pu0`, xezim `d=0 en=0 tran=0/St0 tif1=z/HiZ tif0=0/St0 rtran=St0`
    check(
        "c28_mos",
        include_str!("sv/c28_mos.sv"),
        "c28_mos",
        Order::Exact,
        &[
            "T|28.7|d=z g=0 nmos=z/HiZ pmos=z/HiZ cmos=z/HiZ rnmos=HiZ rpmos=HiZ rcmos=HiZ",
            "T|28.7|d=z g=1 nmos=z/HiZ pmos=z/HiZ cmos=z/HiZ rnmos=HiZ rpmos=HiZ rcmos=HiZ",
            "T|28.8|d=z en=0 tran=z/HiZ tif1=z/HiZ tif0=z/HiZ rtran=HiZ",
            "T|28.8|d=z en=1 tran=z/HiZ tif1=z/HiZ tif0=z/HiZ rtran=HiZ",
            "T|28.8|d=z en=x tran=z/HiZ tif1=z/HiZ tif0=z/HiZ rtran=HiZ",
            "T|28.8|d=z en=z tran=z/HiZ tif1=z/HiZ tif0=z/HiZ rtran=HiZ",
        ],
        &[
            "T|28.7|d=0 g=0 nmos=z/HiZ pmos=0/St0 cmos=z/HiZ rnmos=HiZ rpmos=",
            "T|28.7|d=0 g=1 nmos=0/St0 pmos=z/HiZ cmos=0/St0 rnmos=",
            "T|28.7|d=0 g=x nmos=x/St",
            "T|28.7|d=0 g=z nmos=x/St",
            "T|28.7|d=1 g=0 nmos=z/HiZ pmos=1/St1 cmos=z/HiZ rnmos=HiZ rpmos=",
            "T|28.7|d=1 g=1 nmos=1/St1 pmos=z/HiZ cmos=1/St1 rnmos=",
            "T|28.7|d=1 g=x nmos=x/St",
            "T|28.7|d=1 g=z nmos=x/St",
            "T|28.7|d=x g=0 nmos=z/HiZ pmos=x/StX cmos=z/HiZ rnmos=HiZ rpmos=",
            "T|28.7|d=x g=1 nmos=x/StX pmos=z/HiZ cmos=x/StX rnmos=",
            "T|28.7|d=x g=x nmos=x/StX pmos=x/StX cmos=x/StX rnmos=",
            "T|28.7|d=x g=z nmos=x/StX pmos=x/StX cmos=x/StX rnmos=",
            "T|28.7|d=z g=x nmos=",
            "T|28.7|d=z g=z nmos=",
            "T|28.8|d=0 en=0 tran=0/St0 tif1=z/HiZ tif0=0/St0 rtran=",
            "T|28.8|d=0 en=1 tran=0/St0 tif1=0/St0 tif0=z/HiZ rtran=",
            "T|28.8|d=0 en=x tran=0/St0 tif1=x/St",
            "T|28.8|d=0 en=z tran=0/St0 tif1=x/St",
            "T|28.8|d=1 en=0 tran=1/St1 tif1=z/HiZ tif0=1/St1 rtran=",
            "T|28.8|d=1 en=1 tran=1/St1 tif1=1/St1 tif0=z/HiZ rtran=",
            "T|28.8|d=1 en=x tran=1/St1 tif1=x/St",
            "T|28.8|d=1 en=z tran=1/St1 tif1=x/St",
            "T|28.8|d=x en=0 tran=x/StX tif1=z/HiZ tif0=x/StX rtran=",
            "T|28.8|d=x en=1 tran=x/StX tif1=x/StX tif0=z/HiZ rtran=",
            "T|28.8|d=x en=x tran=x/StX tif1=x/StX tif0=x/StX rtran=",
            "T|28.8|d=x en=z tran=x/StX tif1=x/StX tif0=x/StX rtran=",
        ],
    );
}

// §28.4, §28.15: and/or/xor/buf/not/bufif/notif truth tables
#[test]
fn c28_gate_truth_tables() {
    // Known gap (§28.5): `28.5`: 6 reference lines differ from xezim's 6, e.g. reference `d=0 ctl=x bif1=x/StL bif0=x/StL nif1=x/StH nif0=x/StH`, xezim `d=0 ctl=x bif1=x/StX bif0=x/StX nif1=x/StX nif0=x/StX`
    check(
        "c28_gates",
        include_str!("sv/c28_gates.sv"),
        "c28_gates",
        Order::Exact,
        &[
            "T|28.4|a=0 b=0 and=0 nand=1 or=0 nor=1 xor=0 xnor=1 and3=0 buf=00 not=1",
            "T|28.4|a=0 b=1 and=0 nand=1 or=1 nor=0 xor=1 xnor=0 and3=0 buf=00 not=1",
            "T|28.4|a=0 b=x and=0 nand=1 or=x nor=x xor=x xnor=x and3=0 buf=00 not=1",
            "T|28.4|a=0 b=z and=0 nand=1 or=x nor=x xor=x xnor=x and3=0 buf=00 not=1",
            "T|28.4|a=1 b=0 and=0 nand=1 or=1 nor=0 xor=1 xnor=0 and3=0 buf=11 not=0",
            "T|28.4|a=1 b=1 and=1 nand=0 or=1 nor=0 xor=0 xnor=1 and3=1 buf=11 not=0",
            "T|28.4|a=1 b=x and=x nand=x or=1 nor=0 xor=x xnor=x and3=x buf=11 not=0",
            "T|28.4|a=1 b=z and=x nand=x or=1 nor=0 xor=x xnor=x and3=x buf=11 not=0",
            "T|28.4|a=x b=0 and=0 nand=1 or=x nor=x xor=x xnor=x and3=0 buf=xx not=x",
            "T|28.4|a=x b=1 and=x nand=x or=1 nor=0 xor=x xnor=x and3=x buf=xx not=x",
            "T|28.4|a=x b=x and=x nand=x or=x nor=x xor=x xnor=x and3=x buf=xx not=x",
            "T|28.4|a=x b=z and=x nand=x or=x nor=x xor=x xnor=x and3=x buf=xx not=x",
            "T|28.4|a=z b=0 and=0 nand=1 or=x nor=x xor=x xnor=x and3=0 buf=xx not=x",
            "T|28.4|a=z b=1 and=x nand=x or=1 nor=0 xor=x xnor=x and3=x buf=xx not=x",
            "T|28.4|a=z b=x and=x nand=x or=x nor=x xor=x xnor=x and3=x buf=xx not=x",
            "T|28.4|a=z b=z and=x nand=x or=x nor=x xor=x xnor=x and3=x buf=xx not=x",
            "T|28.5|d=0 ctl=0 bif1=z/HiZ bif0=0/St0 nif1=z/HiZ nif0=1/St1",
            "T|28.5|d=0 ctl=1 bif1=0/St0 bif0=z/HiZ nif1=1/St1 nif0=z/HiZ",
            "T|28.5|d=1 ctl=0 bif1=z/HiZ bif0=1/St1 nif1=z/HiZ nif0=0/St0",
            "T|28.5|d=1 ctl=1 bif1=1/St1 bif0=z/HiZ nif1=0/St0 nif0=z/HiZ",
            "T|28.5|d=x ctl=0 bif1=z/HiZ bif0=x/StX nif1=z/HiZ nif0=x/StX",
            "T|28.5|d=x ctl=1 bif1=x/StX bif0=z/HiZ nif1=x/StX nif0=z/HiZ",
            "T|28.5|d=x ctl=x bif1=x/StX bif0=x/StX nif1=x/StX nif0=x/StX",
            "T|28.5|d=x ctl=z bif1=x/StX bif0=x/StX nif1=x/StX nif0=x/StX",
            "T|28.5|d=z ctl=x bif1=x/StX bif0=x/StX nif1=x/StX nif0=x/StX",
            "T|28.5|d=z ctl=z bif1=x/StX bif0=x/StX nif1=x/StX nif0=x/StX",
        ],
        &[
            "T|28.5|d=0 ctl=x bif1=x/St",
            "T|28.5|d=0 ctl=z bif1=x/St",
            "T|28.5|d=1 ctl=x bif1=x/St",
            "T|28.5|d=1 ctl=z bif1=x/St",
            "T|28.5|d=z ctl=0 bif1=z/HiZ bif0=",
            "T|28.5|d=z ctl=1 bif1=",
        ],
    );
}
