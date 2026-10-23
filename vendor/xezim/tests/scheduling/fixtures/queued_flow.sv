`timescale 1ps/1ps
typedef enum logic [2:0] { Q_IDLE=0, Q_FETCH=1, Q_IN=2, Q_IN_DATA=3,
  Q_OUT=4, Q_OUT_DATA=5, Q_DRAIN=6, Q_BARRIER=7 } phase_t;
typedef struct packed {
  logic inbound;
  logic outbound;
  logic inbound_wait;
  logic outbound_wait;
  logic tail;
  logic [10:0] span;
} descriptor_t;
typedef struct packed { logic [31:0] count; } segment_t;

class flow_agent;
  bit variant;
  descriptor_t saved;
  int unsigned pause_count, take_count, look_count, segment_count, last_take;
  function new(bit variant);
    this.variant=variant;
    saved='0; pause_count=0; take_count=0; look_count=0; segment_count=0; last_take=0;
  endfunction
  virtual task decode(input logic [63:0] token, output descriptor_t decoded,
                      input int preview=0);
    descriptor_t prior;
    prior=saved;
    saved.inbound=((token[39:36]==0)||(token[39:36]==1)) && token[62];
    saved.outbound=((token[39:36]==0)||(token[39:36]==1)) && !token[62];
    saved.inbound_wait=(token[39:36]==6) && !variant;
    saved.outbound_wait=(token[39:36]==6) && variant;
    saved.tail=(saved.outbound || ((token[39:36]==0)&&token[62])) && token[54];
    saved.span=(saved.inbound||saved.outbound) ? ((token[35:25]+11'd1)>>2) : 11'd0;
    if(preview) look_count++;
    else begin take_count++; if((take_count%48)==0) pause_count=3; end
    decoded=saved;
    if(preview) saved=prior;
  endtask
  virtual function segment_t make_segment();
    segment_t entry;
    segment_count++;
    if(take_count!=last_take) begin
      entry.count={21'd0,saved.span}; last_take=take_count;
    end else entry.count=0;
    return entry;
  endfunction
  virtual function logic paused();
    paused=(pause_count!=0);
    if(pause_count!=0) pause_count--;
  endfunction
endclass

module flow_endpoint #(parameter int PORT=0, VARIANT=0, PAUSE=0, GAP=2)(
  input logic clock, ready, present,
  input logic [63:0] token,
  output logic consume, in_valid, out_credit, in_start, out_start, complete,
  input logic out_valid, release_bank,
  output wire [31:0] status
);
  flow_agent agent;
  descriptor_t selected, previewed;
  segment_t segment;
  phase_t phase, following;
  int cooldown, progress, limit, candidate_limit, credit_progress;
  int credits, accepted, tail_cycles;
  logic tail_hold, pause_hold, in_complete;
  logic in_pending, out_pending, in_complete_pending;
  logic in_latched, out_latched;
  wire phase_open, gate_open, consume_unpaused, out_complete;
  wire enabled=1'b1;
  wire in_enabled=enabled, out_enabled=enabled;
  initial begin
    selected='0; previewed='0; segment='0; phase=Q_IDLE;
    cooldown=0; progress=0; limit=0; credit_progress=0;
    credits=0; accepted=0; tail_cycles=0;
    tail_hold=0; pause_hold=0; in_complete=0; in_latched=0; out_latched=0;
    #1; agent=new((VARIANT!=0));
  end
  assign phase_open=(phase==Q_IDLE)||((phase==Q_IN_DATA)&&(progress==limit))||
    ((phase==Q_OUT_DATA)&&(progress==limit)&&(VARIANT==0))||
    ((phase==Q_OUT_DATA)&&(credit_progress==limit)&&(VARIANT!=0));
  assign gate_open=!tail_hold && !(present&&previewed.outbound_wait&&(phase!=Q_IDLE)) &&
    !(present&&previewed.outbound_wait&&(credits!=accepted));
  assign consume_unpaused=present&&phase_open&&(cooldown==0)&&gate_open;
  assign consume=consume_unpaused&&!pause_hold;
  assign in_pending=in_enabled&&(progress<limit)&&(phase==Q_IN_DATA);
  assign out_pending=out_enabled&&(credit_progress<limit)&&(phase==Q_OUT_DATA);
  assign in_complete_pending=(phase==Q_IN_DATA)&&(progress==limit);
  assign out_complete=(credit_progress==limit)&&out_credit;
  assign complete=out_complete||in_complete;
  always_comb candidate_limit=segment.count;
  always_comb begin
    in_start=((phase==Q_IN)||(phase==Q_BARRIER))&&(selected.inbound||selected.inbound_wait);
    out_start=((phase==Q_OUT)||(phase==Q_BARRIER))&&(selected.outbound||selected.outbound_wait);
    in_valid=in_latched; out_credit=out_latched;
  end
  always @(posedge consume_unpaused) begin
    #1;
    if(consume_unpaused) begin
      agent.decode(token,selected);
      $display("[%0t] ENDPOINT[%0d] TAKE op=%0d dir=%b len=%0d tail=%b count=%0d",
        $time,PORT,token[39:36],token[62],(token[35:25]+11'd1)>>2,token[54],agent.take_count);
    end
  end
  always @(clock) begin
    #1;
    if(!ready) previewed='0;
    else if(present) agent.decode(token,previewed,1);
  end
  always @(posedge clock) begin
    if((PAUSE==0)||(ready===1'b0)) pause_hold<=0;
    else pause_hold<=agent.paused();
  end
  always @(negedge clock) begin
    if(phase==Q_FETCH) segment=agent.make_segment();
  end
  always_comb begin
    following=phase;
    if(!ready) following=Q_IDLE;
    else if((phase==Q_IDLE)&&consume) following=Q_FETCH;
    else if((phase==Q_FETCH)&&(selected.inbound_wait||selected.outbound_wait)) following=Q_BARRIER;
    else if((phase==Q_FETCH)&&selected.inbound) following=Q_IN;
    else if((phase==Q_FETCH)&&selected.outbound) following=Q_OUT;
    else if(phase==Q_FETCH) following=Q_IDLE;
    else if((phase==Q_IN)&&(candidate_limit==0)) following=Q_IDLE;
    else if(phase==Q_IN) following=Q_IN_DATA;
    else if(phase==Q_BARRIER) following=Q_IDLE;
    else if((phase==Q_OUT)&&(candidate_limit==0)) following=Q_IDLE;
    else if(phase==Q_OUT) following=Q_OUT_DATA;
    else if((phase==Q_IN_DATA)&&(progress==limit)&&consume) following=Q_FETCH;
    else if((phase==Q_IN_DATA)&&(progress==limit)) following=Q_IDLE;
    else if((phase==Q_OUT_DATA)&&(progress==limit)&&(VARIANT==0)&&!consume) following=Q_IDLE;
    else if((phase==Q_OUT_DATA)&&(progress==limit)&&(VARIANT==0)&&consume) following=Q_FETCH;
    else if((phase==Q_OUT_DATA)&&(credit_progress==limit)&&(VARIANT!=0)&&present&&previewed.outbound_wait) following=Q_DRAIN;
    else if((phase==Q_OUT_DATA)&&(credit_progress==limit)&&(VARIANT!=0)&&!consume) following=Q_IDLE;
    else if((phase==Q_OUT_DATA)&&(credit_progress==limit)&&(VARIANT!=0)&&consume) following=Q_FETCH;
    else if((phase==Q_DRAIN)&&(credits==accepted)) following=Q_IDLE;
  end
  always @(posedge clock) if(!ready||(phase!=Q_FETCH)) phase<=following;
  always @(negedge clock) if(phase==Q_FETCH) phase<=following;
  always @(posedge clock) begin
    limit<=segment.count;
    in_complete<=!ready ? 1'b0 : in_complete_pending;
  end
  always @(negedge clock) begin
    in_latched<=!ready ? 1'b0 : in_pending;
    out_latched<=!ready ? 1'b0 : out_pending;
    if(!ready||release_bank) tail_hold<=0;
    else if(phase==Q_FETCH) tail_hold<=selected.tail;
    tail_cycles<=!tail_hold ? 0 : tail_cycles+1;
    if(!ready) begin
      progress<=limit; credit_progress<=limit; credits<=0; accepted<=0; cooldown<=0;
    end else begin
      if(phase==Q_FETCH) progress<=0;
      else if((progress<limit)&&(in_pending||out_valid)) progress<=progress+1;
      if(phase==Q_FETCH) credit_progress<=0;
      else if((credit_progress<limit)&&out_pending) credit_progress<=credit_progress+1;
      if(out_pending) credits<=credits+1;
      if(out_valid) accepted<=accepted+1;
      if(in_pending) cooldown<=GAP;
      else if(cooldown>0) cooldown<=cooldown-1;
    end
  end
  assign status[2:0]=phase;
  assign status[3]=tail_hold;
  assign status[4]=previewed.outbound_wait;
  assign status[5]=consume_unpaused;
  assign status[6]=gate_open;
  assign status[7]=phase_open;
  assign status[8]=pause_hold;
  assign status[25:10]=cooldown[15:0];
  assign status[31:26]=0;
endmodule

module flow_tb;
  logic clock, ready;
  initial clock=0;
  always #1 clock=~clock;
  initial begin ready=0; repeat(10) @(posedge clock); ready=1; end
  localparam int PORTS=4, REQUESTS=240, CAPACITY=4;
  wire [PORTS-1:0] present, consume, in_valid, out_credit, in_start, out_start, complete, out_valid;
  wire [PORTS-1:0][63:0] token;
  wire [PORTS-1:0][31:0] status;
  wire [PORTS-1:0][7:0] outstanding_view, queued_view, sent_view, finished_view, taken_view;
  wire [PORTS-1:0] done;
  function automatic logic [31:0] advance(input logic [31:0] seed);
    logic [31:0] work;
    work=seed^(seed<<13); work=work^(work>>17); work=work^(work<<5); return work;
  endfunction
  function automatic logic [63:0] build_token(input logic [31:0] seed, input int sent, input int outstanding);
    logic [63:0] work;
    work=0;
    if((sent%40)>=35) begin work[39:36]=0; work[62]=1; work[35:25]=7; end
    else begin
      if(seed[2:0]<3) begin work[39:36]=0; work[62]=1; end
      else if(seed[2:0]<5) begin work[39:36]=0; work[62]=0; end
      else work[39:36]=6;
      work[35:25]={7'd0,seed[4:3],2'b11};
      if((seed[9:5]==0)&&(outstanding==0)) work[54]=1;
    end
    return work;
  endfunction
  genvar channel;
  generate for(channel=0;channel<PORTS;channel++) begin: lanes
    logic [63:0] backlog[$];
    logic valid_local;
    logic [63:0] head_local;
    logic [7:0] depth;
    logic pulse, delayed_pulse, bank_release, pending_tail;
    logic [63:0] current_token, new_token;
    int outstanding, sent, finished, taken, adjustment, finish_events;
    logic [31:0] seed;
    logic barrier;
    always_comb begin
      valid_local=(backlog.size()!=0);
      head_local=(backlog.size()!=0) ? backlog[0] : 64'd0;
      depth=backlog.size();
    end
    assign present[channel]=valid_local;
    assign token[channel]=head_local;
    assign out_valid[channel]=out_credit[channel];
    assign done[channel]=(sent>=REQUESTS)&&(outstanding==0)&&(depth==0);
    flow_endpoint #(.PORT(channel),.VARIANT((channel>=2)?1:0),
      .PAUSE((channel==2)?1:0),.GAP(((channel%2)==0)?2:0)) endpoint(
      .clock(clock),.ready(ready),.present(valid_local),.token(head_local),
      .consume(consume[channel]),.in_valid(in_valid[channel]),.out_credit(out_credit[channel]),
      .in_start(in_start[channel]),.out_start(out_start[channel]),.complete(complete[channel]),
      .out_valid(out_valid[channel]),.release_bank(bank_release),.status(status[channel]));
    always @(posedge clock) begin
      adjustment=0;
      if(!ready) begin
        pulse<=0; delayed_pulse<=0; bank_release<=0; pending_tail<=0;
        outstanding<=0; sent<=0; finished<=0; taken<=0; finish_events<=0;
        seed<=32'h12345678+32'h9E3779B9*(channel+1);
        current_token<=0; new_token<=0; barrier<=0;
      end else begin
        pulse<=0; bank_release<=0; delayed_pulse<=pulse;
        if(consume[channel]) begin
          barrier=(backlog[0][39:36]==6);
          $display("[%0t] SOURCE[%0d] TAKE op=%0d dir=%b",$time,channel,backlog[0][39:36],backlog[0][62]);
          void'(backlog.pop_front()); taken<=taken+1;
          if(barrier) adjustment=adjustment-1;
        end
        if(delayed_pulse) backlog.push_back(current_token);
        if(finish_events!=finished) begin
          adjustment=adjustment-(finish_events-finished); finished<=finish_events;
          if(((outstanding+adjustment)==0)&&pending_tail) begin bank_release<=1; pending_tail<=0; end
        end
        if((sent<REQUESTS)&&(outstanding<CAPACITY)&&!pulse) begin
          seed<=advance(seed); new_token=build_token(seed,sent,outstanding);
          current_token<=new_token; pulse<=1; sent<=sent+1; adjustment=adjustment+1;
          if(new_token[54]) pending_tail<=1;
        end
        if(adjustment!=0) outstanding<=outstanding+adjustment;
      end
    end
    always @(posedge complete[channel]) if(ready) begin
      finish_events<=finish_events+1;
      $display("[%0t] SOURCE[%0d] COMPLETE#%0d",$time,channel,finish_events+1);
    end
    assign outstanding_view[channel]=outstanding[7:0];
    assign queued_view[channel]=depth;
    assign sent_view[channel]=sent[7:0];
    assign finished_view[channel]=finished[7:0];
    assign taken_view[channel]=taken[7:0];
  end endgenerate
  wire all_finished=&done;
  wire [PORTS-1:0] activity=consume|complete|in_valid|out_credit|in_start|out_start;
  int idle_cycles, heartbeat;
  initial begin idle_cycles=0; heartbeat=0; end
  always @(posedge clock) begin
    if(!ready) idle_cycles<=0;
    else if(all_finished||(|activity)) idle_cycles<=0;
    else idle_cycles<=idle_cycles+1;
  end
  always @(posedge clock) begin
    heartbeat<=heartbeat+1;
    if(ready&&((heartbeat%1000)==0)&&!all_finished)
      $display("[%0t] HEARTBEAT cycle=%0d outstanding=%0d %0d %0d %0d queued=%0d %0d %0d %0d phase=%0d %0d %0d %0d",
        $time,heartbeat,outstanding_view[0],outstanding_view[1],outstanding_view[2],outstanding_view[3],
        queued_view[0],queued_view[1],queued_view[2],queued_view[3],status[0][2:0],status[1][2:0],status[2][2:0],status[3][2:0]);
  end
  always @(posedge clock) begin
    if(ready&&(idle_cycles>1000)&&!all_finished) begin
      for(int port=0;port<PORTS;port++)
        $display("[%0t] STUCK port=%0d sent=%0d finished=%0d taken=%0d outstanding=%0d queued=%0d status=%h",
          $time,port,sent_view[port],finished_view[port],taken_view[port],outstanding_view[port],queued_view[port],status[port]);
      $display("FLOW_FAIL: stalled"); $finish;
    end
  end
  int elapsed_cycles;
  initial begin
    elapsed_cycles=0; wait(ready===1'b1);
    while(!all_finished&&(elapsed_cycles<20000)) begin @(posedge clock); elapsed_cycles=elapsed_cycles+1; end
    if(all_finished) $display("FLOW_PASS ports=%0d requests=%0d cycles=%0d",PORTS,REQUESTS,elapsed_cycles);
    else $display("FLOW_FAIL: limit");
    $finish;
  end
endmodule
