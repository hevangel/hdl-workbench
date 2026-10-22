# Contributors

Thank you to everyone who has improved xezim through pull requests:

- **Thomas Burg** — class-system and UVM fixes: static-property chains through
  object handles (§8.25), associative-array method dispatch and ref-writeback,
  `ClassName::static_prop` access, parser-gap self-tests, test-harness
  hardening, per-process bookkeeping for methods that park mid-body, the
  condition-waiter drain de-duplication, the NBA-region lane in the
  `--max-time` hang report, class-method performance (fast accessor inlining,
  peephole branch fusion), packed-struct rand-array constraints, queue and
  associative-array returns, handle-keyed `foreach` binding, the resume
  order of NBA-region waiters, bytecode compilation of wait-free class tasks
  and package functions, character writes into strings through every
  receiver shape, block-local scoping in tasks and methods, per-invocation
  task-local arrays and array formals that write back to class properties,
  `@(... iff g)` guards evaluated in the waiting process, fork children
  waking a parked parent, scalar `bit` DPI formals that carry one bit, and
  DPI regression tests for real actuals and returns, scalar `svLogic` cells
  and enum prototypes.
- **Vrajesh Prakhya** — real-number modelling: Verilog-AMS `wreal` nets
  resolved by summing, user-defined nettypes across the hierarchy and in
  packages (§6.6.7, §6.6.8), real-ness of members projected from call results,
  negative-test registrations, and the diagnosis that `cover property` sites
  were tallied as failing assertions.
- **Oscar Gustafsson** — expanded VPI functionality (`vpi_get_value`,
  `ObjectValType`), CI setup, and clippy cleanups.
- **Chen Ben Haroosh** — submodule-inline generate-for elaboration: genvar-
  dependent declarations and `parameter type` default resolution, plus the
  accompanying SystemVerilog compliance cases.
- **Jayaraman RP** — cross-platform installation scripts, including the macOS
  installer with UVM setup.
- **Ganesh T S** — wide-number parsing for `$sscanf`/`$fscanf`/`$value$plusargs`
  and the string conversion methods (arbitrary-precision decimal in the core),
  loop-variable scoping against same-named identifiers in child instances,
  parameter values converted to their declared type's width and sign (§6.20),
  `%c` in `$fscanf`/`$sscanf` matching exactly one character, and detailed
  self-checking bug reports for struct and class member access.
- **eenky** — declared defaults for parameters a class leaves out when it
  extends a parameterized class, and virtual-interface reads through
  class-handle chains (`cfg.vif.data`).
- **Francesco Urbani** — `+incdir+` directories in `-F` args files resolved
  against the args file's own directory.
- **Taichi Ishitani** — constrained-random fixes (members of rand object array
  elements, `pre_randomize` before the rand and constraint modes are read,
  `this.` member paths in the joint solver, and `XEZIM_RAND_DIAG` reports of
  why `randomize()` failed), named `extends` lists and type-parameter formals
  resolved from the receiver's own bindings, class handles resolved before
  same-named structs, unpacked-struct mailbox messages through task formals,
  and array signs preserved through subroutine frames.
- **AaronKel** — bytecode-compiler rollback that discards stale jump fixups
  (an early `return` inside a loop that cannot be unrolled used to crash the
  compiler or hang the caller's loop), and nested type arguments kept in
  static-call specializations, so `uvm_config_db#(cfg#(AW,DW))::get` reaches
  the configuration that was set, and factory `type_id::create` through chains
  of class typedefs.

## Bug reports

Detailed, self-checking bug reports have driven much of xezim's conformance
work. Thank you in particular to
[AaronKel](https://github.com/AaronKel),
[bharathnaruto-spec](https://github.com/bharathnaruto-spec),
[chenbhil](https://github.com/chenbhil),
[Dragon-Git](https://github.com/Dragon-Git),
[Ganesh-TNL](https://github.com/Ganesh-TNL),
[jjts](https://github.com/jjts),
[overcookedlobster](https://github.com/overcookedlobster),
[rharikrishna25](https://github.com/rharikrishna25),
[soronpo](https://github.com/soronpo),
[taichi-ishitani](https://github.com/taichi-ishitani),
[urbanij](https://github.com/urbanij) and
[vrajeshprakhya](https://github.com/vrajeshprakhya).

New contributors are welcome; see [CONTRIBUTING.md](CONTRIBUTING.md).
