# Running UVM testbenches on xezim

xezim runs **UVM 1800.2-2017 and 1800.2-2020.3.1** testbenches end-to-end on its
event-driven 4-state core:
build → connect → topology → `run_phase` stimulus → sequencer↔driver TLM handshake →
packet collection → objection-driven termination → report summary.

This guide covers how to invoke xezim on a UVM testbench, what is supported, and the known
limitations. (For the internal design/implementation history, see
[`uvm-run-phase-plan.md`](uvm-run-phase-plan.md).)

---

## Quick start (single top)

Point xezim at the UVM library, the include dirs, and the source files, and select the top
module with `-s`:

```bash
xezim --simulate -s top \
  -I <UVM>/src -I <rtl> -I <sv> -I <tb> \
  -D UVM_NO_DPI -D UVM_REPORT_DISABLE_FILE_LINE \
  <UVM>/src/uvm_pkg.sv \
  <design and testbench files...> \
  +UVM_TESTNAME=<test_name>
```

- `-I <UVM>/src` makes `` `include "uvm_macros.svh" `` resolve.
- `-D UVM_NO_DPI` is optional. Without it UVM imports its DPI-C helpers (regex
  matching, the command-line walk, `uvm_hdl_*` backdoor access) and xezim serves them
  from a built-in library — see [UVM's DPI-C library](#uvms-dpi-c-library).
- `+UVM_TESTNAME=<name>` selects the test; it overrides the `run_test("...")` argument.

### Adding the UVM library with `-uvm`

Set `XEZIM_UVM_DIR` once and pass `-uvm` instead of naming the library:

```bash
export XEZIM_UVM_DIR=/path/to/UVM        # or a release root, or its src/ directory
xezim -uvm -s top -I <rtl> -I <sv> -I <tb> \
  <design and testbench files...> +UVM_TESTNAME=<test_name>
```

`-uvm` puts the library's `src` directory on the include path, after your own
`-I` directories, and its `uvm_pkg.sv` first in the file list. It works with
`--compile`, `--parse` and `--preprocess` as well as simulation, and inside
`-f`/`-F` args files. A file list that already contains a `uvm_pkg.sv` is left
as it is. Without `-uvm`, nothing is added even when `XEZIM_UVM_DIR` is set.

`XEZIM_UVM_DIR` may name the `src` directory, a release root that holds
`src/`, or a checkout holding several releases as subdirectories, such as
https://github.com/nitronis/UVM (`1.1d`, `1.2`, `1800.2-2017`, `1800.2-2020`).
For such a checkout, `XEZIM_UVM_VERSION=<release>` picks the release; the
default is the newest present. `-uvm` without a usable `XEZIM_UVM_DIR` stops
with an error that says what is missing.

### Worked example — GettingVerilatorStartedWithUVM

```bash
xezim --simulate -s top \
  -I $UVM/src -I rtl -I sv -I tb \
  -D UVM_REPORT_DISABLE_FILE_LINE -D UVM_NO_DPI -D SVA_ON \
  $UVM/src/uvm_pkg.sv sv/pipe_pkg.sv sv/pipe_if.sv rtl/pipe.v tb/top.sv \
  +UVM_TESTNAME=data0_test
```

Expected: the test topology table, both monitors reporting `COLLECTED PACKETS = 76`, a
`--- UVM Report Summary ---` with `UVM_ERROR : 0` / `UVM_FATAL : 0`, and a clean `$finish`.

---

## Multiple top modules (hdl_top + hvl_top)

Many UVM testbenches declare **two unconnected top modules** — e.g. a BFM `hdl_top` holding
the interfaces, clock, and `uvm_config_db::set` calls, and an `hvl_top` running `run_test`.
Pass each with its own `-s`; xezim elaborates them all under a synthetic wrapper root:

```bash
xezim --simulate -s hdl_top -s hvl_top \
  -I <UVM>/src -I <agent> -I <tb> \
  -D UVM_NO_DPI -D UVM_REPORT_DISABLE_FILE_LINE \
  <UVM>/src/uvm_pkg.sv <agent files...> <rtl files...> \
  <tb>/hdl_top.sv <tb>/hvl_top.sv
```

If you give only one `-s`, behavior is exactly as before (no wrapper synthesized).

---

## UVM library versions

Both Accellera reference libraries work with the same invocation — just point the
include dir and `uvm_pkg.sv` at the version you want:

| Library | Status |
|---|---|
| **1800.2-2017** | Reference target. 32/35 sv-tests examples; Verilator parity on the worked example (76/76 packets). |
| **1800.2-2020.3.1** | Green on the worked example — in/out monitors agree (77/77 packets), `UVM_ERROR`/`UVM_FATAL` = 0. (The library's phasing collects one extra packet vs 2017; both runs are internally consistent.) |

Two version-specific notes:

- **UVM-1.2-era testbenches against a 1800.2 library** (e.g. the `1.2/examples/`
  tree, which the 1800.2 kits don't replicate) reference deprecated-API globals
  such as `uvm_top` and `uvm_default_printer`. The 1800.2 libraries only compile
  those under `` `define UVM_ENABLE_DEPRECATED_API ``, so add
  **`-D UVM_ENABLE_DEPRECATED_API`** — otherwise elaboration fails with
  `Undeclared identifier 'uvm_top'`. This is a library configuration requirement,
  not an xezim gap.
- The 2020 library's inline conditional directives (a `` `ifndef … `endif ``
  in the middle of a declaration line, §22.6) are handled by the preprocessor —
  no user action needed.

---

## What you get

- **Topology** — `this.sprint(printer)` / `print()` renders the component + port tree in
  `uvm_table_printer` format (Name / Type / Size / Value).
- **Stimulus** — the sequencer↔driver `get_next_item` / `item_done` /
  `start_item` / `finish_item` rendezvous runs; sequences drive items into the DUT.
- **Termination** — the run phase ends when the phase objection count returns to zero
  (drain time honored), then extract/check/report/final run, then `$finish`.
- **Report summary** — `--- UVM Report Summary ---` with counts by severity and by id.

For UVM extensions that need their own DPI-C (e.g. a custom HDL backdoor, a cocotb
bridge, or an ISS shim), see [`dpi-guide.md`](dpi-guide.md) — the same `--dpi-lib`
mechanism works for UVM-side code as for plain SV testbenches.

### UVM's DPI-C library

Compiled without `-D UVM_NO_DPI`, UVM imports the C functions of its `src/dpi`
directory. xezim implements them natively; a `--dpi-lib` library that defines one of
the symbols takes precedence.

- **Regular expressions** (`uvm_re_match`, `uvm_glob_to_re`, `uvm_dpi_regcomp` /
  `regexec` / `regfree`, `uvm_dump_re_cache`, and 1800.2-2020.3's `uvm_re_comp` /
  `exec` / `free` / `compexec` / `compexecfree` / `deglobbed` / `buffer`) — POSIX
  extended syntax, as UVM's C code gets from the C library. This is what `uvm_config_db` / `uvm_resource_db` wildcard and
  `/regex/` scopes, `+uvm_set_config_*` / `+uvm_set_*_override` plusargs, factory
  overrides by instance path, and `uvm_cmdline_processor::get_arg_matches("/.../")`
  use. `uvm_glob_to_re` converts globs exactly as the C code does (`*` → `.*`,
  `+` → `.+`, `?` → `.`, `.[]()` escaped, anchored, `/.../` passed through); an empty
  glob gives `""` on UVM 1.2 and `/^$/` on 1800.2, following the library you compile.
- **Command line** — `uvm_cmdline_processor` sees `xezim` followed by every plusarg in
  command-line order (the same list `$test$plusargs` searches); `get_tool_name()`
  returns `xezim` and `get_tool_version()` its version.
- **Backdoor access** (`uvm_hdl_check_path` / `read` / `deposit` / `force` /
  `release` / `release_and_read`) — paths are absolute, as the register layer builds
  them: `top.u_dut.r_ctrl`, optionally prefixed with `$root.`; with several top modules
  each top names its own tree. A path may end in a bit-select (`sig[3]`), a part-select
  in the declared direction (`sig[7:4]`, `asc[0:3]`), a memory word (`mem[5]`, and
  `mem[5][3:0]`), or a packed-struct member (`st.field`). Reads zero-extend into
  `uvm_hdl_data_t`; a deposit writes the low bits; a force holds until released; a
  release returns a net to its drivers at once and leaves a variable at the forced
  value.
- **Signal polling** (1800.2-2020.3 with `UVM_PLI_POLLING_ENABLE`): `uvm_hdl_signal_size`
  gives a path's width; `uvm_polling_create` probes a signal, and once
  `uvm_polling_setup_notifier` names the notifier bit, every value change of an enabled
  probe reaches `uvm_polling_value_change_notify` through
  `uvm_polling_process_changelist`.
- **Errors** are UVM reports with the C code's ids and texts: `UVM/DPI/REGEX_INV`,
  `UVM/DPI/REGEX_MAX`, `UVM/DPI/REGCOMP`, `UVM/DPI/HDL_GET` / `HDL_SET` ("unable to
  locate hdl path"). Forcing or releasing part of a signal, forcing a
  `XEZIM_PACKED_MEM` memory cell, and selecting into a multi-dimensional packed vector
  are not supported: the call returns 0 and prints a one-time `[DPI] error:` line.

---

## Supported

- `uvm_test` / `uvm_env` / `uvm_agent` / `uvm_driver` / `uvm_monitor` / `uvm_sequencer` /
  `uvm_scoreboard` and the standard phase methods.
- Sequences: `body`, `start`, `start_item`/`finish_item`, ``uvm_do``/`uvm_do_with`,
  `randomize() with {...}`.
- TLM: analysis ports (broadcast), `uvm_*_imp`/export, `put`/`get`, TLM fifos, via the
  connect-phase connection graph.
- Virtual interfaces (LRM §25.8/§25.9): member reads, `@(posedge vif.clk)` event
  sensitivity, vif assignment (null clears; vif-to-vif copy), task-arg aliasing.
- `uvm_config_db#(T)::set/get/exists` — scope-aware with wildcard matching, including
  virtual-interface values and the BFM `set(null,"uvm_test_top",...)` pattern.
- Objection model: `raise_objection` / `drop_objection` / `set_drain_time`.
- The factory (`type_id::create`), overrides, and parameterized components.

## Known limitations / out of scope

- **Deprecated UVM-1.0 API** — `` `uvm_sequencer_utils ``, `` `uvm_sequence_utils ``,
  sequence libraries. These macros are undefined in 1800.2-2017 and will produce a parse
  error.
- **Partial forces through the backdoor** — `uvm_hdl_force("top.u.sig[7:4]", …)` (and
  the matching release) fail with a `[DPI] error:`; force the whole signal instead.
- **RAL** (register abstraction layer) and sequence lock/grab arbitration beyond the
  common path.
- **Cosmetic differences vs a reference run:** topology handle ids (`@N`) are xezim heap
  handles; Report-Summary `UVM_INFO`/`UVM_WARNING` totals are higher (xezim emits more
  verbose informational reports). The correctness-bearing `UVM_ERROR : 0` /
  `UVM_FATAL : 0` lines match exactly.

---

## Troubleshooting

| Symptom | Likely cause / fix |
|---|---|
| `config_db ... ::get(...) failed` (NOVIF) at build_phase, sim ends at t=0 | A second top (e.g. BFM `hdl_top`) that holds the `config_db::set` calls wasn't elaborated. Pass every top with its own `-s` (see *Multiple top modules*). |
| `No test specified` (UVM_FATAL NOTEST) | Add `+UVM_TESTNAME=<name>` or ensure `run_test("<name>")` has an argument. |
| `Requested test "X" not found` | The test class name doesn't match a compiled class; check spelling and that the file is in the source list. |
| `unexpected token in class: "("` near a sequencer/sequence | Deprecated UVM-1.0 `` `uvm_*_utils `` macro — out of scope (see limitations). |
| `Undeclared identifier 'uvm_top'` (or `uvm_default_printer`) at elaboration | A UVM-1.2-era testbench compiled against a 1800.2 library. Add `-D UVM_ENABLE_DEPRECATED_API` (see *UVM library versions*). |
| Run never terminates (hits `--max-time`) | The test raises no objection (some examples run open-ended). Set `--max-time <N>` to bound it. |
| Stimulus never flows / monitor collects 0 | Confirm the driver's `seq_item_port` connects to the sequencer in `connect_phase`, and the test starts a sequence on that sequencer. |
