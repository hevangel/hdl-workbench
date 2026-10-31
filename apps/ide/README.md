# HDL Workbench IDE

HDL Workbench is a browser SystemVerilog IDE. The production entry point opens a
persistent browser workspace and runs the pinned Xezim WebAssembly build in a
dedicated Worker. If the WASM assets are missing, simulation is disabled; the
browser does not fall back to native Xezim. Mock services are used by component
tests and the explicit worker harness only.

## DPI and C/C++ helper files

Add `.c`, `.cc`, `.cpp`, or `.cxx` files to the project; add header files with
them when needed. Run compiles each translation unit in a dedicated
`browsercc` Worker, then passes the standalone modules to Xezim. C++ functions
used through DPI must export unmangled `extern "C"` names.

The browser ABI currently supports only DPI functions that return SystemVerilog
`int` and take scalar `input int` arguments. Helpers must link without host
imports or unresolved cross-translation-unit references. Exceptions, RTTI,
libc/libc++ runtime calls, DPI tasks, output/ref arguments, strings, arrays,
callbacks into SV, and time-suspending calls are unsupported. Source, module,
memory, and worker time limits are enforced; see
[`session_history/initial_build/luna-11-dpi-c-cpp-report.md`](../../session_history/initial_build/luna-11-dpi-c-cpp-report.md).

The compiler's first-use assets total 94.4 MB before transport compression.
On the selected Windows workstation, a real Chrome 156 profile completed the
C/C++ compile-and-DPI fixture in about 2.5 seconds; peak browser-process memory
and the exact workload are recorded in
[`session_history/initial_build/luna-12-dpi-memory-report.md`](../../session_history/initial_build/luna-12-dpi-memory-report.md).
That single-machine measurement does not establish requirements for smaller
devices or other browser engines. Public release review is still pending.

## Development

From the repository root:

```powershell
pnpm install --frozen-lockfile
pnpm --filter @xezim-ide/app typecheck
pnpm --filter @xezim-ide/app test
pnpm --filter @xezim-ide/app build
```

For the production-build browser acceptance, rebuild the Xezim WASM assets,
serve the app build, then run `apps/ide/scripts/wasm-browser-acceptance.mjs`
with the Chrome DevTools and harness URL environment variables. The full
reproducible sequence and the native compatibility check are in the LUNA-11
report.

Compiler notices are linked from the IDE status bar and are served from
`/licenses/THIRD_PARTY.md`.
