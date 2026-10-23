// top: rsr
module rsr;
  string a = "abc";
  string r;
  initial begin
    r = {3{a}};
    $display("T|r1|[%s] %0d", r, r.len());
    $display("T|r2|[%s]", {2{a}});
    r = {2{"xy"}}; $display("T|r3|[%s]", r);
  end
endmodule
