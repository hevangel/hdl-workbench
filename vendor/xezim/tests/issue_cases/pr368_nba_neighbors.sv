module top;
  typedef struct packed { logic [8:1] hi; bit [0:3] lo; } packed_t;
  int errors = 0;
  logic [7:0] value = 8'hAA;
  task automatic check(input string label, input logic [63:0] got, want);
    if (got !== want) begin errors++; $display("FAIL %s got=%h want=%h", label, got, want); end
    else $display("PASS %s", label);
  endtask
  task ordering;
    logic [7:0] value;
    value = 0;
    value[3:0] <= 5;
    check("active local", value, 0);
    #0 check("inactive local", value, 0);
    value[7:4] = 4'hA;
    #1 check("commit preserves blocking bits", value, 8'hA5);
  endtask
  task frozen;
    logic [7:0] bits;
    int index;
    bits = 0; index = 2;
    bits[index +: 2] <= 2'b11;
    index = 0;
    bits[index] <= 1;
    bits[3] <= 0;
    #1 check("frozen indices and source order", bits, 8'h05);
  endtask
  task ascending;
    logic [0:7] bits;
    bits = 0;
    bits[0 +: 4] <= 4'hA;
    bits[7] <= 1;
    #1 check("ascending local labels", bits, 8'hA1);
  endtask
  task members;
    packed_t data;
    data = 0;
    data.hi[4:1] <= 4'hB;
    data.lo[0] <= 1;
    #0 check("member writes remain deferred", data, 0);
    #1 check("member write labels", data, 12'h0B8);
  endtask
  task after_return(input bit schedule);
    logic [7:0] bits;
    if (schedule) begin bits = 0; bits[3:0] <= 4'h9; end
    else check("target survives task return", bits, 9);
  endtask
  task scopes;
    begin : first_block
      logic [7:0] bits;
      bits = 0; bits[3:0] <= 3;
      #1 check("first block owns its NBA", bits, 3);
    end
    begin : second_block
      logic [7:0] bits;
      bits = 0; bits[7:4] <= 4'hA;
      #1 check("second block owns its NBA", bits, 8'hA0);
    end
  endtask
  task delayed;
    logic [7:0] bits;
    bits = 0;
    bits[3:0] <= #2 4'h6;
    #1 check("future local NBA stays pending", bits, 0);
    #1;
    #0 check("future NBA waits through inactive", bits, 0);
    #1 check("future local NBA commits", bits, 6);
  endtask
  class C;
    task run;
      static logic [7:0] bits;
      static packed_t record;
      record = 0; record.hi[4:1] <= 4'hB; record.lo[0] <= 1;
      check("method static member active", record, 0);
      bits = 0; bits[3:0] <= 5;
      check("method static local active", bits, 0);
      #0 check("method static local inactive", bits, 0);
      #1 check("method static local committed", bits, 5);
      check("method static member committed", record, 12'h0B8);
      bits[7:4] <= #2 4'hA;
      #1 check("method future NBA pending", bits, 5);
      #2 check("method future NBA committed", bits, 8'hA5);
    endtask
  endclass
  initial begin
    C object_handle;
    ordering; check("hidden module unchanged", value, 8'hAA);
    frozen; ascending; members;
    after_return(1); #1; after_return(0);
    scopes; delayed;
    object_handle = new; object_handle.run;
    if (errors == 0) $display("ALL PASS");
    $finish;
  end
endmodule
