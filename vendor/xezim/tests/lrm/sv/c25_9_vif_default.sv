// §25.9, §13.5.3: an omitted argument to a subroutine whose formal is a
// virtual-interface (vif) with a DEFAULT binds the formal to the default's
// interface — both for reading (`input`) and for copying an `inout` binding
// back to the caller. Regression for #469.
interface bus_if;
  logic [7:0] sig;
endinterface

module t;
  bus_if b();
  virtual bus_if v;

  // input vif formal with an omitted default: the body must read the
  // default interface's data.
  task automatic show(input virtual bus_if x = v);
    if (x == null) $display("T|25.9.vifdef|task-null");
    else $display("T|25.9.vifdef|task-read %0d", x.sig);
  endtask

  // inout vif formal with an omitted default: the binding must copy back to
  // the caller's vif variable rather than nulling it.
  task automatic keep(inout virtual bus_if y = v);
    if (y == null) $display("T|25.9.vifdef|inout-null");
    else $display("T|25.9.vifdef|inout-read %0d", y.sig);
  endtask

  // function vif formal with an omitted default.
  function automatic int fread(input virtual bus_if x = v);
    if (x == null) return -1;
    return int'(x.sig);
  endfunction

  class c;
    virtual bus_if cv;
    task show(input virtual bus_if x = v);
      if (x == null) $display("T|25.9.vifdef|method-null");
      else $display("T|25.9.vifdef|method-read %0d", x.sig);
    endtask
  endclass
  c cc = new;

  initial begin
    v = b;
    b.sig = 8'd42;

    // input default: reads 42.
    if (fread() != 42) $display("T|25.9.vifdef|fn-fail");
    else $display("T|25.9.vifdef|fn-read 42");
    show();

    // inout default: copy-back must leave `v` bound to `b`, so a later read
    // through `v` still sees the interface.
    b.sig = 8'd43;
    keep();
    if (v == null) $display("T|25.9.vifdef|after-inout-null");
    else $display("T|25.9.vifdef|after-inout %0d", v.sig);

    // class-method default: reads 42 (b.sig was set to 43 above, so expect 43).
    b.sig = 8'd44;
    cc.show();
    $finish;
  end
endmodule