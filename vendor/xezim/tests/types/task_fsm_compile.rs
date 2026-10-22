//! Task-structured FSMs compile end to end: argument-carrying task inlines,
//! output-formal functions, register-bank local arrays, compile-time loop
//! unrolling with const-bound loop variables, case-LUTs, and §13.5.2 type
//! inheritance for comma-continued formals.
//!
//! The composite is the point: an AES-style always_ff that calls
//! `sub_bytes(); shift_rows(); mix_columns(); add_round_key(round+1);` used
//! to be ONE interpreted 280µs block per clock, because any single statement
//! failing bailed everything. Values are reference-verified (the shift/mix
//! pipeline below matches the commercial simulator bit for bit; a full AES
//! core passes its NIST vector in 0.47s where the interpreter took 22.5s).
//!
//! The masked byte-lane RAM write pins a PRE-EXISTING bug the unroller
//! exposed: `mem[addr][i*8 +: 8] <= ...` passed (base, WIDTH) through as
//! (hi, lo) and corrupted the neighbouring lane — ibex's prim_ram_2p shape.

use xezim::simulate;

fn notes(src: &str) -> Vec<String> {
    let sim = simulate(src, 1_000_000).expect("simulate failed");
    sim.output
        .iter()
        .map(|o| o.message.trim().to_string())
        .filter(|l| l.starts_with("NOTE:"))
        .collect()
}

const FSM: &str = r#"
module top;
  logic clk = 0;
  always #5 clk = ~clk;
  logic [7:0] state [0:15];
  logic [7:0] o0, o1;
  int phase = 0;

  // §13.5.2: a1/r1 inherit type AND direction from the previous formal.
  function automatic void mc(input logic [7:0] a0, a1, output logic [7:0] r0, r1);
    logic [7:0] t;
    t = a0 ^ a1;
    r0 = a0 ^ t;
    r1 = a1 ^ {t[6:0],1'b0};
  endfunction

  task automatic shr();
    logic [7:0] tmp [0:15];                       // register bank
    for (int i = 0; i < 16; i++) tmp[i] = state[i];
    state[1] = tmp[5];
    state[5] = tmp[9];
    state[2] = tmp[10];
  endtask

  task automatic mix();
    logic [7:0] c0, c1;
    for (int col = 0; col < 4; col++) begin       // unrolled; col is const
      c0 = state[col*4 + 0];
      c1 = state[col*4 + 1];
      mc(c0, c1, state[col*4 + 0], state[col*4 + 1]);
    end
  endtask

  always_ff @(posedge clk) begin
    if (phase == 0) begin
      for (int i = 0; i < 16; i++) state[i] = 8'(i * 7 + 3);
      phase <= 1;
    end else if (phase == 1) begin
      shr();
      phase <= 2;
    end else if (phase == 2) begin
      mix();
      phase <= 3;
    end
  end

  initial begin
    #45;
    $display("NOTE: s0=%0d s1=%0d s2=%0d s5=%0d s9=%0d",
             state[0], state[1], state[2], state[5], state[9]);
    $finish;
  end
endmodule
"#;

/// Reference-verified end state of the load -> shift -> mix pipeline.
#[test]
fn task_structured_fsm_compiles_and_matches_reference() {
    assert_eq!(notes(FSM), vec!["NOTE: s0=38 s1=108 s2=73 s5=248 s9=176"]);
}

const RAM_LANES: &str = r#"
module top;
  logic clk = 0;
  always #5 clk = ~clk;
  logic [31:0] mem [4];
  logic [1:0] addr;
  logic [31:0] wdata;
  logic [3:0] wmask;
  logic we = 0;
  always @(posedge clk) begin
    if (we) begin
      for (int i = 0; i < 4; i = i + 1) begin
        if (wmask[i]) begin
          mem[addr][i*8 +: 8] <= wdata[i*8 +: 8];
        end
      end
    end
  end
  initial begin
    mem[2] = 32'hAAAA_AAAA;
    addr = 2; wdata = 32'h1234_5678; wmask = 4'b0101; we = 1;
    @(posedge clk); #1 we = 0;
    $display("NOTE: mem2=%h", mem[2]);
    $finish;
  end
endmodule
"#;

/// §11.5.1 — lanes 0 and 2 written, lanes 1 and 3 untouched.
#[test]
fn masked_byte_lane_array_nba_hits_the_right_lanes() {
    assert_eq!(notes(RAM_LANES), vec!["NOTE: mem2=aa34aa78"]);
}
