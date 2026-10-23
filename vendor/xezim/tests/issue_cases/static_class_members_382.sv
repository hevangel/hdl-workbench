// =============================================================================
// static_members_comprehensive_probes.sv
//
// Comprehensive MWE integrating BOTH cousin sweeps of xezim static-member
// defects:
//
//   sweep 1 (probes/,  47 probes) -> bugs A, B, C, D
//   sweep 2 (probes2/, 29 probes) -> new shapes of A, B, C + new bugs E, F
//
// Every "CTL" case below is a probe that xezim gets RIGHT in isolation; they
// are kept so a failure cannot be blamed on the probe being illegal or on
// the surrounding machinery.
//
//   BUG A -- BARE static->static task call is dispatched synchronously.
//            mailbox.get() in the inlined callee returns a stale value
//            instead of suspending.  Fails: bare, two-hop, inherited,
//            parameterized class, output arg, LOCAL mailbox, forever
//            consumer (busy-spins to the iteration cap), shadowed static
//            task.  Fine: qualified call, instance-task caller, #delay,
//            semaphore.get, mb.PEEK, fork/join, callee in a package, static
//            task invoked via an instance handle, direct call.
//
//   BUG B -- a BARE (unparenthesised) static-member reference X::m resolves
//            only when X is a LITERAL class name.  Alias / type-parameter /
//            imported / nested / forward-declared / cross-package qualifiers
//            silently yield the type DEFAULT ('' / 0).  The parenthesised
//            form of the same expression works (except BUG E below).
//
//   BUG C -- blocking on class-STATIC synchronization objects inside a
//            static task never wakes, on EVERY call path: @static_event,
//            wait(static prop), @(posedge/negedge hier), @(e1 or e2), ->>
//            trigger, and the same bodies in PARAMETERIZED classes.  The
//            identical bodies in an INSTANCE task suspend correctly.
//
//   BUG D -- merged-context interference: #delay inside a bare static call
//            resumes at the wrong time only when the whole family is
//            elaborated in one unit (passes in isolation).
//
//   BUG E -- NEW: a typedef CHAIN of depth 2 ending at a specialization
//            misbinds even the PARENTHESISED call: b_t::type_name() returns
//            the DEFAULT specialization's value ('<unknown>'), and the bare
//            form returns ''.
//
//   BUG F -- NEW: a BARE static-function reference through an INSTANCE
//            HANDLE (h.type_name) EXECUTES the function but binds the
//            DEFAULT specialization ('<unknown>'); the parenthesised
//            h.type_name() is correct.  (BUG B never dispatches at all;
//            BUG F dispatches with the wrong specialization -- a different
//            code path.)
//
// =============================================================================

`timescale 1ns/1ps

// -----------------------------------------------------------------------------
// Callee classes for BUG B (sweep 1)
// -----------------------------------------------------------------------------
class holder #(type T = int, string Tname = "<unknown>");
  static function string type_name();
    return Tname;
  endfunction
  static string label = "holder-label";
  static int    count = 7;
endclass

class plain_holder;
  static function string type_name();
    return "plain";
  endfunction
  static string label = "plain-label";
  static int    count = 21;
endclass

// callee whose only port carries a DEFAULT -> still "parameterless" per LRM
class holder_dp #(type T = int, string Tname = "<unknown>");
  static function string type_name(string suffix = "!");
    return {Tname, suffix};
  endfunction
endclass

// callee returning packed (non-string) types
class holder_int #(type T = int, int WID = 8);
  static function int  id();   return WID * 3; endfunction
  static function bit  flag(); return 1'b1;    endfunction
endclass

// callee parameterized with NON-TYPE parameters only
class width_holder #(int W = 4, string NM = "<wid>");
  static function string type_name();
    return {NM, $sformatf("(%0d)", W)};
  endfunction
endclass

// callee reached through inheritance
class base_h;
  static function string type_name();
    return "from-base";
  endfunction
endclass
class derived_h extends base_h;
endclass

// callee declared in a PARAMETERIZED ancestor
class bbase #(string N = "<base>");
  static function string type_name();
    return N;
  endfunction
endclass
class bmid extends bbase#("mid-name");
endclass

package pkg;
  class holder #(type T = int, string Tname = "<unknown>");
    static function string type_name();
      return Tname;
    endfunction
  endclass
  typedef holder #(int, "pkg_spec") h_t;
endpackage

typedef plain_holder file_h_t;             // FILE-scope alias

// -----------------------------------------------------------------------------
// Caller classes for BUG B (sweep 1) -- same bare reference shape, only the
// qualifier differs.
// -----------------------------------------------------------------------------
class c_bare_param #(type Tr = int);                 // qualifier = type param
  virtual function string f(); return Tr::type_name; endfunction
endclass
class c_paren_param #(type Tr = int);                // CTL: parenthesised
  virtual function string f(); return Tr::type_name(); endfunction
endclass
class c_bare_prop #(type Tr = int);                  // qualifier = type param
  virtual function string lab(); return Tr::label; endfunction
  virtual function int    cnt(); return Tr::count; endfunction
endclass
class c_bare_defaulted #(type Tr = int);             // callee has defaulted port
  virtual function string f(); return Tr::type_name; endfunction
endclass
class c_bare_intret #(type Tr = int);                // callee returns int/bit
  virtual function int f(); return Tr::id;   endfunction
  virtual function int g(); return Tr::flag; endfunction
endclass
class c_bare_static_caller #(type Tr = int);         // caller is a STATIC fn
  static function string f(); return Tr::type_name; endfunction
endclass
class c_alias_member;                                // qualifier = member typedef
  typedef plain_holder type_id;
  virtual function string f();   return type_id::type_name; endfunction
  virtual function string lab(); return type_id::label;     endfunction
  virtual function int    cnt(); return type_id::count;     endfunction
endclass
class c_alias_file;                                  // qualifier = file-scope typedef
  virtual function string f(); return file_h_t::type_name; endfunction
endclass
class c_alias_pkg;                                   // qualifier = package typedef
  virtual function string f(); return pkg::h_t::type_name; endfunction
endclass
class c_literal_in_class;                            // CTL: literal spec in class
  virtual function string f();
    return holder #(int, "literal_spec")::type_name;
  endfunction
endclass

// -----------------------------------------------------------------------------
// Caller/callee classes for BUG B -- sweep-2 shapes
// -----------------------------------------------------------------------------
class n_defbind #(type Tr = holder #(int, "default_spec"));  // DEFAULT binding
  virtual function string f(); return Tr::type_name; endfunction
endclass

class n_taskref #(type Tr = int);                    // bare ref inside a TASK
  task cache_name(output string s);
    s = Tr::type_name;
  endtask
endclass

class n_dfltarg #(type Tr = int);                    // bare ref as DEFAULT ARG
  virtual function string greet(string prefix = Tr::type_name);
    return prefix;
  endfunction
endclass

class n_wrap #(type T = int);                        // member typedef of own param
  typedef T t_id;
  virtual function string f(); return t_id::type_name; endfunction
endclass

class n_ctor #(type Tr = int);                       // bare ref in CONSTRUCTOR
  string cached;
  function new();
    cached = Tr::type_name;
  endfunction
endclass

class n_fwd_caller #(type Tr = int);                 // callee declared LATER...
  virtual function string f(); return Tr::type_name; endfunction
endclass
class n_late_holder #(string N = "<late>");          // ...than this class
  static function string type_name();
    return N;
  endfunction
endclass

class n_outer;                                       // NESTED class callee
  class n_inner #(string N = "<inner>");
    static function string type_name();
      return N;
    endfunction
  endclass
endclass

class n_litparam #(type Tr = int);                   // CTL: literal spec, param actual
  virtual function string f();
    return holder #(Tr, "param_actual")::type_name;
  endfunction
endclass

// BUG E: typedef CHAIN of depth two ending at a specialization
typedef holder #(int, "chain_spec") n_a_t;
typedef n_a_t n_b_t;

// -----------------------------------------------------------------------------
// Cross-package alias (sweep 2, n44)
// -----------------------------------------------------------------------------
package n_pk_a;
  class holder #(type T = int, string Tname = "<unknown>");
    static function string type_name();
      return Tname;
    endfunction
  endclass
  typedef holder #(int, "xpkg_spec") h_t;
endpackage
package n_pk_b;
  import n_pk_a::h_t;
  class user;
    static function string get();
      return h_t::type_name;              // BARE, cross-package import
    endfunction
  endclass
endpackage

// -----------------------------------------------------------------------------
// $unit-scope function with a bare reference (sweep 2, n13 -- CTL)
// -----------------------------------------------------------------------------
function string n_unit_fn();
  return holder #(int, "unit_spec")::type_name;
endfunction

// -----------------------------------------------------------------------------
// Classes for BUG A -- bare static->static task call vs. the controls.
// -----------------------------------------------------------------------------
class a_bare_mb;                                     // BUG A: BARE
  static mailbox #(int) mb = new();
  static int got[$];
  static task leaf();
    int v;
    repeat (2) begin mb.get(v); got.push_back(v); end
  endtask
  static task caller();
    leaf();
  endtask
endclass

class a_qual_mb;                                     // CTL: QUALIFIED
  static mailbox #(int) mb = new();
  static int got[$];
  static task leaf();
    int v;
    repeat (2) begin mb.get(v); got.push_back(v); end
  endtask
  static task caller();
    a_qual_mb::leaf();
  endtask
endclass

class a_two_hops;                                    // BUG A: two bare hops
  static mailbox #(int) mb = new();
  static int got[$];
  static bit mid_resumed;
  static task leaf();
    int v; mb.get(v); got.push_back(v);
  endtask
  static task mid();
    leaf();  mid_resumed = 1'b1;
  endtask
  static task caller();
    mid();
  endtask
endclass

class a_base;                                        // BUG A: inherited callee
  static mailbox #(int) mb = new();
  static int got[$];
  static task leaf();
    int v; mb.get(v); got.push_back(v);
  endtask
endclass
class a_derived extends a_base;
  static task caller();
    leaf();
  endtask
endclass

class a_param #(string TAG = "t");                   // BUG A: parameterized class
  static mailbox #(int) mb = new();
  static int got[$];
  static task leaf();
    int v; mb.get(v); got.push_back(v);
  endtask
  static task caller();
    leaf();
  endtask
endclass

class a_out;                                         // BUG A: output argument
  static mailbox #(int) mb = new();
  static int seen = -999;
  static task leaf(output int o);
    int v; mb.get(v); o = v * 2;
  endtask
  static task caller();
    int o; o = -1; leaf(o); seen = o;
  endtask
endclass

class a_delay;                                       // CTL: #delay suspends
  static int t_before, t_after, t_resumed;
  static task leaf();
    t_before = $time; #10; t_after = $time;
  endtask
  static task caller();
    leaf(); t_resumed = $time;
  endtask
endclass

class a_sem;                                         // CTL: semaphore.get suspends
  static semaphore s = new(1);
  static int hits, t_done;
  static task leaf();
    s.get(1); hits++; #10; s.put(1);
  endtask
  static task caller();
    leaf(); leaf(); t_done = $time;
  endtask
endclass

// ---- BUG A sweep-2 shapes ----------------------------------------------------

class n_localmb;                                     // BUG A: LOCAL mailbox
  static int got[$];                                 // (nothing ever put)
  static task leaf();
    mailbox #(int) mb = new();         // LOCAL, stays empty
    int v;
    mb.get(v);                         // must suspend forever
    got.push_back(v);
  endtask
  static task caller();
    leaf();
  endtask
endclass

class n_peek;                                        // CTL: mb.PEEK suspends
  static mailbox #(int) mb = new();
  static int got[$];
  static task leaf();
    int v;
    mb.peek(v);                        // blocks without consuming
    got.push_back(v);
  endtask
  static task caller();
    leaf();
  endtask
endclass

class n_forever;                                     // BUG A: forever consumer
  static mailbox #(int) mb = new();    // (the UVM shape)
  static int got[$];
  static task leaf();
    int v;
    forever begin
      mb.get(v);
      got.push_back(v);
    end
  endtask
  static task caller();
    leaf();
  endtask
endclass

class n_sh_base;                                     // BUG A: SHADOWED task
  static mailbox #(int) mb = new();
  static int got[$];
  static task leaf();                  // shadowed by n_sh_derived
    int v; mb.get(v); got.push_back(-1);
  endtask
endclass
class n_sh_derived extends n_sh_base;
  static task leaf();                  // shadows n_sh_base::leaf
    int v; mb.get(v); got.push_back(v);
  endtask
  static task caller();
    leaf();                            // must resolve to n_sh_derived::leaf
  endtask
endclass

class n_hdl_task;                                    // CTL: static task via handle
  static mailbox #(int) mb = new();
  static int got[$];
  static task leaf();
    int v; mb.get(v); got.push_back(v);
  endtask
endclass

package n_pk_c;                                      // CTL: callee in a package
  class c;
    static mailbox #(int) mb = new();
    static int got[$];
    static task leaf();
      int v; mb.get(v); got.push_back(v);
    endtask
    static task caller();
      leaf();                          // BARE, both inside the package
    endtask
  endclass
endpackage

// -----------------------------------------------------------------------------
// Classes for BUG C -- static sync objects inside a static task.
// -----------------------------------------------------------------------------
class c_event;                                       // BUG C: @static event
  static event e1;
  static int hits;
  static int t_wake = -1;
  static task leaf();
    @e1; hits++; t_wake = $time;
  endtask
  static task caller();
    c_event::leaf();
  endtask
endclass

class c_wait;                                        // BUG C: wait(static prop)
  static bit flag = 1'b0;
  static int hits;
  static int t_wake = -1;
  static task leaf();
    wait (flag == 1'b1); hits++; t_wake = $time;
  endtask
  static task caller();
    c_wait::leaf();
  endtask
endclass

class c_posedge;                                     // BUG C: @(posedge hier sig)
  static int hits;
  static int t_wake = -1;
  static task leaf();
    @(posedge tb_top.clk); hits++; t_wake = $time;
  endtask
  static task caller();
    c_posedge::leaf();
  endtask
endclass

class c_inst;                                        // CTL: instance-task twins
  event e1;
  bit   flag = 1'b0;
  int   hits_ev, hits_wt;
  int   t_wake_ev = -1, t_wake_wt = -1;
  task wait_ev(); @e1;              hits_ev++; t_wake_ev = $time; endtask
  task wait_wt(); wait (flag == 1); hits_wt++; t_wake_wt = $time; endtask
endclass

// ---- BUG C sweep-2 shapes ----------------------------------------------------

class n_negedge;                                     // BUG C: @(negedge hier)
  static int hits;
  static int t_wake = -1;
  static task leaf();
    @(negedge tb_top.clk); hits++; t_wake = $time;
  endtask
  static task caller();
    leaf();
  endtask
endclass

class n_orev;                                        // BUG C: @(e1 or e2)
  static event e1, e2;
  static int hits;
  static int t_wake = -1;
  static task leaf();
    @(e1 or e2); hits++; t_wake = $time;
  endtask
  static task caller();
    leaf();
  endtask
endclass

class n_nbtrig;                                      // BUG C: ->> trigger
  static event e1;
  static int hits;
  static int t_wake = -1;
  static task leaf();
    @e1; hits++; t_wake = $time;
  endtask
  static task caller();
    leaf();
  endtask
endclass

class n_pev #(string TAG = "t");                     // BUG C: param class @event
  static event e1;
  static int hits;
  static int t_wake = -1;
  static task leaf();
    @e1; hits++; t_wake = $time;
  endtask
  static task caller();
    n_pev::leaf();                     // QUALIFIED
  endtask
endclass

class n_pwait #(string TAG = "t");                   // BUG C: param class wait
  static bit flag = 1'b0;
  static int hits;
  static int t_wake = -1;
  static task leaf();
    wait (flag == 1'b1); hits++; t_wake = $time;
  endtask
  static task caller();
    n_pwait::leaf();                   // QUALIFIED
  endtask
endclass

// -----------------------------------------------------------------------------
// Static-init ordering (sweep 2, n43 -- CTL)
// -----------------------------------------------------------------------------
class n_initprop #(string N = "<n>");
  static function string make();
    return N;
  endfunction
  static string label = make();        // static init calls static fn
endclass

// -----------------------------------------------------------------------------
// Helper modules (sweep 2)
// -----------------------------------------------------------------------------
module n_mp #(type T = int);                         // MODULE type parameter
  function string get_type_name();
    return T::type_name;               // BARE, module type param
  endfunction
endmodule

module n_modm;                                       // CTL: class IN a module
  class holder #(string N = "<mod>");
    static function string type_name();
      return N;
    endfunction
  endclass
  string s_bare, s_paren;
  initial begin
    s_bare  = holder #("in_mod")::type_name;    // BARE at module scope
    s_paren = holder #("in_mod")::type_name();  // PAREN control
  end
endmodule

typedef a_param #("p") ap_t;                         // parameterized BUG-A class

// =============================================================================
module tb_top;
  import pkg::h_t;                                   // for check B20 (n03)
  bit clk;
  int nchecks, nfails;

  c_bare_param         #(holder #(int, "base_test")) b01 = new();
  c_paren_param        #(holder #(int, "base_test")) b02 = new();
  c_bare_param         #(plain_holder)               b03 = new();
  c_bare_prop          #(holder #(int, "base_test")) b04 = new();
  c_bare_prop          #(plain_holder)               b05 = new();
  c_bare_defaulted     #(holder_dp #(int, "base_test")) b06 = new();
  c_bare_intret        #(holder_int #(int, 14))      b07 = new();
  c_bare_param         #(width_holder #(16, "w16"))  b08 = new();
  c_bare_param         #(derived_h)                  b09 = new();
  c_bare_param         #(bmid)                       b10 = new();
  c_alias_member                                     b11 = new();
  c_alias_file                                       b12 = new();
  c_alias_pkg                                        b13 = new();
  c_literal_in_class                                 b14 = new();
  c_bare_param         #(holder #(int, "ALPHA"))     b16a = new();
  c_bare_param         #(holder #(int, "BETA"))      b16b = new();
  c_bare_param         #(holder #(bit,  "GAMMA"))    b16c = new();

  // sweep-2 BUG B / E / F instances
  n_defbind                                         b18 = new();
  n_taskref          #(holder #(int, "task_spec"))  b23 = new();
  n_dfltarg          #(holder #(int, "dflt_arg"))   b24 = new();
  n_wrap             #(holder #(int, "wrap_spec"))  b25 = new();
  n_ctor             #(holder #(int, "ctor_spec"))  b22 = new();
  n_fwd_caller       #(n_late_holder #("late_spec")) b26 = new();
  n_litparam         #(int)                         b28 = new();
  holder #(int, "handle_spec")                      n_h = new();   // BUG F

  // sweep-2 helper-module instances
  n_mp  #(holder #(int, "mod_spec"))                n_mp_u ();
  n_modm                                            n_mod_u();

  c_inst                ci = new();
  n_hdl_task            n_htu = new();

  always #5 clk = ~clk;

  function void ck_s(input string name, input string got, input string want);
    nchecks++;
    if (got == want) $display("| PASS %s", name);
    else begin nfails++; $display("| FAIL %s: got '%s' want '%s'", name, got, want); end
  endfunction

  function void ck_i(input string name, input int got, input int want);
    nchecks++;
    if (got == want) $display("| PASS %s: %0d", name, got);
    else begin nfails++; $display("| FAIL %s: got %0d want %0d", name, got, want); end
  endfunction

  // ---- one driver + stimulus pair per case, exactly as in the standalone
  //      probes (a single shared 12-way fork perturbs the scheduling) ---------
  initial begin : a01_drv
    fork begin a_bare_mb::caller(); end join_none
  end
  initial begin : a01_stim
    #10 a_bare_mb::mb.put(42);
    #10 a_bare_mb::mb.put(43);
  end

  initial begin : a02_drv
    fork begin a_qual_mb::caller(); end join_none
  end
  initial begin : a02_stim
    #10 a_qual_mb::mb.put(42);
    #10 a_qual_mb::mb.put(43);
  end

  initial begin : a03_drv
    fork begin a_two_hops::caller(); end join_none
  end
  initial begin : a03_stim
    #10 a_two_hops::mb.put(7);
  end

  initial begin : a04_drv
    fork begin a_derived::caller(); end join_none
  end
  initial begin : a04_stim
    #10 a_base::mb.put(9);
  end

  initial begin : a05_drv
    fork begin ap_t::caller(); end join_none
  end
  initial begin : a05_stim
    #10 ap_t::mb.put(5);
  end

  initial begin : a06_drv
    fork begin a_out::caller(); end join_none
  end
  initial begin : a06_stim
    #10 a_out::mb.put(42);
  end

  initial begin : a07_drv
    fork begin a_delay::caller(); end join_none
  end

  initial begin : a08_drv
    fork begin a_sem::caller(); end join_none
  end

  initial begin : c01_drv
    fork begin c_event::caller(); end join_none
  end
  initial begin : c01_stim
    #10 -> c_event::e1;
  end

  initial begin : c02_drv
    fork begin c_wait::caller(); end join_none
  end
  initial begin : c02_stim
    #10 c_wait::flag = 1'b1;
  end

  initial begin : c03_drv
    fork begin c_posedge::caller(); end join_none
  end

  initial begin : c04_drv
    fork
      ci.wait_ev();
      ci.wait_wt();
    join_none
  end
  initial begin : c04_stim
    #10 -> ci.e1;
    #10 ci.flag = 1'b1;
  end

  // ---- sweep-2 drivers -------------------------------------------------------
  initial begin : a09_drv                            // n20: local mailbox
    fork begin n_localmb::caller(); end join_none
  end                                                // no stimulus: box stays
                                                     // empty, get must block

  initial begin : a10_drv                            // n21: mb.peek (CTL)
    fork begin n_peek::caller(); end join_none
  end
  initial begin : a10_stim
    #10 n_peek::mb.put(42);
  end

  initial begin : a11_drv                            // n23: pkg callee (CTL)
    fork begin n_pk_c::c::caller(); end join_none
  end
  initial begin : a11_stim
    #10 n_pk_c::c::mb.put(42);
  end

  initial begin : a12_drv                            // n24: static task via
    fork begin n_htu.leaf(); end join_none          // instance handle (CTL)
  end
  initial begin : a12_stim
    #10 n_hdl_task::mb.put(42);
  end

  initial begin : a13_drv                            // n25: forever consumer
    fork begin n_forever::caller(); end join_none
  end
  initial begin : a13_stim
    #10 n_forever::mb.put(42);
    #10 n_forever::mb.put(43);
  end

  initial begin : a14_drv                            // n27: shadowed static task
    fork begin n_sh_derived::caller(); end join_none
  end
  initial begin : a14_stim
    #10 n_sh_base::mb.put(42);
  end

  initial begin : c05_drv                            // n26: @(negedge clk)
    fork begin n_negedge::caller(); end join_none
  end

  initial begin : c06_drv                            // n28: @(e1 or e2)
    fork begin n_orev::caller(); end join_none
  end
  initial begin : c06_stim
    #10 -> n_orev::e2;                    // only the SECOND arm fires
  end

  initial begin : c07_drv                            // n40: param-class @event
    fork begin n_pev #("p")::caller(); end join_none
  end
  initial begin : c07_stim
    #10 -> n_pev #("p")::e1;
  end

  initial begin : c08_drv                            // n41: param-class wait
    fork begin n_pwait #("p")::caller(); end join_none
  end
  initial begin : c08_stim
    #10 n_pwait #("p")::flag = 1'b1;
  end

  initial begin : c09_drv                            // n42: ->> trigger
    fork begin n_nbtrig::caller(); end join_none
  end
  initial begin : c09_stim
    #10 ->> n_nbtrig::e1;                // nonblocking trigger
  end

  // ---- self-check -------------------------------------------------------------
  initial begin : chk
    #200;
    $display("| ---- BUG B: bare (unparenthesised) static-member reference ----");
    ck_s("B01 bare, type param -> parameterized class", b01.f(), "base_test");
    ck_s("B02 CTL parenthesised, same binding",         b02.f(), "base_test");
    ck_s("B03 CTL bare, plain-class binding",           b03.f(), "plain");
    ck_s("B04 bare static PROPERTY via type param",     b04.lab(), "holder-label");
    ck_i("B04b bare static PROPERTY (int) via type param", b04.cnt(), 7);
    ck_s("B05 CTL bare static PROPERTY via plain class", b05.lab(), "plain-label");
    ck_i("B05b CTL bare static PROPERTY (int)",          b05.cnt(), 21);
    ck_s("B06 bare, callee has a defaulted port",        b06.f(), "base_test!");
    ck_i("B07 bare, callee returns int",                 b07.f(), 42);
    ck_i("B07b bare, callee returns bit",                b07.g(), 1);
    ck_s("B08 bare, callee has non-type params only",    b08.f(), "w16(16)");
    ck_s("B09 CTL bare, fn inherited from plain base",   b09.f(), "from-base");
    ck_s("B10 CTL bare, fn in parameterized ancestor",   b10.f(), "mid-name");
    ck_s("B11 bare via class-member typedef alias",      b11.f(), "plain");
    ck_s("B11b bare static PROPERTY via member alias",   b11.lab(), "plain-label");
    ck_i("B11c bare static PROPERTY (int) via alias",    b11.cnt(), 21);
    ck_s("B12 bare via file-scope typedef alias",        b12.f(), "plain");
    ck_s("B13 bare via package-scope typedef alias",     b13.f(), "pkg_spec");
    ck_s("B14 CTL bare, literal spec inside a class",    b14.f(), "literal_spec");
    ck_s("B15 bare from a STATIC caller function",
         c_bare_static_caller #(holder #(int, "static_ctx"))::f(), "static_ctx");
    ck_s("B16a multi-specialization ALPHA", b16a.f(),  "ALPHA");
    ck_s("B16b multi-specialization BETA",  b16b.f(),  "BETA");
    ck_s("B16c multi-specialization GAMMA", b16c.f(),  "GAMMA");
    ck_s("B17 CTL bare at module scope, literal spec",
         holder #(int, "direct_spec")::type_name, "direct_spec");

    $display("| ---- BUG B: sweep-2 shapes ----");
    ck_s("B18 bare, DEFAULT type-param binding",         b18.f(), "default_spec");
    ck_s("B19 bare, MODULE type parameter",
         n_mp_u.get_type_name(), "mod_spec");
    ck_s("B20 bare via IMPORTED package alias",
         h_t::type_name, "pkg_spec");
    ck_s("B21 bare, class NESTED in a class",
         n_outer::n_inner #("nested_spec")::type_name, "nested_spec");
    ck_s("B21b CTL parenthesised, nested class",
         n_outer::n_inner #("nested_spec")::type_name(), "nested_spec");
    ck_s("B22 bare inside the CONSTRUCTOR",              b22.cached, "ctor_spec");
    begin string s; b23.cache_name(s);
      ck_s("B23 bare inside an instance TASK",           s, "task_spec"); end
    ck_s("B24 bare as a DEFAULT ARGUMENT value",         b24.greet(), "dflt_arg");
    ck_s("B25 bare via member typedef of own param",     b25.f(), "wrap_spec");
    ck_s("B26 bare, callee declared AFTER caller",       b26.f(), "late_spec");
    ck_s("B27 bare via CROSS-PACKAGE imported alias",
         n_pk_b::user::get(), "xpkg_spec");
    ck_s("B28 CTL bare, literal spec with param actual", b28.f(), "param_actual");
    ck_s("B29 CTL bare, class declared IN a module",     n_mod_u.s_bare, "in_mod");
    ck_s("B29b CTL parenthesised, class in a module",    n_mod_u.s_paren, "in_mod");
    ck_s("B30 CTL bare, specialization-of-specialization",
         holder #(holder #(int, "inner"), "outer")::type_name, "outer");
    ck_s("B31 CTL bare inside a $unit-scope function",   n_unit_fn(), "unit_spec");
    ck_s("B32 CTL static-init calls static fn",
         n_initprop #("init_spec")::label, "init_spec");

    $display("| ---- BUG E: typedef chain of depth 2 ----");
    ck_s("E01 chain2 BARE",   n_b_t::type_name,   "chain_spec");
    ck_s("E01b chain2 PAREN", n_b_t::type_name(), "chain_spec");

    $display("| ---- BUG F: bare static ref via instance handle ----");
    ck_s("F01 bare via handle",       n_h.type_name,   "handle_spec");
    ck_s("F01b CTL paren via handle", n_h.type_name(), "handle_spec");

    $display("| ---- BUG A: BARE static->static task call ----");
    begin int s; foreach (a_bare_mb::got[i])   s += a_bare_mb::got[i];
      ck_i("A01 bare mailbox: elements", a_bare_mb::got.size(), 2);
      ck_i("A01b bare mailbox: sum",     s, 85); end
    begin int s; foreach (a_qual_mb::got[i])   s += a_qual_mb::got[i];
      ck_i("A02 CTL qualified: elements", a_qual_mb::got.size(), 2);
      ck_i("A02b CTL qualified: sum",      s, 85); end
    begin int s; foreach (a_two_hops::got[i])  s += a_two_hops::got[i];
      ck_i("A03 two bare hops: elements", a_two_hops::got.size(), 1);
      ck_i("A03b two bare hops: sum",      s, 7);
      ck_i("A03c two bare hops: mid resumed", a_two_hops::mid_resumed, 1); end
    begin int s; foreach (a_base::got[i])      s += a_base::got[i];
      ck_i("A04 inherited callee: elements", a_base::got.size(), 1);
      ck_i("A04b inherited callee: sum",      s, 9); end
    begin int s; foreach (ap_t::got[i])       s += ap_t::got[i];
      ck_i("A05 parameterized class: elements", ap_t::got.size(), 1);
      ck_i("A05b parameterized class: sum",      s, 5); end
    ck_i("A06 output argument filled", a_out::seen, 84);
    ck_i("A07 CTL semaphore.get suspends (hits)", a_sem::hits, 2);
    ck_i("A07b CTL semaphore.get suspends (t_done)", a_sem::t_done, 20);

    $display("| ---- BUG A: sweep-2 shapes ----");
    ck_i("A08 LOCAL mailbox: nothing consumed", n_localmb::got.size(), 0);
    ck_i("A09 CTL mb.peek suspends",            n_peek::got.size(), 1);
    ck_i("A10 CTL callee inside a package",     n_pk_c::c::got.size(), 1);
    ck_i("A11 CTL static task via instance handle", n_hdl_task::got.size(), 1);
    ck_i("A12 forever consumer: elements",      n_forever::got.size(), 2);
    ck_i("A13 shadowed static task: got[0]",
          (n_sh_base::got.size() ? n_sh_base::got[0] : -99), 42);

    $display("| ---- BUG C: blocking on static sync objects in a static task ----");
    ck_i("C01 @static event inside static task",      c_event::hits, 1);
    ck_i("C01b @static event: wake time",             c_event::t_wake, 10);
    ck_i("C02 wait(static prop) inside static task",  c_wait::hits, 1);
    ck_i("C02b wait(static prop): wake time",         c_wait::t_wake, 10);
    ck_i("C03 @(posedge hier) inside static task",    (c_posedge::hits >= 1) ? 1 : 0, 1);
    ck_i("C03b @(posedge hier): wake time",           c_posedge::t_wake, 5);
    ck_i("C04 CTL instance task @event",              ci.hits_ev, 1);
    ck_i("C04b CTL instance task @event: wake time",  ci.t_wake_ev, 10);
    ck_i("C04c CTL instance task wait",               ci.hits_wt, 1);
    ck_i("C04d CTL instance task wait: wake time",    ci.t_wake_wt, 20);

    $display("| ---- BUG C: sweep-2 shapes ----");
    ck_i("C05 @(negedge hier) inside static task",    n_negedge::hits, 1);
    ck_i("C05b @(negedge hier): wake time",           n_negedge::t_wake, 10);
    ck_i("C06 @(e1 or e2) inside static task",        n_orev::hits, 1);
    ck_i("C06b @(e1 or e2): wake time",               n_orev::t_wake, 10);
    ck_i("C07 @static event, PARAMETERIZED class",    n_pev #("p")::hits, 1);
    ck_i("C07b param-class event: wake time",         n_pev #("p")::t_wake, 10);
    ck_i("C08 wait(static prop), PARAMETERIZED class", n_pwait #("p")::hits, 1);
    ck_i("C08b param-class wait: wake time",          n_pwait #("p")::t_wake, 10);
    ck_i("C09 ->> trigger wakes @event",              n_nbtrig::hits, 1);
    ck_i("C09b ->> trigger: wake time",               n_nbtrig::t_wake, 10);

    // These three pass in isolation (probes a03 / a20 / a22 all MATCH) and
    // fail only when the whole family is elaborated together -- the same
    // co-location interference already documented for the foreach MWEs.
    $display("| ---- BUG D: merged-context interference (correct in isolation) ----");
    ck_i("D01 #delay in a bare static call (before)",   a_delay::t_before, 0);
    ck_i("D01b #delay in a bare static call (after)",   a_delay::t_after, 10);
    ck_i("D01c #delay in a bare static call (resumed)", a_delay::t_resumed, 10);

    $display("|");
    if (nfails == 0) $display("| TEST_PASS (%0d checks)", nchecks);
    else             $display("| TEST_FAIL count=%0d (%0d checks)", nfails, nchecks);
    $finish;
  end
endmodule
