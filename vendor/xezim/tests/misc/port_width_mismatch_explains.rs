//! §23.3.3: a width-mismatched port connection is legal (the value is
//! truncated/extended), so xezim warns rather than errors. The bare
//! "port is 116 bit(s) but the connection is 86 bit(s)" left a user with no
//! way to find the under-sized declaration on a real design — the answer is
//! usually one packed struct whose fields were sized by a parameter that
//! resolved to 0, collapsing every `[P:0]` field to a single bit.
//!
//! The warning now names the connection's declaration (file:line), its type,
//! and the per-field widths, which makes that collapse self-evident.

use std::process::Command;

const SRC: &str = r#"`define REQ_W 116
module client (output logic [`REQ_W-1:0] req);
  assign req = '0;
endmodule
module test;
  localparam int P_UNSET = 0;
  typedef struct packed {
    logic [P_UNSET:0] f0;
    logic [P_UNSET:0] f1;
    logic [P_UNSET:0] f2;
    logic [P_UNSET:0] f3;
    logic [P_UNSET:0] f4;
    logic [80:0]      rest;
  } pkt_t;
  pkt_t pkt;
  client u_c (.req(pkt));
endmodule
"#;

#[test]
fn port_width_mismatch_names_the_connection_and_its_fields() {
    let dir = std::env::temp_dir().join(format!("xezim_pwm_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let src = dir.join("pwm.sv");
    std::fs::write(&src, SRC).unwrap();

    let out = Command::new(env!("CARGO_BIN_EXE_xezim"))
        .args([
            "--compile",
            "-s",
            "test",
            src.to_str().unwrap(),
            "--no-cache",
        ])
        .output()
        .expect("run xezim");
    let mut text = String::from_utf8_lossy(&out.stdout).to_string();
    text.push_str(&String::from_utf8_lossy(&out.stderr));

    assert!(
        text.contains("port width mismatch")
            && text.contains("116 bit(s)")
            && text.contains("86 bit(s)"),
        "keeps the §23.3.3 mismatch report:\n{}",
        text
    );
    assert!(
        text.contains("connection 'pkt' is declared as 'pkt_t'"),
        "names the connection and its type:\n{}",
        text
    );
    assert!(
        text.contains("pwm.sv:"),
        "locates the declaration in the source:\n{}",
        text
    );
    assert!(
        text.contains("its packed fields are: f0:1 f1:1 f2:1 f3:1 f4:1 rest:81"),
        "breaks the struct down field by field, in declaration order:\n{}",
        text
    );
    // The generic 1-bit NOTE is superseded by the per-field naming: every
    // collapsed field prints the identifier(s) in its range and their values.
    assert!(
        text.contains("field 'f0' resolved to 1 bit(s); its range reads: P_UNSET = 0"),
        "each collapsed field names its sizing parameter and value:\n{}",
        text
    );
    assert!(
        text.contains("XEZIM_TRACE_PARAM"),
        "and points at the tracing knob:\n{}",
        text
    );
    let _ = std::fs::remove_dir_all(&dir);
}

/// A correctly-sized connection produces no mismatch warning at all.
#[test]
fn matching_widths_stay_silent() {
    let dir = std::env::temp_dir().join(format!("xezim_pwm_ok_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let src = dir.join("ok.sv");
    std::fs::write(
        &src,
        SRC.replace("localparam int P_UNSET = 0;", "localparam int P_UNSET = 6;"),
    )
    .unwrap();

    let out = Command::new(env!("CARGO_BIN_EXE_xezim"))
        .args([
            "--compile",
            "-s",
            "test",
            src.to_str().unwrap(),
            "--no-cache",
        ])
        .output()
        .expect("run xezim");
    let mut text = String::from_utf8_lossy(&out.stdout).to_string();
    text.push_str(&String::from_utf8_lossy(&out.stderr));
    assert!(
        !text.contains("port width mismatch"),
        "7-bit fields make the struct exactly 116 bits:\n{}",
        text
    );
    let _ = std::fs::remove_dir_all(&dir);
}

/// The connection expression lives in the PARENT module's file. With a root
/// file large enough for the span offset to also fit inside it, the location
/// used to report the ROOT file (wrong file, spurious line) — the hint must
/// follow the module that owns the instantiation.
#[test]
fn mismatch_location_names_the_parent_modules_file() {
    let dir = std::env::temp_dir().join(format!("xezim_pwm_mf_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let leaf = dir.join("leaf.sv");
    let top = dir.join("top.sv");
    std::fs::write(
        &leaf,
        "module client (output logic [15:0] req);\n  assign req = '0;\nendmodule\n\
         module mid;\n  typedef struct packed { logic [1:0] f0; logic [2:0] f1; } pkt_t;\n\
         \x20 pkt_t pkt;\n  client u_c (.req(pkt));\nendmodule\n",
    )
    .unwrap();
    let pad = "// padding so the root file is larger than the leaf offset\n".repeat(400);
    std::fs::write(
        &top,
        format!("module test;\n  mid u_mid ();\nendmodule\n{}", pad),
    )
    .unwrap();

    let out = Command::new(env!("CARGO_BIN_EXE_xezim"))
        .args([
            "--compile",
            "-s",
            "test",
            top.to_str().unwrap(),
            leaf.to_str().unwrap(),
            "--no-cache",
        ])
        .output()
        .expect("run xezim");
    let mut text = String::from_utf8_lossy(&out.stdout).to_string();
    text.push_str(&String::from_utf8_lossy(&out.stderr));
    assert!(
        text.contains("leaf.sv:7"),
        "connection location must be in the parent's file (leaf.sv):\n{}",
        text
    );
    assert!(
        !text.contains("top.sv:1") && !text.contains("top.sv:2"),
        "must not misattribute the connection to the root file:\n{}",
        text
    );
    let _ = std::fs::remove_dir_all(&dir);
}

/// The decisive upgrade for the customer's 116-vs-86 case: a field sized by
/// a parameter that resolved to <= 0 (or not at all) prints the parameter's
/// NAME and value right in the warning — no need to open the typedef.
#[test]
fn collapsed_field_names_its_sizing_parameter() {
    let dir = std::env::temp_dir().join(format!("xezim_pwm_fld_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let src = dir.join("fld.sv");
    std::fs::write(
        &src,
        r#"module dut_core (input [115:0] dut_req);
endmodule
module testbench;
  localparam int SMEM_MASK_W = 0;
  typedef struct packed {
    logic [32:0] addr;
    logic [6:0]  bcnt_m1;
    logic        write;
    logic        wrap;
    logic [SMEM_MASK_W-1:0] mask;
  } tb_req_t;
  tb_req_t [1:0] tb_req;
  dut_core u_dut (.dut_req(tb_req));
endmodule
"#,
    )
    .unwrap();
    let out = Command::new(env!("CARGO_BIN_EXE_xezim"))
        .args([
            "--compile",
            "-s",
            "testbench",
            src.to_str().unwrap(),
            "--no-cache",
        ])
        .output()
        .expect("run xezim");
    let mut text = String::from_utf8_lossy(&out.stdout).to_string();
    text.push_str(&String::from_utf8_lossy(&out.stderr));
    assert!(
        text.contains("field 'mask' resolved to") && text.contains("SMEM_MASK_W = 0"),
        "the collapsed field must name its sizing parameter and value:\n{}",
        text
    );
    assert!(
        text.contains("XEZIM_TRACE_PARAM"),
        "and point at the tracing knob:\n{}",
        text
    );
    let _ = std::fs::remove_dir_all(&dir);
}
