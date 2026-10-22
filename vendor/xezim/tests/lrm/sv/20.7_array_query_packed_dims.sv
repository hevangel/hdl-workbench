// top: faq
module faq;
  logic [7:0][3:0] pk [2:5][1:0];
  int ac[string];
  initial begin
    ac["a"] = 1;
    $display("T|20.7a|dims=%0d udims=%0d", $dimensions(pk), $unpacked_dimensions(pk));
    $display("T|20.7c|left3=%0d size3=%0d size4=%0d inc2=%0d size5=%0d", $left(pk, 3), $size(pk, 3), $size(pk, 4), $increment(pk, 2), $size(pk, 5));
    $display("T|20.7d|%0d", $size(ac));
  end
endmodule
