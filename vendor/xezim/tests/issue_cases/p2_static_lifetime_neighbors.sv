module static_unit(output int a, output int b);
  function int bump(); static int n=4; n++; return n; endfunction
  initial begin a=bump(); b=bump(); end
endmodule
module top;
  int errors, calls;
  int a0,b0,a1,b1;
  static_unit u0(a0,b0),u1(a1,b1);
  function int side(); calls++; return 20; endfunction
  function int never_called(); static int n=side(); return n; endfunction
  task automatic task_static(output int v); static int n=10; n++; v=n; endtask
  function automatic int arr_static(); static int a[2]='{3,4}; a[1]++; return a[1]; endfunction
  function int recursive(int n);
    int a[2];
    foreach(a[i]) a[i]=n ? 3 : 7;
    if(n) void'(recursive(0));
    return a[1];
  endfunction
  function int queue_a(); int q[$]; q.push_back(1); return q.size(); endfunction
  function int queue_b(); int q[$]; q.push_back(2); return q.size(); endfunction
  task static automatic_local(output int v); automatic int n=10; n++; v=n; endtask
  function int observed(); static int n=0; n++; return n; endfunction
  function automatic int fresh(); int a[2]; a[0]++; return a[0]; endfunction
  task automatic ck(string label,int got,int expected);
    if(got==expected) $display("PASS %s",label);
    else begin errors++; $display("FAIL %s got=%0d expected=%0d",label,got,expected); end
  endtask
  initial begin
    int v;
    #1;
    ck("uncalled initializer",calls,1);
    task_static(v); ck("automatic task static first",v,11);
    task_static(v); ck("automatic task static second",v,12);
    ck("array initializer first",arr_static(),5);
    ck("array initializer second",arr_static(),6);
    ck("recursive fully overwritten array",recursive(2),7);
    void'(queue_a()); ck("same-name queues a",queue_a(),2);
    ck("same-name queues b",queue_b(),1);
    void'(fresh()); ck("automatic fresh array",fresh(),1);
    ck("instance-owned static cells",a0==5 && b0==6 && a1==5 && b1==6,1);
    v = 0 & observed();
    ck("static side effects are observable",observed(),2);
    automatic_local(v); ck("automatic local first",v,11);
    automatic_local(v); ck("automatic local second",v,11);
    if(errors==0) $display("ALL PASS");
    $finish;
  end
endmodule
