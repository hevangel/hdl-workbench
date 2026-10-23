module leaf;
 for(genvar g=0;g<2;g++) begin: bank localparam int A[2]='{g+7,g+9}; end
endmodule
module top; leaf u(); initial begin $display("T|generate %0d %0d",u.bank[0].A[1],u.bank[1].A[1]); $finish; end endmodule
