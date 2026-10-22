//! Keep engine policy tests independent of process-wide cached settings.

use std::process::Command;

pub(super) fn eager() -> bool {
    policies(&[("1", "0")])
}

pub(super) fn policies(settings: &[(&str, &str)]) -> bool {
    let thread = std::thread::current();
    let name = thread.name().expect("named test thread");
    if std::env::var("XEZIM_TEST_METHOD_CHILD").as_deref() == Ok(name) {
        return true;
    }
    for &(enabled, tier) in settings {
        let output = Command::new(std::env::current_exe().expect("test binary"))
            .args(["--exact", name, "--nocapture", "--test-threads=1"])
            .env("XEZIM_TEST_METHOD_CHILD", name)
            .env("XEZIM_COMPILE_METHODS", enabled)
            .env("XEZIM_METHOD_TIER", tier)
            .env_remove("XEZIM_METHOD_CACHE")
            .output()
            .expect("run isolated policy test");
        assert!(
            output.status.success(),
            "{name}: enabled={enabled}, tier={tier}\n{}\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(
            String::from_utf8_lossy(&output.stdout).contains("1 passed"),
            "isolated test filter did not run {name}"
        );
    }
    false
}
