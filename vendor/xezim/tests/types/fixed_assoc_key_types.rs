//! §7.8.4 / §12.7.3: every fixed-array row retains its associative key type.
use xezim::simulate;

fn output(src: &str) -> String {
    simulate(src, 100)
        .expect("simulate")
        .output
        .iter()
        .map(|line| line.message.as_str())
        .collect::<Vec<_>>()
        .join("\n")
}

#[test]
fn declared_int_key_applies_to_module_and_class_rows() {
    assert_eq!(
        output(
            r#"module top;
  int b [3][int];                      // each b[i] is an associative array, signed int key
  class c; int b [3][int]; endclass
  int errors = 0;
  task check(string what, int got, int exp);
    if (got != exp) begin errors++; $display("FAIL %s: got %0d, expected %0d", what, got, exp); end
  endtask
  initial begin
    c h = new; bit [31:0] u; int s;
    u = 32'hffff_fffd;                 // same bits as -3
    b[2][-3] = 30;   h.b[2][-3] = 30;
    check("b[2].exists(u)",   b[2].exists(u),   1);
    check("h.b[2].exists(u)", h.b[2].exists(u), 1);
    check("b[2][u]",          b[2][u],          30);
    b[2][u] = 31;                      // must overwrite the same element
    check("b[2].num()",       b[2].num(),       1);
    s = 0; foreach (b[2][k]) s += k + b[2][k];
    check("foreach b[2] key+val",   s, 28);
    s = 0; foreach (h.b[2][k]) s += k + h.b[2][k];
    check("foreach h.b[2] key+val", s, 27);
    if (errors == 0) $display("PASS"); else $display("FAILED: %0d check(s)", errors);
  end
endmodule
"#
        ),
        "PASS"
    );
}

#[test]
fn submodule_generate_and_multidimensional_rows_keep_signed_keys() {
    let src = r#"
module child;
  int rows[1:2][byte];
  initial begin
    int k, sum, found; bit [31:0] u=32'hffff_fffd;
    rows[2][-3]=30; rows[2][u]=31;
    sum=0; foreach(rows[2][i]) sum+=i+rows[2][i];
    k=0; found=rows[2].first(k);
    $display("T|child=%0d,%0d,%0d,%0d",rows[2].num(),rows[2].exists(u),k,sum);
  end
endmodule
module top;
  child u();
  if (1) begin : generated
    int rows[2][shortint];
    int matrix[2][2][int];
    initial begin
      bit [31:0] u=32'hffff_fffd; int sum=0;
      rows[1][-3]=9; rows[1][u]=10;
      matrix[1][0][-3]=20; matrix[1][0][u]=21;
      foreach(rows[1][i]) sum+=i+rows[1][i];
      $display("T|generate=%0d,%0d,%0d,%0d",rows[1].num(),sum,matrix[1][0].num(),matrix[1][0][-3]);
    end
  end
endmodule
"#;
    let out = output(src);
    assert!(out.contains("T|child=1,1,-3,28"), "{out}");
    assert!(out.contains("T|generate=1,7,1,21"), "{out}");
}

#[test]
fn row_key_aliases_narrow_before_lookup_and_iteration() {
    let src = r#"
typedef logic signed [7:0] signed_key_t;
module top;
  int signed_rows[2][signed_key_t];
  int unsigned_rows[2][bit [7:0]];
  initial begin
    int a=0,b=0; bit [31:0] u=32'hffff_fffd;
    signed_rows[1][-3]=30; signed_rows[1][u]=31;
    unsigned_rows[1][-3]=40; unsigned_rows[1][u]=41;
    foreach(signed_rows[1][k]) a+=k+signed_rows[1][k];
    foreach(unsigned_rows[1][k]) b+=k+unsigned_rows[1][k];
    $display("T|alias=%0d,%0d unsigned=%0d,%0d",signed_rows[1].num(),a,unsigned_rows[1].num(),b);
  end
endmodule
"#;
    assert_eq!(output(src), "T|alias=1,28 unsigned=1,294");
}

#[test]
fn inherited_class_rows_keep_integral_key_width_and_sign() {
    let src = r#"
class base;
  int signed_rows[2][byte];
  int unsigned_rows[2][bit [7:0]];
endclass
class derived extends base; endclass
module top;
  initial begin
    derived h=new; int a=0,b=0; bit [31:0] u=32'hffff_fffd;
    h.signed_rows[1][-3]=30; h.signed_rows[1][u]=31;
    h.unsigned_rows[1][-3]=40; h.unsigned_rows[1][u]=41;
    foreach(h.signed_rows[1][k]) a+=k+h.signed_rows[1][k];
    foreach(h.unsigned_rows[1][k]) b+=k+h.unsigned_rows[1][k];
    $display("T|inherited=%0d,%0d unsigned=%0d,%0d",h.signed_rows[1].num(),a,h.unsigned_rows[1].num(),b);
  end
endmodule
"#;
    assert_eq!(output(src), "T|inherited=1,28 unsigned=1,294");
}

#[test]
fn automatic_task_rows_retain_key_type_on_reentry() {
    assert_eq!(
        output(
            r#"module top;
 task automatic show();
   int b[2][byte]; int sum=0; bit [31:0] u=32'hffff_fffd;
   b[1][-3]=30; b[1][u]=31;
   foreach(b[1][k]) sum+=k+b[1][k];
   $display("T|local=%0d,%0d",b[1].num(),sum);
 endtask
 initial begin show(); show(); end
endmodule
"#
        ),
        "T|local=1,28\nT|local=1,28"
    );
}

#[test]
fn traversal_methods_on_class_and_module_rows_keep_signed_keys() {
    let src = r#"
class holder; int rows[2][byte]; endclass
module top;
  int rows[2][byte];
  initial begin
    holder h=new; int k, first_key,last_key,next_key,prev_key,ok;
    rows[1][-3]=30; rows[1][2]=20;
    ok=rows[1].first(k); first_key=k;
    ok=rows[1].next(k); next_key=k;
    ok=rows[1].last(k); last_key=k;
    ok=rows[1].prev(k); prev_key=k;
    $display("T|module-walk=%0d,%0d,%0d,%0d",first_key,next_key,last_key,prev_key);
    h.rows[1][-3]=30; h.rows[1][2]=20;
    ok=h.rows[1].first(k); first_key=k;
    ok=h.rows[1].next(k); next_key=k;
    ok=h.rows[1].last(k); last_key=k;
    ok=h.rows[1].prev(k); prev_key=k;
    $display("T|class-walk=%0d,%0d,%0d,%0d",first_key,next_key,last_key,prev_key);
  end
endmodule
"#;
    let out = output(src);
    assert!(out.contains("T|module-walk=-3,2,2,-3"), "{out}");
    assert!(out.contains("T|class-walk=-3,2,2,-3"), "{out}");
}
