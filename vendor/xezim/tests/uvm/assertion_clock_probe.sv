`timescale 1ns/1ns
module clock_guard(input logic clk, input logic level);
  int fails = 0;
  ap_guard: assert property (@(posedge clk) level |=> !level)
    else begin
      fails++;
      $display("T|guard %m time=%0t", $time);
    end
endmodule

interface channel_if(input logic clk);
  logic level = 1;
  int fails = 0;
  ap_guard: assert property (@(posedge clk) level |=> !level)
    else begin
      fails++;
      $display("T|guard %m time=%0t", $time);
    end
endinterface

import uvm_pkg::*;
`include "uvm_macros.svh"
class clock_test extends uvm_test;
  `uvm_component_utils(clock_test)
  virtual channel_if vif;
  function new(string name = "clock_test", uvm_component parent = null);
    super.new(name, parent);
  endfunction
  function void build_phase(uvm_phase phase);
    bit bound;
    bound = uvm_config_db#(virtual channel_if)::get(this, "", "vif", vif);
    $display("T|bound=%0d", bound);
  endfunction
  task run_phase(uvm_phase phase);
    phase.raise_objection(this);
    $display("T|run_start=%0t", $time);
    top.started = 1;
    #16;
    phase.drop_objection(this);
  endtask
endclass

module top;
  logic clk = 0;
  bit started = 0;
  int top_fails = 0;
  initial begin
    wait (started);
    forever #1 clk = ~clk;
  end
  channel_if channel(clk);
  clock_guard direct(.clk(clk), .level(channel.level));
  ap_top: assert property (@(posedge clk) channel.level |=> !channel.level)
    else begin
      top_fails++;
      $display("T|top time=%0t", $time);
    end
  initial begin
    virtual channel_if handle;
    handle = channel;
    uvm_config_db#(virtual channel_if)::set(null, "*", "vif", handle);
    run_test("clock_test");
  end
  final $display("T|counts top=%0d direct=%0d inner=%0d", top_fails, direct.fails, channel.fails);
  initial begin
    #100;
    $fatal(1, "watchdog");
  end
endmodule
