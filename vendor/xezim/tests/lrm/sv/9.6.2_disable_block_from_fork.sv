// top: rdf
module rdf;
  string lg[$];
  initial begin
    begin : blk
      fork
        begin #1 lg.push_back("H"); disable blk; end
        begin #3 lg.push_back("I"); end
      join
      lg.push_back("not_here");
    end
    #5 $display("T|r1|%p t=%0t", lg, $time);
  end
endmodule
