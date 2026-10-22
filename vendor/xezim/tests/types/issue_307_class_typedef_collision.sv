// Two classes each declare a class-scoped typedef named `r` with different
// packed-struct layouts; the second is declared later and never instantiated.
class leaf_cfg;
	typedef struct packed {
		bit [1:0] intlv;
		bit [3:0] num_hosts;
		bit [3:0] first_host;
	} r;

	rand r regs_r;
	rand bit [3:0] sel_pre;

	constraint c_sel {
		sel_pre dist { 0 :/ 15, 1 :/ 7, 2 :/ 2, 3 :/ 1 };
		regs_r.num_hosts == (4'hf >> sel_pre);
	}

	constraint c_mode {
		({1'b0, regs_r.num_hosts} + {1'b0, regs_r.first_host}) <= 1;
		regs_r.intlv == 0;
	}
endclass

class other_cfg;
	typedef struct packed {
		bit [7:0] other_field;
		bit [15:0] another;
	} r;
endclass

class env_cfg;
	rand leaf_cfg leaf;

	function new();
		leaf = new();
	endfunction
endclass

module top_mod;
	initial begin
		env_cfg env = new();
		bit ok;
		ok = env.randomize();
		if (ok && env.leaf.regs_r.num_hosts == (4'hf >> env.leaf.sel_pre)
		    && env.leaf.regs_r.intlv == 0
		    && (env.leaf.regs_r.num_hosts + env.leaf.regs_r.first_host) <= 1)
			$display("MWE-BUDGET: PASS sel_pre=%0d num_hosts=%0d", env.leaf.sel_pre, env.leaf.regs_r.num_hosts);
		else
			$display("MWE-BUDGET: FAIL rc=%0d sel_pre=%0d num_hosts=%0d", ok, env.leaf.sel_pre, env.leaf.regs_r.num_hosts);
		$finish;
	end
endmodule
