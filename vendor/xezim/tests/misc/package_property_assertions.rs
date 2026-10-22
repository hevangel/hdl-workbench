//! §26.2 + §16.6: a named property declared in a PACKAGE (with formal
//! ports) and instantiated by an importing module's concurrent assertion.
//! Two stacked defects: the package-item parser had no `property` arm (the
//! whole package failed to parse), and an `assert property (p(actuals))`
//! INSTANTIATION was never resolved to the property body — the assert was
//! silently dropped (never passed, never failed), which reads as
//! always-true. Formals now substitute positionally, including a formal
//! used as the CLOCK. Expected counts reference-verified (7 passes /
//! 3 failures; the sampled values are preponed, so the action block's
//! $display of current values shows the post-edge stimulus).

use std::process::Command;

fn run(name: &str, src: &str) -> String {
    let dir = std::env::temp_dir().join(format!("xezim_ppa_{}_{}", name, std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join(format!("{name}.sv"));
    std::fs::write(&path, src).unwrap();
    let out = Command::new(env!("CARGO_BIN_EXE_xezim"))
        .args([
            "--simulate",
            "-s",
            "tb_check",
            path.to_str().unwrap(),
            "--no-cache",
        ])
        .output()
        .expect("run xezim");
    let mut text = String::from_utf8_lossy(&out.stdout).to_string();
    text.push_str(&String::from_utf8_lossy(&out.stderr));
    text
}

#[test]
fn package_property_with_ports_asserts_and_counts() {
    let text = run(
        "pkg_prop",
        r#"package chk_pkg;
  property no_overlap(clk, arm, req_a, req_b);
    @(posedge clk) not(arm && (req_a && req_b || $isunknown(req_a) || $isunknown(req_b)));
  endproperty
endpackage

module tb_check;
  import chk_pkg::*;
  bit clk, arm;
  logic req_a, req_b;
  int pass_count = 0, fail_count = 0;
  always #5 clk = ~clk;

  chk_overlap :
  assert property (no_overlap(clk, arm, req_a, req_b)) pass_count++;
  else begin
    fail_count++;
    $display("[VIOLATION] @%0t arm=%b a=%b b=%b", $time, arm, req_a, req_b);
  end

  initial begin
    clk = 0; arm = 0; req_a = 0; req_b = 0;
    @(posedge clk); arm = 0; req_a = 1'b1; req_b = 1'b1;  // masked: arm low
    @(posedge clk); arm = 0; req_a = 1'bx; req_b = 1'b0;  // masked: arm low
    @(posedge clk); arm = 1; req_a = 1'b1; req_b = 1'b0;  // pass
    @(posedge clk); arm = 1; req_a = 1'b0; req_b = 1'b1;  // pass
    @(posedge clk); arm = 1; req_a = 1'b0; req_b = 1'b0;  // pass
    @(posedge clk); arm = 1; req_a = 1'b1; req_b = 1'b1;  // FAIL
    @(posedge clk); arm = 1; req_a = 1'b1; req_b = 1'b0;  // pass
    @(posedge clk); arm = 1; req_a = 1'bx; req_b = 1'b0;  // FAIL
    @(posedge clk); arm = 1; req_a = 1'b0; req_b = 1'bz;  // FAIL
    @(posedge clk); arm = 1; req_a = 1'b0; req_b = 1'b0;  // pass
    @(posedge clk);
    $display("T|pass=%0d fail=%0d", pass_count, fail_count);
    if (pass_count == 7 && fail_count == 3) $display("TEST_PASS");
    else $display("TEST_FAIL");
    $finish;
  end
endmodule
"#,
    );
    assert!(text.contains("T|pass=7 fail=3"), "counts:\n{text}");
    assert!(text.contains("TEST_PASS"), "verdict:\n{text}");
}

#[test]
fn module_property_with_ports_next_to_portless_one() {
    let text = run(
        "mod_prop",
        r#"module tb_check;
  bit clk, arm; logic a, b;
  int pc = 0, fc = 0;
  always #5 clk = ~clk;
  property with_ports(clk, r, x, y);
    @(posedge clk) not(r && (x && y));
  endproperty
  property portless;
    @(posedge clk) not(arm && (a && b));
  endproperty
  assert property (with_ports(clk, arm, a, b)) pc++; else fc++;
  assert property (portless)                    pc++; else fc++;
  initial begin
    arm = 1; a = 0; b = 0;
    @(posedge clk); a = 1; b = 1;
    @(posedge clk); a = 0; b = 0;
    @(posedge clk); @(posedge clk);
    $display("T|pc=%0d fc=%0d", pc, fc);
    $finish;
  end
endmodule
"#,
    );
    assert!(text.contains("T|pc=4 fc=2"), "both asserts fire:\n{text}");
}
