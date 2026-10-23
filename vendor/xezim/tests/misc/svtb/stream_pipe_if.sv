`ifndef SVTEST_DEFS_SVH
`define SVTEST_DEFS_SVH
`define SVTEST_INIT \
  int failures = 0;
`define SVTEST_CHECK(expr, msg) \
  if (!(expr)) begin \
    failures++; \
    $display("FAIL @%0t : %s", $time, msg); \
  end
`define SVTEST_PASSFAIL \
  if (failures == 0) begin \
    $display("TEST_PASS"); \
  end else begin \
    $display("TEST_FAIL count=%0d", failures); \
    $fatal(1); \
  end
`endif

interface stream_if;
   logic        valid;
   logic        ready;
   logic [63:0] payload;
   logic [15:0] seq;
   logic        rsp_valid;
   logic        lane_valid;
   logic [63:0] local_payload;
   logic [15:0] local_seq;
   logic        token_valid;
   logic [2:0]  token_count;
endinterface
module ep_leaf(
   stream_if pipe_if
);
   always_comb begin
      pipe_if.token_valid = pipe_if.lane_valid;
      pipe_if.token_count = pipe_if.lane_valid ? 3 : 0;
   end
endmodule
module ep_adapter(
   stream_if pipe_if
);
   ep_leaf u_endpoint(.pipe_if(pipe_if));
endmodule
module ep_router(
   input logic clk,
   input logic rst_n,
   stream_if pipe_if
);
   typedef struct {
      logic [63:0] payload;
      logic [15:0] seq;
      integer latency;
   } item_t;
   item_t q[$];
   ep_adapter u_adapter(.pipe_if(pipe_if));
   integer i;
   always_ff @(posedge clk) begin
      if(!rst_n) begin
         q.delete();
         pipe_if.ready         <= 0;
         pipe_if.rsp_valid     <= 0;
         pipe_if.lane_valid    <= 0;
         pipe_if.local_payload <= 0;
         pipe_if.local_seq     <= 0;
      end
      else begin
         pipe_if.ready      <= $urandom_range(0,1);
         pipe_if.rsp_valid  <= 0;
         pipe_if.lane_valid <= 0;
         if(pipe_if.valid && pipe_if.ready) begin
            item_t item;
            item.payload = pipe_if.payload;
            item.seq = pipe_if.seq;
            item.latency = $urandom_range(0,7);
            q.push_back(item);
            pipe_if.rsp_valid <= 1;
         end
         for(i=0;i<q.size();i++)
            q[i].latency--;
         if(q.size() > 0 && q[0].latency <= 0) begin
            pipe_if.lane_valid <= 1;
            pipe_if.local_payload <= q[0].payload;
            pipe_if.local_seq <= q[0].seq;
            q.pop_front();
         end
      end
   end
endmodule
module stream_dut(
   input logic clk,
   input logic rst_n,
   stream_if pipe_if
);
   ep_router u_router(.clk(clk), .rst_n(rst_n), .pipe_if(pipe_if));
endmodule
class txn;
   bit [63:0] payload;
   bit [15:0] seq;
endclass
module tb_stream;
   logic clk;
   logic rst_n;
   stream_if pipe_if();
   stream_dut dut(.clk(clk), .rst_n(rst_n), .pipe_if(pipe_if));
   `SVTEST_INIT
   mailbox exp_mb;
   mailbox act_mb;
   integer accepted_cnt;
   integer completed_cnt;
   initial clk = 0;
   always #5 clk = ~clk;
   initial begin
      rst_n = 0;
      repeat(5) @(posedge clk);
      rst_n = 1;
   end
   initial begin
      exp_mb = new();
      act_mb = new();
      accepted_cnt  = 0;
      completed_cnt = 0;
   end
   task driver;
      static int seq_id = 0;
      wait(rst_n);
      forever begin
         @(posedge clk);
         pipe_if.valid <= $urandom_range(0,1);
         pipe_if.payload <= { $urandom(), $urandom() };
         pipe_if.seq <= seq_id;
         if(pipe_if.valid && pipe_if.ready)
            seq_id++;
      end
   endtask
   task input_monitor;
      txn t;
      wait(rst_n);
      forever begin
         @(posedge clk);
         if(pipe_if.valid && pipe_if.ready) begin
            t = new();
            t.payload = pipe_if.payload;
            t.seq = pipe_if.seq;
            exp_mb.put(t);
            accepted_cnt++;
         end
      end
   endtask
   task output_monitor;
      txn t;
      wait(rst_n);
      forever begin
         @(posedge clk);
         if(pipe_if.lane_valid) begin
            t = new();
            t.payload = pipe_if.local_payload;
            t.seq = pipe_if.local_seq;
            act_mb.put(t);
            completed_cnt++;
         end
      end
   endtask
   task scoreboard;
      txn exp;
      txn act;
      forever begin
         exp_mb.get(exp);
         act_mb.get(act);
         `SVTEST_CHECK(exp.seq == act.seq, "sequence mismatch")
         `SVTEST_CHECK(exp.payload == act.payload, "payload mismatch")
      end
   endtask
   task protocol_checker;
      wait(rst_n);
      forever begin
         @(posedge clk);
         `SVTEST_CHECK(pipe_if.token_valid == pipe_if.lane_valid, "token_valid mismatch")
         if(pipe_if.token_valid)
            `SVTEST_CHECK(pipe_if.token_count == 3, "token_count mismatch")
      end
   endtask
   initial fork
      driver();
      input_monitor();
      output_monitor();
      scoreboard();
      protocol_checker();
   join_none
   initial begin
      wait(rst_n);
      repeat(5000) @(posedge clk);
      `SVTEST_CHECK(accepted_cnt > 100, "insufficient accepted traffic")
      `SVTEST_CHECK(completed_cnt > 100, "insufficient completed traffic")
      $display("Accepted  = %0d", accepted_cnt);
      $display("Completed = %0d", completed_cnt);
      `SVTEST_PASSFAIL
      $finish;
   end
endmodule
