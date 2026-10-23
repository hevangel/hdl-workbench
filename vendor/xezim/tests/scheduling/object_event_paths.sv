module pulse_source;
  event notice;
  event channels[2];
  int local_hits = 0;
  initial repeat (3) #10 ->notice;
  initial repeat (3) begin @notice; local_hits++; end
  initial #10 ->channels[1];
  initial #20 ->>channels[0];
endmodule

class completion_cell;
  bit ready;
  event arrival;
  task await_change(); @ready; endtask
  function void release_cell(); ready = 1; endfunction
  task await_notice(); @arrival; endtask
  task await_level(); wait(ready); endtask
endclass

class collection_client;
  int returned = 0;
  task park_event();
    completion_cell pending[$];
    completion_cell entry = new();
    pending.push_back(entry);
    pending[0].await_notice();
    returned++;
  endtask
  task park_level();
    completion_cell pending[$];
    completion_cell entry = new();
    pending.push_back(entry);
    pending[0].await_level();
    returned++;
  endtask
endclass

module top;
  pulse_source scalar();
  pulse_source bank[1]();
  completion_cell first = new(), second = new();
  collection_client client = new();
  int scalar_hits = 0, bank_hits = 0, first_hits = 0, second_hits = 0;
  int selected_hits = 0, deferred_hits = 0, qualified_hits = 0, selector = 0;
  initial begin
    fork
      forever begin @(scalar.notice); scalar_hits++; end
      forever begin @(bank[0].notice); bank_hits++; end
      begin first.await_change(); first_hits++; end
      begin @(second.ready); second_hits++; end
      client.park_event();
      client.park_level();
      begin @(scalar.channels[selector + 1]); selected_hits++; end
      begin @(scalar.channels[selector]); deferred_hits++; end
      repeat (3) begin @(top.bank[0].notice); qualified_hits++; end
    join_none
    #2;
    first.release_cell();
    second.ready = 1;
    #48;
    $display("T|hier=%0d,%0d prop=%0d,%0d blocked=%0d", scalar_hits, bank_hits,
             first_hits, second_hits, client.returned);
    $display("T|selected=%0d qualified=%0d", selected_hits, qualified_hits);
    $display("T|local=%0d,%0d deferred=%0d", scalar.local_hits, bank[0].local_hits, deferred_hits);
    $finish;
  end
endmodule
