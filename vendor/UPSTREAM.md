# Vendored simulator sources

`xezim/` is Xezim revision `e8266416d52b0bed45c0b78b8b7db39098221391` (release 0.11.4).
`xezim-core/` (including `xezim-parser/`) is revision
`8ebdf4a3f1346fe758e98f4a6a6c822f17f38e54`.

Both upstream projects are licensed `MIT OR Apache-2.0`; their license files
are kept in the corresponding source trees. The local copies were created from
`git archive` at those commits, without `.git` metadata. LUNA-03 portability
changes are applied directly to the vendored Xezim manifest, build script,
and library entry points. Native behavior remains the upstream default.

LUNA-11 adds a `wasm32`-only scalar DPI import dispatcher in
`xezim/src/compiler/simulator/dpi_imports.rs`. The WASM implementation binds
only function imports returning SV `int` with scalar `input int` ports and
calls a worker-installed JavaScript hook. Unsupported browser prototypes
produce compile errors. The existing native libffi/shared-library paths remain
under `cfg(not(target_arch = "wasm32"))` and are unchanged by this patch.
Browser module signature and resource checks live in
`apps/ide/src/workers/dpiWasmModules.ts`.

Plusargs: `xezim/src/lib.rs` gains `simulate_project_inline_with_plusargs`, an additive entry point that forwards `+name[=value]` arguments to the simulator's existing `set_plusargs`. `simulate_project_inline_with_output_limit` calls it with no arguments, so native behavior and existing callers are unchanged.

`fst-writer/` is `fst-writer` 0.3.1 (BSD-3-Clause, Kevin Laeufer) with `FstHeaderWriter::new` added so an FST can be written to any seekable writer. Upstream opens files by path only, and the browser has no filesystem. Only `crates/xezim-web` uses this copy, through `[patch.crates-io]`; the proptest-based unit tests were dropped with the dev-dependencies. Native Xezim's FST sink is unchanged and is not used in the browser: `crates/xezim-web/src/fst.rs` converts the VCD the simulator already produced.
