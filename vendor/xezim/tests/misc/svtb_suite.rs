//! Self-checking SystemVerilog testbench suite (tests/misc/svtb/*.sv): each
//! DUT+TB pair prints TEST_PASS when its own scoreboard/reference model is
//! satisfied, and every one is verified to pass in the reference simulator.
//! The set covers the shapes recent scheduler/comb/bytecode fixes were built
//! for: parameterized pipeline stages with queue scoreboards, packed-array
//! bitcast windows, per-slot cooldown trackers, LRM always_comb semantics
//! (function hidden deps, time-0 eval), cross-scope randomization
//! constraints, interface hierarchies with queue routers and mailboxes,
//! program-block/class/type parameters, NBA last-assign-wins with chaotic
//! X-clock startups, struct config register files, credit pools, and a
//! dual-clock gray-code credit synchronizer whose TB signal names SHADOW the
//! DUT port names (the round-69 dependency bug shape).

use std::process::Command;

fn run_tb(file: &str, top: &str) -> String {
    let src = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/misc/svtb")
        .join(file);
    let out = Command::new(env!("CARGO_BIN_EXE_xezim"))
        .args([
            "--simulate",
            "-s",
            top,
            src.to_str().unwrap(),
            "--no-cache",
            "--max-time",
            "200000",
        ])
        .output()
        .expect("run xezim");
    let mut text = String::from_utf8_lossy(&out.stdout).to_string();
    text.push_str(&String::from_utf8_lossy(&out.stderr));
    text
}

fn assert_pass(file: &str, top: &str) {
    let text = run_tb(file, top);
    assert!(
        text.contains("TEST_PASS") && !text.contains("TEST_FAIL"),
        "{file} did not TEST_PASS:\n{text}"
    );
}

#[test]
fn flow_stages_queue_scoreboard() {
    assert_pass("flow_stages.sv", "tb_flow_stages");
}

#[test]
fn bitcast_window_part_select() {
    assert_pass("bitcast_window.sv", "tb_bitcast");
}

#[test]
fn cool_tracker_lfsr_model() {
    assert_pass("cool_tracker.sv", "tb_cool");
}

#[test]
fn comb_lrm_semantics() {
    assert_pass("comb_lrm.sv", "tb_comb_lrm");
}

#[test]
fn rand_cross_scope_constraints() {
    assert_pass("rand_scope.sv", "tb_rand_scope");
}

#[test]
fn stream_pipe_interface_router() {
    assert_pass("stream_pipe_if.sv", "tb_stream");
}

#[test]
fn program_block_parameters() {
    assert_pass("prog_params.sv", "tb_prog_params");
}

#[test]
fn lane_calc_shift_pipeline() {
    assert_pass("lane_calc.sv", "tb_lane_calc");
}

#[test]
fn nba_override_chaotic_startup() {
    assert_pass("nba_override.sv", "tb_nba_override");
}

#[test]
fn cfg_regs_struct_hierarchy() {
    assert_pass("cfg_regs.sv", "tb_cfg_regs");
}

#[test]
fn credit_pool_constraint_modes() {
    assert_pass("credit_pool.sv", "tb_credit_pool");
}

#[test]
fn struct_params_consumer() {
    assert_pass("struct_params.sv", "tb_combined_struct_test");
}

#[test]
fn vif_manager_modports() {
    assert_pass("vif_manager.sv", "tb_vif_mgr");
}

#[test]
fn cdc_client_dual_clock_sync() {
    // Full dual-clock gray-code credit synchronizer. The TB declares
    // `quad_sync_t sync_in;` — the SAME NAME as the DUT's input port — the
    // exact shadowing that round 69 fixed (member-fanout assigns froze on
    // the pre-update port value because their dependency resolved only to
    // the TB-level signal).
    assert_pass("cdc_client.sv", "tb_cdc_client");
}

#[test]
fn port_struct_shadow_dep_minimal() {
    // Minimal pin of the round-69 fix: a member-fanout assign inside an
    // instance whose input port shares its name with the connected TB
    // signal. The assign's dep must include the instance-scoped port copy,
    // or a waiter-continuation write to the TB struct never propagates.
    let dir = std::env::temp_dir().join(format!("xezim_pssd_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("pssd.sv");
    std::fs::write(
        &path,
        r#"package sp;
  typedef struct packed { logic [9:0] f1; logic [9:0] f0; } duo_t;
endpackage
module fanout import sp::*; (sync_in, fan);
  input  duo_t sync_in;
  output logic [1:0][9:0] fan;
  assign fan[0] = sync_in.f0;
  assign fan[1] = sync_in.f1;
endmodule
module test;
  import sp::*;
  duo_t sync_in;
  wire [1:0][9:0] fan;
  logic clk = 0; always #2.5 clk = ~clk;
  fanout u_f (.sync_in(sync_in), .fan(fan));
  initial begin
    sync_in.f0 = '0; sync_in.f1 = '0;
    #10;
    @(posedge clk);
    sync_in.f0 = 10'h1A5;
    sync_in.f1 = 10'h25B;
    #10;
    $display("T|fan0=%h fan1=%h", fan[0], fan[1]);
    $finish;
  end
endmodule
"#,
    )
    .unwrap();
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
    let text = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(text.contains("T|fan0=1a5 fan1=25b"), "{text}");
}
