// tops: topA topB (both uninstantiated)
module sub; initial #1 $display("T|23.3.1|%m"); endmodule
module topA; sub s(); initial #2 $display("T|23.3.1a|A sees B: %0d", $root.topB.bv); endmodule
module topB; int bv = 5; sub s(); initial #3 $display("T|23.3.1b|B up=%0d", $root.topB.bv); endmodule
