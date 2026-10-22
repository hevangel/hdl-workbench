//! Processes parked on NBA-region completion (`nba <= next; @(nba)`, as in
//! `uvm_wait_for_nba_region`) resume only after every Active and Inactive
//! event of the time slot has run and the NBA region has committed
//! (IEEE 1800-2023 §4.5). Expected lines are the reference simulator's;
//! they are compared as a multiset because the order in which two waiters
//! woken by the same update run is not specified (§4.7).

fn lines(src: &str) -> Vec<String> {
    let sim = xezim::simulate(src, 1000).expect("simulation must finish");
    let mut v: Vec<String> = sim
        .output
        .iter()
        .map(|l| l.message.clone())
        .filter(|m| m.starts_with("T|"))
        .collect();
    v.sort();
    v
}

#[test]
fn n1_nba_commit() {
    let mut want: Vec<String> = vec!["T|child x=7 t=5".into(), "T|parent x=7 t=5".into()];
    want.sort();
    assert_eq!(lines(include_str!("nba_waiters/n1_nba_commit.sv")), want);
}

#[test]
fn n4_edge_parent() {
    let mut want: Vec<String> = vec!["T|parent total=1 t=15".into()];
    want.sort();
    assert_eq!(lines(include_str!("nba_waiters/n4_edge_parent.sv")), want);
}

#[test]
fn n5_two_waiters() {
    let mut want: Vec<String> = vec!["T|A sees a=1 b=1".into(), "T|B sees a=1 b=1".into()];
    want.sort();
    assert_eq!(lines(include_str!("nba_waiters/n5_two_waiters.sv")), want);
}

#[test]
fn n7_cond_nba() {
    let mut want: Vec<String> = vec!["T|hopper got=1 t=5".into(), "T|waiter t=5".into()];
    want.sort();
    assert_eq!(lines(include_str!("nba_waiters/n7_cond_nba.sv")), want);
}

#[test]
fn n8_static_hop_child() {
    let mut want: Vec<String> = vec!["T|parent total=1".into()];
    want.sort();
    assert_eq!(
        lines(include_str!("nba_waiters/n8_static_hop_child.sv")),
        want
    );
}

#[test]
fn n9_static_hop_hash0() {
    let mut want: Vec<String> = vec!["T|parent total=1".into()];
    want.sort();
    assert_eq!(
        lines(include_str!("nba_waiters/n9_static_hop_hash0.sv")),
        want
    );
}

#[test]
fn n10_static_hop_childnba() {
    let mut want: Vec<String> = vec!["T|parent total=1".into()];
    want.sort();
    assert_eq!(
        lines(include_str!("nba_waiters/n10_static_hop_childnba.sv")),
        want
    );
}

#[test]
fn n11_edge_hash0() {
    let mut want: Vec<String> = vec!["T|parent total=1 t=5".into()];
    want.sort();
    assert_eq!(lines(include_str!("nba_waiters/n11_edge_hash0.sv")), want);
}
