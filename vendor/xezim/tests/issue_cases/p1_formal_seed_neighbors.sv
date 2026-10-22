module top;
  typedef byte signed_byte;
  int out_value, inout_value, task_value;
  int a[2];
  string text;
  class Holder; int value=13; endclass
  Holder object_value;
  task automatic output_byte(output byte x); x=16'h01f0; endtask
  task automatic inout_byte(inout byte x); x=16'h01f0; endtask
  task automatic typedef_input(input signed_byte x, output int result);
    x=16'h01f0; result=x;
  endtask
  function automatic int typedef_formal(input signed_byte x);
    x=16'h01f0; return x;
  endfunction
  function automatic byte implicit_byte();
    implicit_byte=8'hf0; implicit_byte >>>= 1;
  endfunction
  class ReturnSeed;
    function byte implicit_method();
      implicit_method=8'hf0; implicit_method >>>= 1;
    endfunction
  endclass
  task automatic keep_text(inout string x); x={x,"!"}; endtask
  task automatic keep_handle(inout Holder x); x.value=17; endtask
  task automatic keep_array(inout int x[2]); x[1]=7; endtask
  initial begin
    ReturnSeed r;
    int method_value;
    out_value=0; inout_value=0;
    output_byte(out_value); inout_byte(inout_value);
    typedef_input(32'd0,task_value);
    r=new;
    for(int i=0;i<1200;i++) method_value=r.implicit_method();
    text="ok"; keep_text(text);
    object_value=new; keep_handle(object_value);
    a[0]=3; a[1]=4; keep_array(a);
    $display("T|output-byte %0d",out_value);
    $display("T|inout-byte %0d",inout_value);
    $display("T|task-typedef %0d",task_value);
    $display("T|function-typedef %0d",typedef_formal(32'd0));
    $display("T|function-return %0d",implicit_byte());
    $display("T|method-return %0d",method_value);
    $display("T|text %s",text);
    $display("T|handle %0d",object_value.value);
    $display("T|array %0d %0d",a[0],a[1]);
    $finish;
  end
endmodule
