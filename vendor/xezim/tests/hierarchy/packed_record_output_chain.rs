//! Packed records retain wide member updates through typed-to-flat output
//! ports, selected actuals, concatenation, and a parameterized pipeline.

use xezim::simulate;

#[test]
fn wide_member_updates_cross_the_complete_output_chain() {
    let sim = simulate(
        r#"
package parcel_types;
  typedef struct packed {
    logic [1:0][63:0] payload;
    logic [1:0][7:0]  lane_mask;
    logic [1:0]       attribute;
  } parcel_t;
  typedef struct packed { parcel_t [1:0] pair; } pair_t;
endpackage

module sample_ff #(parameter int WIDTH = 1) (
  input logic clk,
  input logic [WIDTH-1:0] in_value,
  output logic [WIDTH-1:0] out_value
);
  always @(posedge clk) out_value <= in_value;
endmodule

module sample_delay #(parameter int DEPTH = 1) (
  input logic clk,
  input logic [$bits(parcel_types::pair_t)-1:0] in_pair,
  output logic [$bits(parcel_types::pair_t)-1:0] out_pair,
  input logic [3:0] in_ready,
  output logic [3:0] out_ready
);
  wire [$bits(parcel_types::pair_t)-1:0] pair_q [DEPTH:0];
  wire [3:0] ready_q [DEPTH:0];
  assign pair_q[0] = in_pair;
  assign ready_q[0] = in_ready;
  assign out_pair = pair_q[DEPTH];
  assign out_ready = ready_q[DEPTH];
  generate
    for (genvar n = 0; n < DEPTH; n++) begin : stages
      sample_ff #(.WIDTH($bits(parcel_types::pair_t))) data_reg
        (.clk(clk), .in_value(pair_q[n]), .out_value(pair_q[n+1]));
      sample_ff #(.WIDTH(4)) ready_reg
        (.clk(clk), .in_value(ready_q[n]), .out_value(ready_q[n+1]));
    end
  endgenerate
endmodule

module parcel_source #(parameter int CHANNEL = 0, COUNT = 4) (
  input logic clk,
  input logic rst,
  output parcel_types::parcel_t packet_out,
  output logic [1:0] ready_out
);
  logic [1:0][63:0] value_reg;
  logic [1:0][7:0] mask_reg;
  logic [1:0] attr_reg, ready_reg;
  logic [7:0] count_reg;

  always @(posedge clk) begin
    if (rst) begin
      value_reg <= '0;
      mask_reg <= '0;
      attr_reg <= '0;
      ready_reg <= '0;
      count_reg <= '0;
    end else if (count_reg < COUNT) begin
      ready_reg <= 2'b11;
      value_reg[0] <= 64'(CHANNEL * 4096 + 2 * count_reg);
      value_reg[1] <= 64'(CHANNEL * 4096 + 2 * count_reg + 1);
      mask_reg <= '0;
      attr_reg <= count_reg[0] ? 2'b01 : 2'b11;
      count_reg <= count_reg + 1'b1;
    end else begin
      ready_reg <= '0;
    end
  end

  always_comb begin
    ready_out[0] = ready_reg[0];
    packet_out.payload[0] = value_reg[0];
    packet_out.lane_mask[0] = mask_reg[0];
    packet_out.attribute[0] = attr_reg[0];
    ready_out[1] = ready_reg[1];
    packet_out.payload[1] = value_reg[1];
    packet_out.lane_mask[1] = mask_reg[1];
    packet_out.attribute[1] = attr_reg[1];
  end
endmodule

module record_link #(parameter int CHANNEL = 0) (
  input logic clk,
  input logic rst,
  output parcel_types::parcel_t packet,
  output logic [1:0] ready
);
  parcel_source #(.CHANNEL(CHANNEL)) source
    (.clk(clk), .rst(rst), .packet_out(packet), .ready_out(ready));
endmodule

module flat_link #(parameter int CHANNEL = 0) (
  input logic clk,
  input logic rst,
  output logic [$bits(parcel_types::parcel_t)-1:0] bits,
  output logic [1:0] ready
);
  record_link #(.CHANNEL(CHANNEL)) link
    (.clk(clk), .rst(rst), .packet(bits), .ready(ready));
endmodule

module link_pair (
  input logic clk,
  input logic rst,
  output logic [$bits(parcel_types::parcel_t)-1:0] bits0,
  output logic [$bits(parcel_types::parcel_t)-1:0] bits1,
  output logic [1:0] ready0,
  output logic [1:0] ready1
);
  flat_link #(.CHANNEL(0)) first
    (.clk(clk), .rst(rst), .bits(bits0[$bits(parcel_types::parcel_t)-1:0]),
     .ready(ready0[1:0]));
  flat_link #(.CHANNEL(1)) second
    (.clk(clk), .rst(rst), .bits(bits1[$bits(parcel_types::parcel_t)-1:0]),
     .ready(ready1[1:0]));
endmodule

module packed_path (
  input logic clk,
  input logic rst,
  output logic [3:0] ready,
  output logic [$bits(parcel_types::pair_t)-1:0] pair
);
  logic [1:0] ready0, ready1;
  logic [$bits(parcel_types::parcel_t)-1:0] bits0, bits1;
  logic [3:0] merged_ready;
  logic [$bits(parcel_types::pair_t)-1:0] merged_pair;

  link_pair links (
    .clk(clk), .rst(rst),
    .bits0(bits0[$bits(parcel_types::parcel_t)-1:0]),
    .bits1(bits1[$bits(parcel_types::parcel_t)-1:0]),
    .ready0(ready0[1:0]), .ready1(ready1[1:0]));
  always_comb begin
    merged_ready = {ready1, ready0};
    merged_pair = {bits1, bits0};
  end
  sample_delay #(.DEPTH(1)) delay (
    .clk(clk), .in_pair(merged_pair), .out_pair(pair),
    .in_ready(merged_ready), .out_ready(ready));
endmodule

module tb;
  import parcel_types::*;
  logic clk = 1'b0;
  logic rst = 1'b1;
  always #5 clk = ~clk;

  parcel_t [0:0][1:0] observed;
  logic [0:0][1:0][1:0] observed_ready;
  int expected [2];
  int matched [2];

  packed_path dut
    (.clk(clk), .rst(rst), .pair(observed[0]), .ready(observed_ready[0]));

  initial begin
    expected[0] = 0;
    expected[1] = 4096;
    matched[0] = 0;
    matched[1] = 0;
    repeat (4) @(posedge clk);
    rst <= 1'b0;
  end

  always @(posedge clk) begin
    if (!rst) begin
      for (int channel = 0; channel < 2; channel++) begin
        if (observed_ready[0][channel][0]) begin
          if (observed[0][channel].payload[0] !== 64'(expected[channel])) begin
            $display("PACKED_CHAIN_FAIL channel=%0d lane=0", channel);
            $finish;
          end
          matched[channel]++;
        end
        if (observed_ready[0][channel][1]) begin
          if (observed[0][channel].payload[1] !== 64'(expected[channel] + 1)) begin
            $display("PACKED_CHAIN_FAIL channel=%0d lane=1", channel);
            $finish;
          end
          expected[channel] += 2;
        end
      end
      if (matched[0] >= 4 && matched[1] >= 4) begin
        $display("PACKED_CHAIN_PASS");
        $finish;
      end
    end
  end

  initial begin
    #500;
    $display("PACKED_CHAIN_FAIL timeout");
    $finish;
  end
endmodule
"#,
        1_000,
    )
    .expect("simulation failed");

    let output = sim
        .output
        .iter()
        .map(|line| line.message.as_str())
        .collect::<Vec<_>>()
        .join("\n");
    assert!(
        output.contains("PACKED_CHAIN_PASS"),
        "unexpected output:\n{output}"
    );
    assert!(
        !output.contains("PACKED_CHAIN_FAIL"),
        "unexpected output:\n{output}"
    );
}
