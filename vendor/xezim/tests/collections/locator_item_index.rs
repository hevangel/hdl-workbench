//! §7.12.4: in a locator's `with` clause, `item.index` (or
//! `<iterator>.index`) is the current element's index (sv-tests
//! `unpacked-array-iterator-index-querying`). The locator path bound `item`
//! but never `item.index`, which then read 0, so `arr.find with (item ==
//! item.index)` matched only a zero at index 0. Cross-checked against the
//! reference simulator.

#[test]
fn locator_with_clause_reads_item_index() {
    let sim = xezim::simulate(
        r#"
module top;
  int arr[] = { 0, 1, 3, 3 };
  int fa[4] = '{5, 6, 7, 8};
  int q[$];
  initial begin
    q = arr.find with ( item == item.index );
    $display("L|%p", q);
    q = fa.find_index with ( item.index >= 2 );
    $display("L|%p", q);
    q = fa.find(x) with ( x.index == 1 );
    $display("L|%p", q);
    q = fa.find_first with ( item == 2 * item.index + 3 );
    $display("L|%p", q);
  end
endmodule
"#,
        10,
    )
    .expect("simulate");
    let o: Vec<String> = sim.output.iter().map(|o| o.message.clone()).collect();
    let l: Vec<&String> = o.iter().filter(|l| l.starts_with("L|")).collect();
    assert_eq!(
        l,
        vec!["L|'{0, 1, 3}", "L|'{2, 3}", "L|'{6}", "L|'{7}"],
        "{o:?}"
    );
}
