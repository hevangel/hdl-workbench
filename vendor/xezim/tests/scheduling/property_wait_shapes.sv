class status_cell;
  bit ready;
  event pulse;
  task await_ready(); @((ready)); endtask
  task await_pulse(); @((pulse)); endtask
endclass

class status_holder;
  status_cell child = new();
endclass

module top;
  status_holder box = new();
  status_cell cells[2];
  int selected = 0;
  int nested_time = -1, indexed_time = -1;
  int bare_flag_time = -1, bare_pulse_time = -1;
  int outer_flag_time = -1, outer_pulse_time = -1;
  event first, second, alias_event;
  int first_hits = 0, second_hits = 0;
  initial begin
    cells[0] = new();
    cells[1] = new();
    alias_event = first;
    fork
      begin @(box.child.ready); nested_time = $time; end
      begin @(cells[selected].ready); indexed_time = $time; end
      begin cells[0].await_ready(); bare_flag_time = $time; end
      begin cells[0].await_pulse(); bare_pulse_time = $time; end
      begin @((box.child.ready)); outer_flag_time = $time; end
      begin @((box.child.pulse)); outer_pulse_time = $time; end
      begin @first; first_hits++; end
      begin @second; second_hits++; end
    join_none
    #1;
    cells[1].ready = 1;
    box.child.ready = 0;
    #1;
    box.child.ready = 1;
    cells[0].ready = 1;
    ->cells[0].pulse;
    ->box.child.pulse;
    ->>alias_event;
    alias_event = second;
    #2;
    $display("T|properties=%0d,%0d", nested_time, indexed_time);
    $display("T|paren=%0d,%0d,%0d,%0d", bare_flag_time, bare_pulse_time, outer_flag_time, outer_pulse_time);
    $display("T|deferred=%0d,%0d", first_hits, second_hits);
    $finish;
  end
endmodule
