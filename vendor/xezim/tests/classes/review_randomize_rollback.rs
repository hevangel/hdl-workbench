//! Behavioral regressions for reviewed PR follow-ups (IEEE 1800 §6.21, §11.6, §16.5, §18.12, §20.7).
fn output(src: &str) -> String {
    xezim::simulate(src, 1_000_000)
        .expect("simulation failed")
        .output
        .iter()
        .map(|o| o.message.as_str())
        .collect::<Vec<_>>()
        .join("\n")
}

#[test]
fn early_unsat_preserves_dynamic_elements() {
    let got = output(
        r###"module top;
 int a[]; int rc;
 initial begin
 a=new[2]; a[0]=7; a[1]=8;
 rc=std::randomize(a) with { a.size()==-1; };
 $display("T|rc %0d size %0d a0 %0d a1 %0d",rc,a.size(),a[0],a[1]);
 $finish;
 end
endmodule
"###,
    );
    assert!(got.contains("T|rc 0 size 2 a0 7 a1 8"), "{got}");
}

#[test]
fn failed_resize_restores_dynamic_size() {
    let got = output(
        r###"module top;
 int a[]; int rc;
 initial begin
  a=new[2]; a[0]=7; a[1]=8;
  rc=std::randomize(a) with { a.size()==3; foreach(a[i]) a[i]==10; a.sum()==99; };
  $display("T|rc %0d size %0d a0 %0d a1 %0d",rc,a.size(),a[0],a[1]);
  $finish;
 end
endmodule
"###,
    );
    assert!(got.contains("T|rc 0 size 2 a0 7 a1 8"), "{got}");
}

#[test]
fn failed_same_size_restores_dynamic_elements() {
    let got = output(
        r###"module top;
 int a[]; int rc;
 initial begin
  a=new[2]; a[0]=7; a[1]=8;
  rc=std::randomize(a) with { a.size()==2; foreach(a[i]) a[i]==10; a.sum()==99; };
  $display("T|rc %0d size %0d a0 %0d a1 %0d",rc,a.size(),a[0],a[1]);
  $finish;
 end
endmodule
"###,
    );
    assert!(got.contains("T|rc 0 size 2 a0 7 a1 8"), "{got}");
}

#[test]
fn failed_resize_restores_queue() {
    let got = output(
        r###"module top;
 int a[$]; int rc;
 initial begin
  a.push_back(7); a.push_back(8);
  rc=std::randomize(a) with { a.size()==3; foreach(a[i]) a[i]==10; a.sum()==99; };
  $display("T|rc %0d size %0d a0 %0d a1 %0d",rc,a.size(),a[0],a[1]);
  $finish;
 end
endmodule
"###,
    );
    assert!(got.contains("T|rc 0 size 2 a0 7 a1 8"), "{got}");
}

#[test]
fn failed_resize_preserves_empty_array() {
    let got = output(
        r###"module top;
 int a[]; int rc;
 initial begin
  a=new[0];
  rc=std::randomize(a) with { a.size()==3; foreach(a[i]) a[i]==10; a.sum()==99; };
  $display("T|rc %0d size %0d",rc,a.size());
  $finish;
 end
endmodule
"###,
    );
    assert!(got.contains("T|rc 0 size 0"), "{got}");
}
