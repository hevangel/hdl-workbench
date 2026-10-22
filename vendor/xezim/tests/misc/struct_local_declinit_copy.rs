//! §7.2/§13.4.2: declaration-with-initializer of an UNPACKED-struct local
//! copies member-wise, exactly like the assignment arm. The decl-init path
//! only handled class-property sources; from any other source — another
//! local, a queue/array element (`op_s x = accesses[i];`, UVM reg-map's bus
//! access loop) — the packed init value had no leaves to scatter and every
//! member of the fresh local stayed x. In UVM's `do_bus_read`/`do_bus_write`
//! that made `rw_access.kind` read x, the adapter drove x transactions, and
//! every RAL frontdoor data value came back x. Also pins §6.8/§6.18: a local
//! whose type is a TYPEDEF of a 2-state vector (`typedef bit [63:0] t;`)
//! initializes to 0, not x — UVM's field read-modify-write ORs into such a
//! local, so an x seed poisoned the whole register write. All expected
//! values reference-verified.

use std::process::Command;

fn run(name: &str, src: &str) -> String {
    let dir = std::env::temp_dir().join(format!("xezim_sdic_{}_{}", name, std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join(format!("{name}.sv"));
    std::fs::write(&path, src).unwrap();
    let out = Command::new(env!("CARGO_BIN_EXE_xezim"))
        .args([
            "--simulate",
            "-s",
            "test",
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
fn struct_local_decl_init_copies_members() {
    let text = run(
        "declinit",
        r#"package p;
  typedef struct {
    int  kind;
    logic [31:0] addr;
  } op_s;
endpackage
import p::*;
class c;
  task go();
    op_s q[$];
    op_s rw;
    op_s cp1;
    op_s cp2;
    rw.kind = 9; rw.addr = 32'h99;
    q.push_back(rw);
    cp1 = q[0];
    $display("T|assign-from-q kind=%0d addr=%h", cp1.kind, cp1.addr);
    cp2 = rw;
    $display("T|assign-from-var kind=%0d addr=%h", cp2.kind, cp2.addr);
    begin
      op_s cp3 = rw;
      $display("T|declinit-from-var kind=%0d addr=%h", cp3.kind, cp3.addr);
    end
    begin
      op_s cp4 = q[0];
      $display("T|declinit-from-q kind=%0d addr=%h", cp4.kind, cp4.addr);
    end
  endtask
endclass
module test;
  c cc = new();
  initial begin cc.go(); $finish; end
endmodule
"#,
    );
    for line in [
        "T|assign-from-q kind=9 addr=00000099",
        "T|assign-from-var kind=9 addr=00000099",
        "T|declinit-from-var kind=9 addr=00000099",
        "T|declinit-from-q kind=9 addr=00000099",
    ] {
        assert!(text.contains(line), "missing `{line}`:\n{text}");
    }
}

#[test]
fn struct_queue_roundtrip_in_access_loop() {
    // The reg-map bus access shape: build structs in a loop, push into a
    // local queue, read each back through a foreach decl-init and mutate the
    // copy without touching the queue element.
    let text = run(
        "access_loop",
        r#"package p;
  typedef logic [63:0] data_t;
  typedef enum { K_READ, K_WRITE } kind_e;
  typedef struct {
    kind_e     kind;
    data_t     addr;
    data_t     data;
    int        n_bits;
    bit [7:0]  byte_en;
  } op_s;
endpackage
import p::*;
class map_c;
  task go();
    op_s acc[$];
    for (int i = 0; i < 2; i++) begin
      op_s rw_access;
      rw_access.kind = K_READ;
      rw_access.addr = 'h10 + i*4;
      rw_access.data = i;
      rw_access.n_bits = 32;
      rw_access.byte_en = 8'hFF;
      acc.push_back(rw_access);
    end
    foreach (acc[i]) begin
      op_s rw_access = acc[i];
      $display("T|acc[%0d] kind=%0d addr=%h nb=%0d be=%h", i,
               rw_access.kind, rw_access.addr, rw_access.n_bits, rw_access.byte_en);
      rw_access.data = '0;
      $display("T|clr[%0d] data=%h addr=%h", i, rw_access.data, rw_access.addr);
    end
  endtask
endclass
module test;
  map_c m = new();
  initial begin m.go(); $finish; end
endmodule
"#,
    );
    for line in [
        "T|acc[0] kind=0 addr=0000000000000010 nb=32 be=ff",
        "T|clr[0] data=0000000000000000 addr=0000000000000010",
        "T|acc[1] kind=0 addr=0000000000000014 nb=32 be=ff",
        "T|clr[1] data=0000000000000000 addr=0000000000000014",
    ] {
        assert!(text.contains(line), "missing `{line}`:\n{text}");
    }
}

#[test]
fn two_state_typedef_local_defaults_to_zero() {
    let text = run(
        "twostate",
        r#"package p;
  typedef bit unsigned [63:0] data_t;
endpackage
import p::*;
class c;
  task go();
    data_t va;
    logic [63:0] vc;
    $display("T|va=%h vc=%h", va, vc);
    va |= 64'h3A00;
    $display("T|or va=%h", va);
  endtask
endclass
module test;
  c cc = new();
  initial begin cc.go(); $finish; end
endmodule
"#,
    );
    assert!(
        text.contains("T|va=0000000000000000 vc=xxxxxxxxxxxxxxxx"),
        "2-state typedef local is 0, 4-state stays x:\n{text}"
    );
    assert!(
        text.contains("T|or va=0000000000003a00"),
        "|= into clean 0 base:\n{text}"
    );
}
