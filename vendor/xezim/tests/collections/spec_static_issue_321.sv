
typedef struct { string scope; string field; int val; } entry_t;

// shape 1: static queue of a CLASS-scoped struct typedef, parameterized class
// (this is the exact shape the uvm_config_db stand-in had)
class box_p #(type T = int);
  typedef struct { string scope; string field; T val; } pentry_t;
  static pentry_t q[$];
  static function void put(string s, string f, T v);
    pentry_t e;
    e.scope = s; e.field = f; e.val = v;
    q.push_back(e);
  endfunction
endclass

// shape 2: static queue of a class-scoped struct typedef, plain class
class box_s;
  typedef struct { string scope; string field; int val; } sentry_t;
  static sentry_t q[$];
  static function void put(string s, string f, int v);
    sentry_t e;
    e.scope = s; e.field = f; e.val = v;
    q.push_back(e);
  endfunction
endclass

// shape 3: static queue of the FILE-scope struct inside a class
class box_f;
  static entry_t q[$];
  static function void put(string s, string f, int v);
    entry_t e;
    e.scope = s; e.field = f; e.val = v;
    q.push_back(e);
  endfunction
endclass

// shape 4 control: NON-static member queue of the same struct
class box_n;
  entry_t q[$];
  function void put(string s, string f, int v);
    entry_t e;
    e.scope = s; e.field = f; e.val = v;
    q.push_back(e);
  endfunction
endclass

module tb;
  // shape 5 control: module-scope queue of the file-scope struct
  entry_t mq[$];
  int errors;

  function automatic void check(string name, int want_size, string exp_scope,
                                int got_size, string got_scope,
                                string got_field, int got_val);
    if (got_size != want_size) begin
      $display("[ERROR] %s size: got %0d expected %0d", name, got_size, want_size);
      errors++;
    end else if (got_size == 0) begin
      $display("[ERROR] %s: entry missing", name);
      errors++;
    end else begin
      if (got_scope != exp_scope) begin
        $display("[ERROR] %s .scope: got [%s] expected [%s]", name, got_scope, exp_scope);
        errors++;
      end
      if (got_field != "x") begin
        $display("[ERROR] %s .field: got [%s] expected [x]", name, got_field);
        errors++;
      end
      if (got_val != 7) begin
        $display("[ERROR] %s .val: got %0d expected 7", name, got_val);
        errors++;
      end
      if (got_scope == exp_scope && got_field == "x" && got_val == 7)
        $display("[OK] %s", name);
    end
  endfunction

  initial begin
    entry_t e;
    box_n n = new();

    box_p#(int)::put("top.sub_env.IPW1*", "x", 7);
    check("static queue of struct, parameterized class", 1, "top.sub_env.IPW1*",
          box_p#(int)::q.size(), box_p#(int)::q[0].scope,
          box_p#(int)::q[0].field, box_p#(int)::q[0].val);

    box_s::put("top.sub_env.IPW2*", "x", 7);
    check("static queue of struct, plain class", 1, "top.sub_env.IPW2*",
          box_s::q.size(), box_s::q[0].scope,
          box_s::q[0].field, box_s::q[0].val);

    box_f::put("top.sub_env.IPW3*", "x", 7);
    check("static queue of file-scope struct, in class", 1, "top.sub_env.IPW3*",
          box_f::q.size(), box_f::q[0].scope,
          box_f::q[0].field, box_f::q[0].val);

    n.put("top.sub_env.IPW4*", "x", 7);
    check("non-static member queue of struct (control)", 1, "top.sub_env.IPW4*",
          n.q.size(), n.q[0].scope, n.q[0].field, n.q[0].val);

    e.scope = "top.sub_env.IPW5*"; e.field = "x"; e.val = 7;
    mq.push_back(e);
    check("module-scope queue of struct (control)", 1, "top.sub_env.IPW5*",
          mq.size(), mq[0].scope, mq[0].field, mq[0].val);

    if (errors == 0)
      $display("TEST PASSED");
    else
      $display("TEST FAILED with %0d errors", errors);
    $finish;
  end
endmodule

