// IEEE 1800 §6.21 / §23.11: bound interfaces have ordinary static startup.
interface path_if;
  string path = $sformatf("%m");
endinterface
module host;
endmodule
module top;
  host h();
  bind host path_if bus();
  initial begin
    $display("T|%s", h.bus.path);
    $finish;
  end
endmodule
