//! Hierarchical interface-port aliases survive elaboration (§25.3 / §23.8).
use xezim_core::elaborate::interface_ports;

#[test]
fn interface_ports_without_bodies_retain_their_connections() {
    let source = r#"
interface bus_if; logic [8:0] state; modport slv(output state); endinterface
module regs(bus_if.slv bus); endmodule
module bridge(interface bus); regs u_regs(bus); endmodule
module top;
 bus_if b[2]();
 for(genvar g=0;g<2;g++) begin: bank
   bridge u_bridge(b[g].slv);
 end
endmodule
"#;
    let (_, design) = xezim_core::parse_and_elaborate_multi(
        &[source.to_string()],
        Some("top"),
        &[],
        &["top.sv".to_string()],
        &[],
    )
    .expect("elaborate");
    for i in 0..2 {
        let formal = format!("bank[{i}].u_bridge.u_regs.bus");
        let name = format!("{formal}.state");
        assert!(design.interface_port_aliases.contains_key(&formal));
        assert_eq!(
            interface_ports::resolve(&name, &design.interface_port_aliases, "top"),
            format!("b[{i}].state")
        );
    }
}

#[test]
fn interface_port_aliases_keep_element_bindings_and_name_boundaries() {
    let mut aliases = xezim_core::hasher::HashMap::default();
    aliases.insert("u.bus".into(), "physical".into());
    aliases.insert("u.bus[0]".into(), "physical[3]".into());
    aliases.insert("v.port".into(), "u.bus[0]".into());
    for (name, expected) in [
        ("u.bus.state", "physical.state"),
        ("top.u.bus[0].state", "physical[3].state"),
        ("v.port.words[2]", "physical[3].words[2]"),
        ("u.buss.state", "u.buss.state"),
        ("u.busy", "u.busy"),
    ] {
        assert_eq!(interface_ports::resolve(name, &aliases, "top"), expected);
    }
    aliases.insert("physical".into(), "u.bus".into());
    assert!(!interface_ports::resolve("u.bus.state", &aliases, "top").is_empty());
}
