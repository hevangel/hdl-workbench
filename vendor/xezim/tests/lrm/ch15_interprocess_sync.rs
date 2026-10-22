//! IEEE 1800-2023 clause 15: interprocess synchronization.

use crate::harness::{Order, check};

// §15.3, §15.4, §15.5.1, §15.5.2, §15.5.3, §15.5.5: semaphores, mailboxes, named events, triggered
#[test]
fn c15_interprocess_sync() {
    // Known gap (§15.4): `15.4k` reference `-1`, xezim `1`
    check(
        "c15_sync",
        include_str!("sv/c15_sync.sv"),
        "c15",
        Order::Exact,
        &[
            "T|15.3a|try=0",
            "T|15.3b|try=1",
            r#"T|15.3c|'{"put2@5", "got2@5"}"#,
            "T|15.3d|try=1",
            r#"T|15.3e|'{"w3", "w1"}"#,
            "T|15.4a|num=2",
            "T|15.4b|1 str",
            "T|15.4c|tryput=0 num=2",
            "T|15.4d|peek=1",
            "T|15.4e|v=10 num=2",
            "T|15.4f|10",
            "T|15.4g|20",
            "T|15.4h|tryget=0",
            r#"T|15.4i|'{"g77@11"}"#,
            r#"T|15.4j|'{"got1", "put3@15"}"#,
            "T|15.4l|x 1",
            "T|15.5.3a|triggered seen t=15",
            "T|15.5.3b|triggered next step=0",
            "T|15.5.5a|merged e3 woke via e2 t=17",
            "T|15.5.5b|null never fires",
            "T|15.5.1|nb trigger seen t=21",
        ],
        &["T|15.4k|"],
    );
}
