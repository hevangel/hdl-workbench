//! `randomize()` shapes next to #301: a constraint through a sub-object of a
//! sub-object (§18.5.9), `rand` unpacked struct members with wide fields
//! (§18.4), `z == x ^ y` over wide dynamic-array elements and
//! `std::randomize` of locals wider than 64 bits (§18.12). Each probe of
//! `randomize_wide_open.sv` prints whether its constraints hold; the
//! reference simulator prints `r=8 ok=8` on every line.

use xezim::simulate;

fn check(prefix: &str) {
    let src = include_str!("randomize_wide_open.sv");
    let got: Vec<String> = simulate(src, 100)
        .expect("simulate failed")
        .output
        .iter()
        .map(|o| o.message.clone())
        .filter(|m| m.starts_with(&format!("T|{prefix}")))
        .collect();
    let want: Vec<String> = src
        .lines()
        .filter_map(|l| {
            let i = l.find(&format!("$display(\"T|{prefix}"))? + "$display(\"".len();
            let name = l[i..].split_whitespace().next()?;
            Some(format!("{name} r=8 ok=8"))
        })
        .collect();
    assert!(!want.is_empty());
    assert_eq!(got, want);
}

#[test]
fn constraint_through_two_levels_of_sub_objects() {
    check("nest3_");
}

#[test]
fn rand_unpacked_struct_with_wide_fields() {
    check("ustruct_");
}

#[test]
fn wide_dynamic_array_elements_through_xor() {
    check("darr_");
}

#[test]
fn std_randomize_of_wide_locals() {
    check("std_");
}
