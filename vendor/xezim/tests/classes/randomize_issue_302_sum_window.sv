// Issue #302: array sum constraints whose window centre is itself random.

module t;
        int v1 [(16 + 1)];
        int v0, v2;

        initial begin
                bit [15:0] p0;
                bit [1:0] p1;
                p1 = 3;
                p0 = 16'd256;
                foreach (v1[i]) begin
                        v1[i] = 1000 + (i % 37);
                end
                v2 = 17;
                // B1
                v0 = 17408;
                if (std::randomize(v0, v1) with {
                        v0 == (v2 * 32'(p0) * (p1 + 1));
                        v1.sum() with (32'(item)) inside { [v0 - 10:v0 + 10] };
                }) $display("T|B1 preassign+expr+varrange: PASS");
                else $display("T|B1 preassign+expr+varrange: FAIL");
                // B2
                if (std::randomize(v0, v1) with {
                        v0 == (v2 * 32'(p0) * (p1 + 1));
                        v1.sum() with (32'(item)) inside { [17398:17418] };
                }) $display("T|B2 expr+constrange: PASS");
                else $display("T|B2 expr+constrange: FAIL");
                // B3
                v0 = 17408;
                if (std::randomize(v1) with {
                        v1.sum() with (32'(item)) inside { [v0 - 10:v0 + 10] };
                }) $display("T|B3 center-fixed+varrange: PASS");
                else $display("T|B3 center-fixed+varrange: FAIL");
                // B4
                if (std::randomize(v0, v1) with {
                        v0 == 17408;
                        v1.sum() with (32'(item)) inside { [v0 - 10:v0 + 10] };
                }) $display("T|B4 literal+varrange: PASS");
                else $display("T|B4 literal+varrange: FAIL");
                foreach (v1[i]) begin
                        v1[i] = 1000 + (i % 37);
                end
                if (std::randomize(v0, v1) with {
                        v0 == 17408;
                        v1.sum() >= v0 - 10;
                        v1.sum() <= v0 + 10;
                }) $display("T|C2 sum in two items: PASS");
                else $display("T|C2 sum in two items: FAIL");

                $finish;
        end
endmodule

