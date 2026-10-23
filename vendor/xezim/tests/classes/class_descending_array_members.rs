//! §7.4.2 / §10.9 / §20.7: a class property declared with a descending
//! unpacked dimension (`int ds[3:1]`) keeps its declared direction, as a
//! module array does: an assignment pattern's first item lands on the left
//! bound, `%p` and `foreach` go left to right, and `$left`/`$right`/
//! `$increment` report the declared bounds. A query of a selected row
//! (`$left(m[0])`) describes the dimensions left after the select. Expected
//! values are the reference simulator's.

use xezim::simulate;

fn t_lines(src: &str) -> Vec<String> {
    let sim = simulate(src, 100).expect("simulate failed");
    sim.output
        .iter()
        .map(|o| o.message.trim_end().to_string())
        .filter(|m| m.starts_with("T|"))
        .collect()
}

const SRC: &str = r#"class C;
  int a[4]; int m[2][3]; int ds[3:1]; int dm[1:0][2:4];
  function void run();
    ds = '{1, 2, 3};
    dm = '{'{1,2,3},'{4,5,6}};
    $display("T|c ds3=%0d ds1=%0d dm12=%0d dm14=%0d dm02=%0d p=%p q=%p", ds[3], ds[1], dm[1][2], dm[1][4], dm[0][2], ds, dm);
    $display("T|c a=%0d:%0d m0=%0d:%0d ds=%0d:%0d dm=%0d:%0d dm1=%0d:%0d inc=%0d incdm1=%0d hidm1=%0d", $left(a), $right(a), $left(m[0]), $right(m[0]), $left(ds), $right(ds), $left(dm), $right(dm), $left(dm[1]), $right(dm[1]), $increment(ds), $increment(dm[1]), $high(dm[1]));
    ds = '{default: 9};
    ds[3] = 7;
    $display("T|c d ds=%p", ds);
    foreach (ds[i]) $display("T|c fe i=%0d v=%0d", i, ds[i]);
    foreach (dm[i, j]) if (i == 1) $display("T|c fe2 %0d %0d %0d", i, j, dm[i][j]);
  endfunction
endclass
module top;
  int ma[4]; int mm[2][3]; int mdm[1:0][2:4];
  initial begin
    C c;
    $display("T|m mm0=%0d:%0d mdm1=%0d:%0d lo=%0d inc=%0d sz=%0d", $left(mm[0]), $right(mm[0]), $left(mdm[1]), $right(mdm[1]), $low(mdm[1]), $increment(mdm[1]), $size(mm[1]));
    c = new; c.run();
    c.ds[2] = 8;
    $display("T|o %p %0d %0d:%0d", c.ds, c.ds[2], $left(c.ds), $right(c.ds));
    foreach (c.ds[i]) $display("T|o fe i=%0d", i);
  end
endmodule
"#;

#[test]
fn class_descending_array_members() {
    let want = [
        "T|m mm0=0:2 mdm1=2:4 lo=2 inc=-1 sz=3",
        "T|c ds3=1 ds1=3 dm12=1 dm14=3 dm02=4 p='{1, 2, 3} q='{'{1, 2, 3}, '{4, 5, 6}}",
        "T|c a=0:3 m0=0:2 ds=3:1 dm=1:0 dm1=2:4 inc=1 incdm1=-1 hidm1=4",
        "T|c d ds='{7, 9, 9}",
        "T|c fe i=3 v=7",
        "T|c fe i=2 v=9",
        "T|c fe i=1 v=9",
        "T|c fe2 1 2 1",
        "T|c fe2 1 3 2",
        "T|c fe2 1 4 3",
        "T|o '{7, 8, 9} 8 3:1",
        "T|o fe i=3",
        "T|o fe i=2",
        "T|o fe i=1",
    ];
    assert_eq!(t_lines(SRC), want);
}
