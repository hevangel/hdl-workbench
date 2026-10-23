module top;
  integer iter;
  initial begin
    randcase
      0: $display("T|zero-weight-unreachable");
      1: begin : selected
        #1;
        $display("T|selected %m");
        disable selected;
        $display("T|selected-unreachable");
      end
    endcase
    $display("T|after-selected %m");
    randcase
      1: begin : declared
        integer value;
        value = 23;
        #1;
        $display("T|declared %m %0d", value);
        disable declared;
        $display("T|declared-unreachable");
      end
    endcase
    randcase
      1: begin : outer_scope
        begin : inner_scope
          #1;
          $display("T|inner %m");
          disable inner_scope;
          $display("T|inner-unreachable");
        end
        $display("T|outer %m");
      end
    endcase
    for (iter = 0; iter < 2; iter = iter + 1) begin
      randcase
        1: begin : repeated
          #1;
          $display("T|repeated %m %0d", iter);
          disable repeated;
          $display("T|repeated-unreachable");
        end
      endcase
    end
    case (1)
      1: begin : case_scope
        #1;
        $display("T|case %m");
        disable case_scope;
        $display("T|case-unreachable");
      end
    endcase
    randcase
      1: begin
        #1;
        $display("T|unnamed %m");
      end
    endcase
    $display("T|done %m");
  end
endmodule
