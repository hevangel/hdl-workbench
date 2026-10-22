use xezim::simulate;

/// IEEE 1800 §9.4.2 and §15.5: nested/indexed properties and redundant
/// parentheses retain the same value-change or named-event wait semantics.
#[test]
fn property_wait_shapes() {
    let sim = simulate(include_str!("property_wait_shapes.sv"), 20).expect("simulate");
    let lines: Vec<&str> = sim
        .output
        .iter()
        .flat_map(|entry| entry.message.lines())
        .collect();
    assert!(lines.contains(&"T|properties=2,2"), "{lines:?}");
    assert!(lines.contains(&"T|paren=2,2,2,2"), "{lines:?}");
    assert!(lines.contains(&"T|deferred=0,1"), "{lines:?}");
}
