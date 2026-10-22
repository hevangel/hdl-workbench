//! Issue #320 (reported by Ganesh-TNL): a single-file, UVM-free stand-in for
//! a config_db harness. A: a `*` scope glob crossing `.` (already passed).
//! C: a `%m`-initialized path read through a module function called from a
//! class method (§23.6, §20.10) came back empty, so the enum-typed get
//! missed. B: a parameterized virtual-interface type set with concrete
//! parameters and got through the class's type parameters keyed a separate,
//! empty table (§25.9, §8.25). Expected lines come from the reference
//! simulator.

use xezim::simulate;

#[test]
fn issue320_harness_mwe() {
    let expected = [
        "INFO: local call: inst_path=[tb.sub_env.u_h] get_inst_path=[sub_env]",
        "INFO: get_inst_path() from module initial = [sub_env] (expected [sub_env])",
        "INFO: const_path() from module initial = [sub_env] (expected [sub_env])",
        "INFO: module-scope control gets f1=1 f2=1 (both must be 1)",
        "INFO: get_inst_path() from class method = [sub_env] (expected [sub_env])",
        "INFO: const_path() from class method, via variable = [sub_env] (expected [sub_env])",
        "INFO: const_path() from class method, inline = [sub_env] (expected [sub_env])",
        "[OK] C: enum ipw61_env_cfg get hit, cfg=ACTIVE_CFG",
        "[OK] A: int get hit, x=7",
        "[OK] B: parameterized-vif get hit, data=42",
        "TEST PASSED",
    ];
    let got: Vec<String> = simulate(include_str!("issue320_harness_mwe.sv"), 1000)
        .expect("simulate failed")
        .output
        .iter()
        .map(|o| o.message.clone())
        .filter(|m| m.starts_with("INFO") || m.starts_with('[') || m.starts_with("TEST"))
        .collect();
    assert_eq!(got, expected);
}
