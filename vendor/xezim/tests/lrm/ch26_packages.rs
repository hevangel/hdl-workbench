//! IEEE 1800-2023 clause 26: packages.

use crate::harness::{Order, check};

// §26.3: package var local shadow
#[test]
fn c26_3_package_var_local_shadow() {
    check(
        "26.3_package_var_local_shadow",
        include_str!("sv/26.3_package_var_local_shadow.sv"),
        "rpl",
        Order::Exact,
        &["T|r1|local=99 pkg=1"],
        &[],
    );
}

// §26.3: Variables with the same name in two packages, or in a package and an importing module
#[test]
fn c26_3_package_var_name_aliasing() {
    check(
        "26.3_package_var_name_aliasing",
        include_str!("sv/26.3_package_var_name_aliasing.sv"),
        "rpv",
        Order::Exact,
        &["T|r1|pa=1 pc=50", "T|r2|pa=2 pc=50", "T|r3|pa=2 pc=7"],
        &[],
    );
}

// §26.4: package import rules
#[test]
fn c26_4_package_import_rules() {
    check(
        "c26_more",
        include_str!("sv/c26_more.sv"),
        "c26x",
        Order::Exact,
        &["T|26.3e|99 1", "T|26.4|10 3", "T|26.3f|32"],
        &[],
    );
}

// §26.3 (issue #425): an unpacked-array package localparam used through its
// package-qualified name outside the package — `pkg::LIM[3]` as an unpacked
// dimension const-folds to the element value, and a scoped scalar localparam
// still sizes a dimension (no sticky diagnostics from the speculative
// associative-index data-type parse of either spelling).
#[test]
fn c26_3_pkg_array_param_element_dim() {
    check(
        "26.3_pkg_array_param_element_dim",
        include_str!("sv/26.3_pkg_array_param_element_dim.sv"),
        "c26pad",
        Order::Exact,
        &[
            "T|26.3g|size(ctl)=8",
            "T|26.3g|size(mem)=8",
            "T|26.3g|mem[7]=5 elem=8",
        ],
        &[],
    );
}
