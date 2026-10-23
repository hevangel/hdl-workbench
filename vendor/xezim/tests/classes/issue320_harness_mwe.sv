typedef enum int { PASSIVE_CFG, ACTIVE_CFG } env_cfg_e;

interface my_if #(type T = int);
  T data;
endinterface

// ---- minimal uvm_component stand-in: name/parent + full name ----
class component;
  string    m_name;
  component m_parent;
  static int errors;               // shared failure counter for the self-check
  function new(string name, component parent);
    m_name   = name;
    m_parent = parent;
  endfunction
  function string get_full_name();
    if (m_parent == null) return m_name;
    return {m_parent.get_full_name(), ".", m_name};
  endfunction
endclass

// ---- uvm_config_db scope matching: '*' matches any run INCLUDING '.' ----
function automatic bit glob_match(string pat, string s);
  int p, i, star_p, star_i;
  p = 0; i = 0; star_p = -1; star_i = 0;
  while (i < s.len()) begin
    if (p < pat.len() && pat[p] == "*") begin
      star_p = p; star_i = i; p++;
    end else if (p < pat.len() && pat[p] == s[i]) begin
      p++; i++;
    end else if (star_p >= 0) begin
      p = star_p + 1; star_i++; i = star_i;
    end else begin
      return 0;
    end
  end
  while (p < pat.len() && pat[p] == "*") p++;
  return (p == pat.len());
endfunction

// ---- minimal uvm_config_db#(T) stand-in: one static table per
// ---- specialization; set stores a scope PATTERN, get glob-matches ----
class cfg_db #(type T = int);
  // Parallel static queues instead of a queue-of-struct: the struct form
  // (static queue of a struct with string members) loses its string members
  // on read-back in xezim -- a separate finding, kept out of the A/B/C legs.
  static string scopes[$];
  static string fields[$];
  static T      vals[$];

  static function string scope_of(component ctx, string inst_name);
    if (ctx == null)          scope_of = inst_name;
    else if (inst_name == "") scope_of = ctx.get_full_name();
    else                      scope_of = {ctx.get_full_name(), ".", inst_name};
  endfunction

  static function void set(component ctx, string inst_name, string field, T val);
    scopes.push_back(scope_of(ctx, inst_name));
    fields.push_back(field);
    vals.push_back(val);
  endfunction

  static function int size();
    return scopes.size();
  endfunction

  static function bit get(component ctx, string inst_name, string field, output T val);
    string scope = scope_of(ctx, inst_name);
    foreach (scopes[i]) begin
      if (fields[i] == field && glob_match(scopes[i], scope)) begin
        val = vals[i];
        return 1;
      end
    end
    return 0;
  endfunction
endclass

// ---- B: monitor gets a parameterized vif through its class type params ----
class mon #(type T = int) extends component;
  virtual my_if#(T) vif;
  int      xi;
  function new(string name, component parent);
    super.new(name, parent);
  endfunction
  function void build();
    if (!cfg_db#(int)::get(this, "", "x", xi)) begin
      $display("[ERROR] A: int get with dot-crossing glob missed");
      errors++;
    end else
      $display("[OK] A: int get hit, x=%0d", xi);
    if (!cfg_db#(virtual my_if#(T))::get(this, "", "vif", vif)) begin
      $display("[ERROR] B: parameterized-vif get missed (class-scope table size %0d, expected 1)",
               cfg_db#(virtual my_if#(T))::size());
      errors++;
    end else
      $display("[OK] B: parameterized-vif get hit, data=%0d", int'(vif.data));
  endfunction
endclass

class env extends component;
  mon#(int) m;
  env_cfg_e cfg;
  function new(string name, component parent);
    super.new(name, parent);
  endfunction
  function void build();
    m = new("IPW55_master_dot_monitor", this);
    if (!cfg_db#(env_cfg_e)::get(this, "", "ipw61_env_cfg", cfg)) begin
      $display("[ERROR] C: enum ipw61_env_cfg get missed (dut_path from the class-method call was empty)");
      errors++;
    end else
      $display("[OK] C: enum ipw61_env_cfg get hit, cfg=%s", cfg.name());
    m.build();
  endfunction
endclass

class test extends component;
  env e;
  string dut_path;
  string cp;
  function new(string name, component parent);
    super.new(name, parent);
  endfunction
  function void build();
    e = new("sub_env", this);                    // env at top.sub_env
    dut_path = tb.sub_env.u_h.get_inst_path();   // like the production harness helper
    $display("INFO: get_inst_path() from class method = [%s] (expected [sub_env])", dut_path);
    cp = tb.sub_env.u_h.const_path();
    $display("INFO: const_path() from class method, via variable = [%s] (expected [sub_env])", cp);
    $display("INFO: const_path() from class method, inline = [%s] (expected [sub_env])",
             tb.sub_env.u_h.const_path());
    cfg_db#(env_cfg_e)::set(this, dut_path, "ipw61_env_cfg", ACTIVE_CFG);
    e.build();
  endfunction
endclass

// ---- the harness stand-in: %m initializer + path split/join ----
// (split_path_components replicated EXACTLY from the production utilities
//  package, including the `ref` queue argument)
module harness_mod;
  string inst_path = $sformatf("%m");
  function automatic void split_path_components(string s1, ref string inst_hier[$]);
    int last_sep_idx, curr_inst_len;
    last_sep_idx = -1;
    curr_inst_len = 0;
    for (int i = 0; i < s1.len(); i++) begin
      if ((s1[i] == ".") || (i == (s1.len()-1))) begin
        inst_hier.push_back(s1.substr(last_sep_idx + 1, last_sep_idx + curr_inst_len + (s1[i] != ".")));
        last_sep_idx = i;
        curr_inst_len = 0;
      end
      else begin
        curr_inst_len++;
      end
    end
  endfunction
  function automatic string get_inst_path();
    string comps[$];
    int i;
    split_path_components(inst_path, comps);
    get_inst_path = comps[1];
    i = 2;
    while (i < (comps.size() - 1)) begin
      get_inst_path = {get_inst_path, ".", comps[i]};
      i++;
    end
  endfunction
  // Control: a module function with no %m / no queue at all, to tell whether
  // the class-method call failure is specific to the %m path or general.
  function automatic string const_path();
    const_path = "sub_env";
  endfunction
  initial $display("INFO: local call: inst_path=[%s] get_inst_path=[%s]",
                   inst_path, get_inst_path());
endmodule

// wrapper so the harness path is tb.sub_env.u_h -> get_inst_path() = "sub_env",
// matching the env component at top.sub_env (like production's
// testbench -> SoC -> core -> memory-controller hierarchy)
module sub_env_mod;
  harness_mod u_h();
endmodule

module tb;
  sub_env_mod sub_env();
  my_if#(int) vif();
  initial begin
    test t;
    int xi2;
    virtual my_if#(int) v2;
    bit f1, f2;
    $display("INFO: get_inst_path() from module initial = [%s] (expected [sub_env])",
             sub_env.u_h.get_inst_path());
    $display("INFO: const_path() from module initial = [%s] (expected [sub_env])",
             sub_env.u_h.const_path());
    vif.data = 42;
    // A: '*' must cross the '.' to match IPW55_master_dot_monitor
    cfg_db#(int)::set(null, "top.sub_env.IPW55*", "x", 7);
    // B: set with concrete params, get goes through class type params
    cfg_db#(virtual my_if#(int))::set(null, "top.sub_env.IPW55*", "vif", vif);
    // Control: the same gets from the SAME scope as the sets must hit — this
    // pins B's failure to the class-scope specialization key, not to the set.
    f1 = cfg_db#(int)::get(null, "top.sub_env.IPW55_master_dot_monitor", "x", xi2);
    f2 = cfg_db#(virtual my_if#(int))::get(null, "top.sub_env.IPW55_master_dot_monitor", "vif", v2);
    $display("INFO: module-scope control gets f1=%0b f2=%0b (both must be 1)", f1, f2);
    // stand-in for run_test(): construct the tree, then build top-down
    t = new("top", null);
    t.build();
    if (component::errors == 0)
      $display("TEST PASSED");
    else
      $display("TEST FAILED with %0d errors", component::errors);
    $finish;
  end
endmodule

