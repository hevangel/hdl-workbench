// top: rsr2
module rsr2;
  initial begin
    string a = "abc";
    string r;
    r = {3{a}};
    $display("T|r1|[%s] %0d", r, r.len());
  end
endmodule
