module top;
function automatic void probe(input int unsigned value);$display("FUNCTION %0d",-value>1.0);endfunction
localparam int unsigned UP=1;int unsigned value;
initial begin value=1;$display("MODULE %0d",-UP>1.0);probe(value);$finish;end
endmodule
