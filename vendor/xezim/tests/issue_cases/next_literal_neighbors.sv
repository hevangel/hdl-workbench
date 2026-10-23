module top;
string a="z", b="aa";
logic r1,r2,r3; assign r1=("z"<"aa"); assign r2=("aa">"z"); assign r3=(("z")<16'h6161);
initial #1 begin
 if (r1 && ("z"<"aa")) $display("PASS literal less"); else $display("FAIL literal less");
 if (r2 && ("aa">"z")) $display("PASS literal greater"); else $display("FAIL literal greater");
 if (r3 && (("z")<16'h6161)) $display("PASS numeric context"); else $display("FAIL numeric context");
 if (!(a<b) && !(a<"aa")) $display("PASS string context"); else $display("FAIL string context");
 if (""<"a" && "z"<="aa" && "aa">="z") $display("PASS ordering variants"); else $display("FAIL ordering variants");
 if ("abcdefghijklmnopq">"z") $display("PASS wide literal"); else $display("FAIL wide literal");
 $finish;
end
endmodule
