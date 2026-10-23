/// An indexed/part-select STORE into an element of an unpacked-array class
/// property whose element type is a multi-dimensional packed array
/// (`bit [8:0][7:0] arr[3]`) was silently dropped (bit form:
/// `d.arr[1][2] = 1'b1` left the element untouched) or spliced at raw bit
/// offsets (range form: `d.arr[1][8:0] = 9'h1FF` wrote bits [8:0] instead
/// of the nine 8-bit elements, IEEE 1800-2017 §7.4.1/§7.4.2). The element
/// lives whole under the per-instance store `<h>#arr[1]`; the write now
/// mirrors the read-side walk (`class_packed_operand`), splices the fitted
/// value into the addressed element(s), and stores it back.
///
/// Covers bare-member and `this.`-qualified stores in methods, module
/// scope, element/bit range, `+:`/`-:`, an x/z select (§11.5.1: selects
/// nothing) and an out-of-range select.
///
/// Expected output verified byte-for-byte against the reference simulator.
use xezim::simulate;

fn t_lines(src: &str) -> Vec<String> {
    simulate(src, 100)
        .expect("simulate failed")
        .output
        .iter()
        .map(|o| o.message.clone())
        .filter(|m| m.starts_with("T|"))
        .collect()
}

#[test]
fn class_packed_elem_store() {
    let src = include_str!("class_packed_elem_store.sv");
    assert_eq!(
        t_lines(src),
        [
            "T| w1  000000000102010405",
            "T| w1b 010000000102010405",
            "T| w2  0000000000000001ff",
            "T| w2b 0000000000020001ff",
            "T| w3  00000000bcde0001ff",
            "T| w4  00c0ffeebcde0001ff",
            "T| tw  000000011122334455",
            "T| xz  0000000003020100aa",
            "T| oob 0000000003020100aa",
            "T| elem 0000000003027e00aa",
            "T| hot 000000000002010405 0000000000000001f0",
        ]
    );
}
