//! Issue #435: IEEE 1800 §6.21 / §25.3 static interface initialization.
use xezim::simulate_multi;

fn check(source: &str, top: &str, expected: &[&str]) {
    let sim = simulate_multi(
        &[source.to_string()],
        10,
        Some(top),
        &[],
        &[],
        None,
        false,
        None,
        None,
        &[],
        &[],
        None,
        &[],
        0,
        u64::MAX,
        None,
        &[],
        None,
        None,
        None,
        None,
        false,
    )
    .expect("simulate interface static initialization");
    let actual: Vec<_> = sim
        .output
        .iter()
        .map(|o| o.message.trim_end())
        .filter(|line| line.starts_with("IFACE_PATH|") || line.starts_with("T|"))
        .collect();
    assert_eq!(actual, expected);
}

#[test]
fn reported_interface_string_initializer() {
    check(
        include_str!("../issue_cases/interface_string_initializer.sv"),
        "interface_string_initializer_probe",
        &[
            "IFACE_PATH|literal=literal|path=interface_string_initializer_probe.bus|method=interface_string_initializer_probe.bus",
            "IFACE_PATH|done",
        ],
    );
}

#[test]
fn per_instance_paths_and_initializers_once() {
    check(
        include_str!("../issue_cases/interface_string_hierarchy.sv"),
        "interface_string_hierarchy_probe",
        &[
            "IFACE_PATH|first=interface_string_hierarchy_probe.first",
            "IFACE_PATH|second=interface_string_hierarchy_probe.second:9",
            "IFACE_PATH|array0=interface_string_hierarchy_probe.arrayed[0]",
            "IFACE_PATH|array1=interface_string_hierarchy_probe.arrayed[1]",
            "IFACE_PATH|nested=interface_string_hierarchy_probe.outer.inner",
            "IFACE_PATH|module=interface_string_hierarchy_probe.holder",
            "IFACE_PATH|module_interface=interface_string_hierarchy_probe.holder.bus:8",
            "IFACE_PATH|generate0=interface_string_hierarchy_probe.nodes[0].bus:10",
            "IFACE_PATH|generate1=interface_string_hierarchy_probe.nodes[1].bus:11",
            "IFACE_PATH|package=init_path_pkg.label",
            "IFACE_PATH|virtual=interface_string_hierarchy_probe.first",
            "IFACE_PATH|initializers_once",
            "IFACE_PATH|done",
        ],
    );
}

#[test]
fn interface_inside_descending_module_array() {
    check(
        include_str!("../issue_cases/interface_initializer_module_array.sv"),
        "top",
        &["T|top.holder[3].bus", "T|top.holder[2].bus"],
    );
}

#[test]
fn bound_interface_initializer() {
    check(
        include_str!("../issue_cases/interface_initializer_bound.sv"),
        "top",
        &["T|top.h.bus"],
    );
}
