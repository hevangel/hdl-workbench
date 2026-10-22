//! §26.6: `export P::*` / `export *::*` re-export every declaration the
//! package actually imported from `P`, and an explicit `import P::x;`
//! imports `x` whether or not the package body references it (§26.3) — only
//! a wildcard import waits for a reference. The re-export resolver required
//! a reference in every case, so `import P2::x;` of a name P2 imported
//! explicitly and exported by wildcard failed with "Symbol 'x' not found in
//! package 'P2'" (ivtest `sv_export2`, `sv_export3`; the reference simulator
//! accepts both and prints PASSED).

fn run(export: &str) -> Vec<String> {
    let src = format!(
        "package P1; integer x = 123; endpackage
package P2; import P1::x; {export} endpackage
module test;
  import P2::x;
  initial $display(\"E|%0d\", x);
endmodule
"
    );
    xezim::simulate(&src, 10)
        .expect("simulate")
        .output
        .iter()
        .map(|o| o.message.clone())
        .collect()
}

#[test]
fn wildcard_export_of_source_package_reexports_explicit_import() {
    let o = run("export P1::*;");
    assert!(o.iter().any(|l| l == "E|123"), "{o:?}");
}

#[test]
fn all_wildcard_export_reexports_explicit_import() {
    let o = run("export *::*;");
    assert!(o.iter().any(|l| l == "E|123"), "{o:?}");
}

/// §26.6: "declarations imported into a package are not visible by way of
/// subsequent imports of that package" unless exported. Pinned to the LRM:
/// the reference simulator is lenient here and resolves the name.
#[test]
fn unexported_name_is_still_not_visible() {
    let src = "package P1; integer x = 123; endpackage
package P2; import P1::x; endpackage
module test;
  import P2::x;
  initial $display(\"E|%0d\", x);
endmodule
";
    assert!(xezim::simulate(src, 10).is_err());
}
