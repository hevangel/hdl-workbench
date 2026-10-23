//! Issue #281: `$readmemh` into another instance's memory through a
//! hierarchical name (`r.mem`) loaded nothing, silently, when the call was in
//! a task. Inside a subroutine body (and through an instance-array element,
//! `ra[1].mem`, anywhere) the target arrives as a member-access chain rather
//! than a flat hierarchical name, and the memory-task target resolver only
//! accepted the flat form. IEEE 1800-2023 §21.4 (`$readmemb`/`$readmemh`),
//! §21.5 (`$writememb`/`$writememh`) and §23.6 (hierarchical names). Every
//! expected value below comes from the reference simulator.

use xezim::simulate;

fn t_lines(src: &str) -> Vec<String> {
    simulate(src, 100)
        .expect("simulate")
        .output
        .iter()
        .map(|o| o.message.trim_end().to_string())
        .filter(|l| l.starts_with("T|") || l.starts_with("mem[") || l.contains("[xezim]"))
        .collect()
}

fn scratch(tag: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!("xezim_i281_{}_{}", tag, std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

/// The issue's reproducer.
#[test]
fn issue_281_reproducer() {
    let dir = scratch("repro");
    let hex = dir.join("m.hex");
    std::fs::write(&hex, "deeeddde\n").unwrap();
    let src = format!(
        r#"
module ram;
   reg [31:0] mem [0:7];
endmodule

module top;
   ram r ();

   task load;
      $readmemh("{0}", r.mem);
   endtask

   initial begin
      load();
      $display("mem[0]=%h (expect deeeddde)", r.mem[0]);
   end
endmodule
"#,
        hex.display()
    );
    let got = t_lines(&src);
    let _ = std::fs::remove_dir_all(&dir);
    assert_eq!(got, ["mem[0]=deeeddde (expect deeeddde)"]);
}

/// Calls from a task, a function, a class method, an instance's own task
/// (upward and absolute names) and an `initial` block; an instance-array
/// element; a packed-arena memory (over 100k cells) with start/end
/// addresses; and `$writememh`/`$writememb` from a task.
#[test]
fn readmem_writemem_hierarchical_targets() {
    let dir = scratch("forms");
    let hex = dir.join("m.hex");
    let bin = dir.join("b.mem");
    let wh = dir.join("w.hex");
    let wb = dir.join("wb.bin");
    std::fs::write(&hex, "deeeddde\n11112222\n33334444\n55556666\n").unwrap();
    std::fs::write(
        &bin,
        "@186a0\n00000001\n00000010\n00000011\n00000100\n00000101\n",
    )
    .unwrap();
    let src = r#"
module ram;
  reg [31:0] mem [0:7];
  reg [31:0] mem2 [0:7];
  reg [31:0] mem3 [0:7];
  sub s ();
endmodule
module sub;
  reg [31:0] umem [0:7];
  task upload;
    $readmemh("@HEX@", top.r.mem3);
  endtask
  task upl2;
    $readmemh("@HEX@", r.mem2);
  endtask
endmodule
module big;
  reg [7:0] bm [0:131071];
endmodule
module top;
  ram r ();
  ram ra [0:1] ();
  big g ();
  reg [31:0] wb [0:7];
  class L;
    function void ld();
      $readmemh("@HEX@", r.mem2);
    endfunction
  endclass
  task load;
    $readmemh("@HEX@", r.mem);
  endtask
  function void fload();
    $readmemh("@HEX@", r.s.umem);
  endfunction
  task loadrange;
    $readmemh("@HEX@", ra[1].mem, 2, 5);
  endtask
  task loadbig;
    $readmemb("@BIN@", g.bm, 100000, 100003);
  endtask
  task wr;
    $writememh("@WH@", r.mem);
    $writememb("@WB@", r.mem, 0, 1);
  endtask
  initial begin
    L o;
    o = new;
    load();
    $display("T|task r.mem[0]=%h r.mem[3]=%h", r.mem[0], r.mem[3]);
    fload();
    $display("T|func r.s.umem[1]=%h", r.s.umem[1]);
    o.ld();
    $display("T|class r.mem2[2]=%h", r.mem2[2]);
    loadrange();
    $display("T|range ra1.mem[1]=%h [2]=%h [5]=%h [6]=%h", ra[1].mem[1], ra[1].mem[2], ra[1].mem[5], ra[1].mem[6]);
    loadbig();
    $display("T|big bm[99999]=%h [100000]=%h [100003]=%h [100004]=%h", g.bm[99999], g.bm[100000], g.bm[100003], g.bm[100004]);
    r.s.upload();
    $display("T|down-abs r.mem3[3]=%h", r.mem3[3]);
    r.mem2[0] = 0;
    r.s.upl2();
    $display("T|upward r.mem2[0]=%h", r.mem2[0]);
    wr();
    $readmemh("@WH@", wb);
    $display("T|writememh wb[0]=%h wb[3]=%h wb[4]=%h", wb[0], wb[3], wb[4]);
    $readmemb("@WB@", wb);
    $display("T|writememb wb[0]=%h wb[1]=%h", wb[0], wb[1]);
    $readmemh("@HEX@", ra[0].mem);
    $display("T|init ra0.mem[0]=%h", ra[0].mem[0]);
  end
endmodule
"#
    .replace("@HEX@", &hex.display().to_string())
    .replace("@BIN@", &bin.display().to_string())
    .replace("@WH@", &wh.display().to_string())
    .replace("@WB@", &wb.display().to_string());
    let got = t_lines(&src);
    let _ = std::fs::remove_dir_all(&dir);
    assert_eq!(
        got,
        [
            "T|task r.mem[0]=deeeddde r.mem[3]=55556666",
            "T|func r.s.umem[1]=11112222",
            "T|class r.mem2[2]=33334444",
            "T|range ra1.mem[1]=xxxxxxxx [2]=deeeddde [5]=55556666 [6]=xxxxxxxx",
            "T|big bm[99999]=xx [100000]=01 [100003]=04 [100004]=xx",
            "T|down-abs r.mem3[3]=55556666",
            "T|upward r.mem2[0]=deeeddde",
            "T|writememh wb[0]=deeeddde wb[3]=55556666 wb[4]=xxxxxxxx",
            "T|writememb wb[0]=deeeddde wb[1]=11112222",
            "T|init ra0.mem[0]=deeeddde",
        ],
    );
}

/// A target that names no memory rejects the design at elaboration: `r.nomem`
/// is a hierarchical name that does not resolve (IEEE 1800-2023 §23.8).
#[test]
fn readmem_rejects_missing_target() {
    let dir = scratch("missing");
    let hex = dir.join("m.hex");
    std::fs::write(&hex, "deeeddde\n").unwrap();
    let src = format!(
        r#"
module ram;
  reg [31:0] mem [0:7];
endmodule
module top;
  ram r ();
  task load;
    $readmemh("{0}", r.nomem);
    $writememh("{0}.out", r.nomem);
  endtask
  initial load();
endmodule
"#,
        hex.display()
    );
    let err = simulate(&src, 100).err().expect("elaboration error");
    let _ = std::fs::remove_dir_all(&dir);
    assert!(
        err.contains("hierarchical name 'r.nomem' does not resolve"),
        "{err}"
    );
}

/// A file that cannot be opened is a warning, and the memory is left
/// unchanged.
#[test]
fn readmem_reports_missing_file() {
    let dir = scratch("nofile");
    let nofile = dir.join("nofile.hex");
    let src = format!(
        r#"
module ram;
  reg [31:0] mem [0:7];
endmodule
module top;
  ram r ();
  task load;
    $readmemh("{0}", r.mem);
  endtask
  initial begin
    load();
    $display("T|done %h", r.mem[0]);
  end
endmodule
"#,
        nofile.display()
    );
    let sim = simulate(&src, 100).expect("simulate");
    let lines: Vec<String> = sim
        .output
        .iter()
        .map(|o| o.message.trim_end().to_string())
        .collect();
    let _ = std::fs::remove_dir_all(&dir);
    assert!(
        lines
            .iter()
            .any(|l| l.contains("[xezim][warning] $readmemh: cannot open file")),
        "{lines:?}"
    );
    assert!(lines.iter().any(|l| l == "T|done xxxxxxxx"), "{lines:?}");
    assert_eq!(sim.error_count, 0);
}

/// Generate-block scopes: a for-generate element (`g[1].gm`), an instance
/// inside one (`g[0].rr.mem`) and an if-generate block (`c.cm`, loaded with a
/// descending address range), all from a task.
#[test]
fn readmem_into_generate_scopes_from_a_task() {
    let dir = scratch("gen");
    let hex = dir.join("m.hex");
    std::fs::write(&hex, "deeeddde\n11112222\n33334444\n55556666\n").unwrap();
    let src = r#"
module ram;
  reg [31:0] mem [0:7];
endmodule
module top;
  for (genvar i = 0; i < 2; i++) begin : g
    reg [31:0] gm [0:3];
    ram rr ();
  end
  if (1) begin : c
    reg [31:0] cm [0:3];
  end
  task load;
    $readmemh("@HEX@", g[1].gm);
    $readmemh("@HEX@", g[0].rr.mem, 1);
    $readmemh("@HEX@", c.cm, 3, 0);
  endtask
  initial begin
    load();
    $display("T|g1.gm[0]=%h g0.gm[0]=%h", g[1].gm[0], g[0].gm[0]);
    $display("T|g0.rr.mem[1]=%h [0]=%h", g[0].rr.mem[1], g[0].rr.mem[0]);
    $display("T|c.cm[3]=%h c.cm[0]=%h", c.cm[3], c.cm[0]);
  end
endmodule
"#
    .replace("@HEX@", &hex.display().to_string());
    let got = t_lines(&src);
    let _ = std::fs::remove_dir_all(&dir);
    assert_eq!(
        got,
        [
            "T|g1.gm[0]=deeeddde g0.gm[0]=xxxxxxxx",
            "T|g0.rr.mem[1]=deeeddde [0]=xxxxxxxx",
            "T|c.cm[3]=deeeddde c.cm[0]=55556666",
        ],
    );
}
