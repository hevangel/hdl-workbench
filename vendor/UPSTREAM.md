# Vendored simulator sources

`xezim/` is Xezim revision `22faa127a9cc0156bd23e0ab117f400c64231913`.
`xezim-core/` (including `xezim-parser/`) is revision
`d15cdc39e27a4c0a8a55f7e1d46369181307ead9`.

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
