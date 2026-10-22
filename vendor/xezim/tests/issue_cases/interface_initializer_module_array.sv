// IEEE 1800 §6.21 / §25.3: interface state belongs to each elaborated scope.
interface path_if;
  string path = $sformatf("%m");
endinterface
module wrapper;
  path_if bus();
endmodule
module top;
  wrapper holder[3:2]();
  initial begin
    $display("T|%s", holder[3].bus.path);
    $display("T|%s", holder[2].bus.path);
    $finish;
  end
endmodule
