use super::*;

fn ident(name: &str) -> Expression {
    let span = crate::ast::Span::dummy();
    Simulator::hier_ident_expr(Simulator::iface_path_segs(name, span), span)
}

#[test]
fn event_name_classification_preserves_prefixes_live_aliases_and_owned_keys() {
    let mut sim = crate::simulate(
        "module tb; event pulse, other_pulse; int data; endmodule",
        0,
    )
    .expect("event classification fixture");
    assert_eq!(sim.event_ref_key_static(&ident("data")), None);
    assert_eq!(
        sim.event_ref_key_static(&ident("pulse")),
        Some("pulse".into())
    );
    assert_eq!(
        sim.event_ref_key_static(&ident("tb.pulse")),
        Some("pulse".into())
    );
    assert_eq!(sim.event_ref_key_static(&ident("tb_extra.pulse")), None);
    sim.event_aliases
        .insert("runtime_alias".into(), "pulse".into());
    let captured = sim
        .event_ref_key_static(&ident("runtime_alias"))
        .expect("canonical event key");
    sim.event_aliases
        .insert("runtime_alias".into(), "other_pulse".into());
    assert_eq!(captured, "pulse", "returned keys are independent snapshots");
    assert_eq!(
        sim.event_ref_key_static(&ident("runtime_alias")),
        Some("other_pulse".into())
    );
    sim.event_aliases
        .insert("runtime_alias".into(), Simulator::EVENT_NULL_KEY.into());
    assert_eq!(sim.event_ref_key_static(&ident("runtime_alias")), None);
    assert_eq!(sim.event_ref_key_static(&ident("created_later")), None);
    sim.module.events.insert("created_later".into());
    assert_eq!(
        sim.event_ref_key_static(&ident("created_later")),
        Some("created_later".into())
    );
}
