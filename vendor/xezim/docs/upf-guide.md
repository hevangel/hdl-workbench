# Power intent (UPF)

xezim reads IEEE 1801 (Unified Power Format) files and simulates the power
intent alongside the RTL: supply nets carry a state and a voltage, power
switches gate them, powered-down logic corrupts to `x`, isolation cells clamp
domain outputs, and retained registers keep their values.

## Flags

| Flag | Meaning |
|---|---|
| `--upf <file>` | Load a UPF file. Repeat for several files; `load_upf` inside a file resolves relative to that file. |
| `--upf-top </path/to/instance>` | The design instance the UPF scope (`set_design_top`) refers to. Without it the first instance of the `set_design_top` module is used. |
| `XEZIM_UPF_DUMP=1` | Print the generated power-aware glue. |

    xezim --simulate -s tb --upf power.upf --upf-top /tb/dut/core rtl.v tb.sv

A complete runnable example (switched domain, header switch, isolation and
retention) lives in `examples/upf/`; `./examples/upf/run.sh` simulates it and
`tests/upf/` covers the flow in the regression suite.

## Driving supplies from the testbench

The testbench controls the supply ports through the standard `UPF` package
(IEEE 1801 §11.2.4), which xezim provides automatically when `--upf` is given:

```systemverilog
import UPF::*;
initial begin
  st = supply_on("/tb/dut/core/VDD", 1.0);   // state FULL_ON, 1.0 V
  st = supply_on("/tb/dut/core/VSS", 0.0);
  ...
  st = supply_off("/tb/dut/core/VDD");
end
```

| Function | Effect |
|---|---|
| `supply_on(path, volts = 1.0)` | Supply port goes FULL_ON at the given voltage. |
| `supply_off(path)` | Supply port goes OFF. |
| `supply_partial_on(path, volts)` | Reported as PARTIAL_ON and applied as FULL_ON at the given voltage. |
| `get_supply_on_state(path)` | 1 while the net is FULL_ON. |
| `get_supply_voltage(path)` | The net's voltage as a real. |

Paths are `/top/inst/.../NET`, the dotted form, or a net name relative to the
UPF scope.

## Commands

Simulated:

| Command | Behaviour |
|---|---|
| `create_supply_net`, `create_supply_port`, `connect_supply_net`, `set_domain_supply_net` | Each net is a state (FULL_ON, OFF, UNDETERMINED) plus a voltage; ports are the nets the testbench drives. |
| `create_power_switch` | The output supply follows the input while an `-on_state` boolean over the control ports holds; an `x` control yields UNDETERMINED. Multiple `-on_state`/`-off_state` clauses are honoured. |
| `create_power_domain -elements` | While a domain's primary power or ground is not FULL_ON, every variable, net and output inside its elements reads `x` and keeps `x` until written after power-up. `-elements {.}` names the scope instance itself. A domain without a primary supply is always on. |
| `set_isolation`, `set_isolation_control` | While the control is active (`-isolation_sense`), the domain's isolated outputs read their `-clamp_value` at the domain boundary; the drivers resume when the control releases. Element-specific strategies override `-applies_to outputs`; `-update` merges options into the named strategy. A domain that powers down with its isolation control inactive is reported. |
| `set_retention` (+ `set_retention_control`) | Retained elements are exempt from corruption and keep their values through the power-down. |
| `load_upf [-scope inst]` | Nested files load relative to the loading file; with `-scope` their commands apply below that instance, and a `set_design_top` inside them names that instance's module. |
| `set_scope`, `set`, `$var`, `puts` | Tcl subset: braces, quotes, `\` continuation, `#` comments, `;` separators, variable substitution. |

Parsed and reported only (no runtime effect): `set_level_shifter`,
`add_port_state`, `create_pst`, `add_pst_state`, `create_supply_set`,
`associate_supply_set`, `add_power_state`, `create_logic_net`,
`create_logic_port`, `connect_logic_net`, `set_port_attributes`,
`set_design_attributes`, `set_simstate_behavior`, `upf_version`. Any other
command is skipped with a warning, so a full-flow UPF set (constraints,
configuration and implementation files chained by `load_upf -scope`) loads and
the simulated subset applies.

## Reporting

Elaboration prints a `[UPF]` summary (scope, supply nets, switches, domains
with their corruptible-signal count and retained elements, isolation
strategies, PSTs) followed by warnings for anything unresolved. During
simulation every power event is logged with the `[UPF] Time: ...` prefix:
supply changes, switch state, domain power-up/down, isolation enable/disable,
and isolation-control checks.

## Not modelled

PST legality checks at run time, supply-set functions (`PD.primary.power`),
`add_power_state` evaluation, level shifters (transparent), the `latch` clamp
value, `-applies_to inputs`, retention save/restore timing, and elements
inside instance arrays or generate blocks.
