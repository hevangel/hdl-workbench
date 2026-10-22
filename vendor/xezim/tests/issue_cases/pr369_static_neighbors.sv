module top;
  typedef logic [1:0][7:0] packed_t;
  typedef struct packed { logic [8:1] hi; logic [0:3] lo; } struct_t;
  typedef union packed { logic [15:0] raw; packed_t bytes; } union_t;
  typedef struct packed { bit [3:0] two; logic [3:0] four; } mixed_t;
  int errors = 0;
  task automatic check(input string label, input logic [63:0] got, want);
    if (got !== want) begin errors++; $display("FAIL %s got=%h want=%h", label, got, want); end
    else $display("PASS %s", label);
  endtask
  task automatic vector_case;
    static packed_t bits;
    bits = 0; bits[1][3:0] = 9;
    check("explicit static packed vector", bits, 16'h0900);
    #1 check("static vector survives wait", bits, 16'h0900);
  endtask
  task automatic struct_case;
    static struct_t data;
    data = 0; data.hi[4:1] = 4'hB; data.lo[0] = 1;
    check("static struct declared labels", data, 12'h0B8);
  endtask
  task automatic union_case;
    static union_t data;
    data = 0; data.bytes[1][3:0] = 4'hA;
    check("static union nested selection", data.raw, 16'h0A00);
  endtask
  task automatic recursive(input int depth);
    static packed_t bits;
    if (depth == 0) begin bits = 0; recursive(1); check("recursive static reads live cell", bits, 16'h0900); end
    else bits[1][3:0] = 9;
  endtask
  task automatic mixed_case;
    mixed_t data;
    data = 0; data.two[3:0] = 4'bx1z0; data.four[3:0] = 4'bx1z0;
    check("mixed member two state conversion", data.two, 4'b0100);
    check("mixed member four state preservation", data.four, 4'bx1z0);
  endtask
  initial begin
    vector_case; struct_case; union_case; recursive(0); mixed_case;
    if (errors == 0) $display("ALL PASS");
    $finish;
  end
endmodule
