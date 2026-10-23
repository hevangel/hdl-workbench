# Agent operating rules — Xezim Web IDE

Read `IMPLEMENTATION_PLAN.md` and `LUNA_TASKS.md` before writing code.

- **Primary product invariant:** in browser-only and agent-controlled browser modes, simulation executes Xezim WebAssembly in a browser Web Worker. Do not silently substitute native Xezim or a fake JS interpreter.
- **Keep source fidelity:** preserve existing native Xezim API, tests and license. Pin upstream SHA and document each feature-gate patch.
- **Separation of concerns:** frontend controller, WorkspaceAdapter, SimulationService, SurferAdapter, and MCP bridge communicate over versioned typed contracts. Never directly manipulate Surfer's unstable private messages from unrelated modules.
- **Data protection:** no write without authoritative workspace and checked revision. Report save status only after successful persistence. Snapshot project files for compile; attach snapshot ID to logs/waveforms.
- **Security:** MCP bridge listens only on loopback, pairs authenticated browser sessions, validates origins, constrains operations to approved root, rejects symlink/path escape, and does not execute arbitrary commands. No untrusted native DPI in browser.
- **Truthful APIs:** distinguish implemented behavior from proposed APIs. Do not claim arbitrary pause/resume or historical `getSignal(time)` exists unless code and tests prove it. Test actual pinned Surfer integration messages.
- **Testing:** for every feature, add positive, negative and recovery tests. Reference comparison for simulation semantics uses native Xezim with the same sources/options.
- **Concurrent agents:** work on feature-named branches or worktrees. Respect path ownership defined in LUNA_TASKS.md. Do not concurrently edit Xezim core files; one lane coordinates those changes.
- **On every PR:** provide changed-file list, new APIs, tests run and exact results, screenshots where relevant, known limitations, upstream SHA, and instructions to reproduce.
- **Fail fast:** before a large implementation, prove WASM feasibility and Surfer load/integration. Escalate blockers with compiler output and minimal reproducer, not speculative workarounds.
