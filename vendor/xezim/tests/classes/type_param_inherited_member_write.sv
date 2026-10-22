`timescale 1ns/1ns
module top;
    class item;
        logic [63:0] rdata;
        function new(); endfunction
    endclass
    class item2 extends item;
        function new(); super.new(); endfunction
    endclass

    // V1: two independent params, no default
    class b1 #(type P = item, type Q = item2);
        P req;
    endclass
    class d1 extends b1 #(item);
        task drive(input logic [63:0] v); req.rdata = v; endtask
    endclass
    // V2: single param
    class b2 #(type P = item);
        P req;
    endclass
    class d2 #(type T = item) extends b2 #(T);
        task drive(input logic [63:0] v); req.rdata = v; endtask
    endclass
    // V3: two params with default RSP=REQ, extends with CONCRETE type
    class b3 #(type REQ = item, type RSP = REQ);
        REQ req;
    endclass
    class d3 extends b3 #(item);
        task drive(input logic [63:0] v); req.rdata = v; endtask
    endclass
    // V4: plain (non-param) parent with typed member via typedef
    class b4;
        item req;
    endclass
    class d4 extends b4;
        task drive(input logic [63:0] v); req.rdata = v; endtask
    endclass

    d1 v1; d2 #(item) v2; d3 v3; d4 v4;
    initial begin
        v1 = new(); v1.req = new(); v1.drive(64'h11); $display("V1 %h", v1.req.rdata);
        v2 = new(); v2.req = new(); v2.drive(64'h22); $display("V2 %h", v2.req.rdata);
        v3 = new(); v3.req = new(); v3.drive(64'h33); $display("V3 %h", v3.req.rdata);
        v4 = new(); v4.req = new(); v4.drive(64'h44); $display("V4 %h", v4.req.rdata);
        if (v1.req.rdata==64'h11 && v2.req.rdata==64'h22 && v3.req.rdata==64'h33 && v4.req.rdata==64'h44)
            $display("TAG_PASS"); else $display("TAG_FAIL");
    end
endmodule
