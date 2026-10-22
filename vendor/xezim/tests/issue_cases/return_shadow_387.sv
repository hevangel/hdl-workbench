module top;
 int scratch[2]='{2,3};
 int fifo[$]='{4,5};
 int data[]='{6,7};
 function automatic int scalar_shadow(); int scratch; scratch=17; return scratch; endfunction
 function automatic int queue_shadow(); int fifo; fifo=18; return fifo; endfunction
 function automatic int dynamic_shadow(); int data; data=19; return data; endfunction
 initial begin
  $display("T|shadow|%0d %0d %0d",scalar_shadow(),scratch[0],scratch[1]);
  $display("T|dynamic_shadow|%0d %0d %0d %0d %0d %0d",queue_shadow(),dynamic_shadow(),fifo[0],fifo[1],data[0],data[1]);
 end
endmodule
