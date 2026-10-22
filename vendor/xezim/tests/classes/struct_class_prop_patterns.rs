//! §10.9.2/§8.4: an assignment pattern written to an unpacked-struct CLASS
//! PROPERTY (`a = '{x: 1, y: 2}` in a method, `c.a = ...`, `'{default: v}`,
//! a nested struct member) reached module-scope cells nothing reads, so every
//! member kept its old value. Each member is now assigned through the
//! property's own member path. Found while probing the struct members of
//! #310 (reported by Ganesh-TNL). Expected values come from the reference
//! simulator.

use xezim::simulate;

fn t_lines(src: &str) -> Vec<String> {
    simulate(src, 1000)
        .expect("simulate failed")
        .output
        .iter()
        .map(|o| o.message.clone())
        .filter(|m| m.starts_with("T|"))
        .collect()
}

#[test]
fn patterns_reach_struct_class_properties() {
    let expected = [
        "T|ctor a=1,2 n=3,4,5 arr1=6,7",
        "T|ext a=10,11 n=1,2,3",
        "T|dflt a=9,9 n=8,8,8",
        "T|p '{z:8, in:'{x:8, y:8}}",
    ];
    assert_eq!(
        t_lines(include_str!("struct_class_prop_patterns.sv")),
        expected
    );
}
