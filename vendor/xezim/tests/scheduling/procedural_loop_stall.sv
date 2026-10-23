module top;
  class test;
    static task dummy_task(int i);
    endtask

    static task run();
      longint sum = 0;
      for (int i = 0; i < 200000; i++) begin
        dummy_task(i);
        sum += i;
      end
      if (sum == 64'd19999900000) begin
        $display("TAG_PASS: sum = %0d", sum);
      end else begin
        $display("TAG_FAIL: sum = %0d", sum);
      end
    endtask
  endclass

  initial begin
    test::run();
    $finish;
  end
endmodule
