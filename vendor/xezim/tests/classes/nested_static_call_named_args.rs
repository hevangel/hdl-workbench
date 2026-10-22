//! A static call whose class specialization nests another specialization
//! (`db#(wrap#(cfg#(AW,DW)))::get()`) or uses named parameter assignments
//! (`db#(cfg#(.AW(AW),.DW(DW)))`, `db#(.T(...))`) resolves the enclosing
//! class's value parameters at every level, so the getter reaches the same
//! specialization the setter wrote (IEEE 1800-2023 §8.25). Expected lines are
//! the reference simulator's.

fn lines(src: &str) -> Vec<String> {
    let sim = xezim::simulate(src, 100).expect("simulation must finish");
    sim.output
        .iter()
        .map(|l| l.message.clone())
        .filter(|m| m.starts_with("T|"))
        .collect()
}

fn expect(tag: &str) -> Vec<String> {
    [
        format!("T|{} 12 32 got=set", tag),
        format!("T|{} 49 128 got=null", tag),
        format!("T|{} 12 32 got=set", tag),
        format!("T|{} 49 128 got=set", tag),
    ]
    .to_vec()
}

#[test]
fn deeper_nesting() {
    assert_eq!(
        lines(include_str!("static_call_spec_p1_deep.sv")),
        expect("p1_deep")
    );
}

#[test]
fn named_inner_arguments() {
    assert_eq!(
        lines(include_str!("static_call_spec_p2_named_inner.sv")),
        expect("p2_named_inner")
    );
}

#[test]
fn named_outer_argument() {
    assert_eq!(
        lines(include_str!("static_call_spec_p3_named_outer.sv")),
        expect("p3_named_outer")
    );
}

#[test]
fn named_inner_and_outer_arguments() {
    assert_eq!(
        lines(include_str!("static_call_spec_p4_named_both.sv")),
        expect("p4_named_both")
    );
}

#[test]
fn partially_named_arguments() {
    assert_eq!(
        lines(include_str!("static_call_spec_p7_named_partial.sv")),
        expect("p7_named_partial")
    );
}

#[test]
fn defaulted_trailing_argument() {
    assert_eq!(
        lines(include_str!("static_call_spec_p8_default_tail.sv")),
        expect("p8_default_tail")
    );
}
