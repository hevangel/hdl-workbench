interface bus_if;
 logic [8:0] state;
 modport slv (output state);
endinterface
module regs (bus_if.slv bus);
 initial #1 bus.state = 9'h1A;
endmodule
module dut;
 bus_if b();
 regs u_regs (.bus(b));
endmodule
module top;
 dut u_dut();
 initial begin
 wait (u_dut.u_regs.bus.state === 9'h1A);
 $display("state=%h at %0t", u_dut.u_regs.bus.state, $time);
 $finish;
 end
endmodule
