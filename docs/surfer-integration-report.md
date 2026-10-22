# LUNA-00B: Surfer browser integration feasibility

**Status: blocked; do not freeze the Surfer adapter contract from this spike.** The pinned source was inspected, and a VCD plus a diagnostic harness were prepared. This environment could not execute the harness in a browser or obtain/build a pinned WASM bundle. No claim is made that the fixture loaded, that a signal was added, or that the cursor moved.

## Pin and licensing

| Item | Finding |
|---|---|
| Upstream | [surfer-project/surfer](https://gitlab.com/surfer-project/surfer) |
| Source release | `v0.7.0`, released 2026-04-27 |
| Source revision | `bd749b1f786c1c62cd67893ca71346cbe6983915` |
| License | EUPL-1.2; retain the upstream `LICENSE-EUPL-1.2.txt` with redistributed assets |
| WASM artifact | **Not pinned.** Upstream README identifies `pages_build` as a CI artifact from moving `main`, not a release asset. The corresponding `v0.7.0` job-artifact URL returned HTTP 404. No artifact bytes or SHA-256 were obtained. |

Therefore, the source is pinned but the WASM asset is not. A production bundle still needs to be built from the pinned revision or fetched from an immutable CI artifact, then recorded with a content digest and license file. The available environment has Node `v25.2.1` and pnpm `11.25.0`, but no Rust toolchain, so an upstream WASM rebuild was not possible here.

## Source-level integration findings

The pinned [`surfer/assets/integration.js`](https://gitlab.com/surfer-project/surfer/-/blob/bd749b1f786c1c62cd67893ca71346cbe6983915/surfer/assets/integration.js) listens for `window.postMessage` objects and recognizes these wrapper commands:

- `LoadUrl`: `{ command: "LoadUrl", url: "https://..." }`. The wrapper turns this into the internal `LoadWaveformFileFromUrl` message with `Clear` load options.
- `InjectMessage`: `{ command: "InjectMessage", message: "<JSON>" }`. The wrapper forwards the string to Surfer's internal `inject_message` function. Upstream explicitly describes the inner message API as unstable; it is not a stable host contract.

At the pinned source revision, Rust's Serde message definitions and reference types imply these candidate JSON payloads for a scalar `top.count` at tick 25:

```json
{"AddVariables":[{"path":{"strs":["top"],"id":"None"},"name":"count","id":"None","index":null}]}
```

```json
{"CursorSet":[1,[25]]}
```

The second payload follows the pinned `BigInt` Serde tuple representation: positive sign (`1`) and little-endian base-2^32 digits (`[25]`). The harness sends these payloads through `InjectMessage`. This is source-derived protocol evidence only: neither payload was accepted by a running viewer in this spike, so they must not be treated as verified capabilities. There is no documented stable host API for add-signal or cursor control in the inspected integration wrapper.

Upstream's README recommends an iframe and describes a setup-hook placeholder that an integrator can replace to signal viewer readiness. The inspected integration script does not provide a parent-page ready event by itself. A fixed delay is used by the diagnostic harness and is not suitable as a production handshake.

The WASM source contains WCP-related functions, but the pinned Trunk import/export glue inspected for this spike exports rendering and message-injection functions to the viewer page; it does not expose those WCP functions as a parent-page bridge. Do not assume WCP is a usable browser-host API without a build-specific proof.

## Spike artifacts

- [`spikes/surfer-embed/fixture-counter.vcd`](../spikes/surfer-embed/fixture-counter.vcd) is a real, small VCD fixture with a 1 ns timescale, `top.clk`, and four-bit `top.count`; the counter changes every 5 ns and equals 5 at tick 25.
- [`spikes/surfer-embed/index.html`](../spikes/surfer-embed/index.html) is a diagnostic harness. It embeds the public hosted viewer and posts `LoadUrl`, candidate `AddVariables`, then candidate `CursorSet` messages. Since the hosted app is unpinned, this is not an acceptable release integration test.
- [`spikes/surfer-embed/server.mjs`](../spikes/surfer-embed/server.mjs) serves the harness and VCD with CORS enabled for the hosted viewer's cross-origin fetch.

The harness does **not** satisfy the same-origin iframe requirement: the only obtainable viewer was `https://app.surfer-project.org/`, while the harness/fixture are served locally. It is retained as a diagnostic artifact, not as a verified implementation.

## Reproduction and test log

Run from the repository root in PowerShell:

```powershell
cd .\spikes\surfer-embed
node .\server.mjs
```

Then open `http://127.0.0.1:4173/` in a browser that can reach the local machine. The expected observations are recorded in the page: viewer frame load, `LoadUrl` to `fixture-counter.vcd`, and candidate add-signal/cursor messages. A successful run must additionally show `top.count` and the cursor at tick 25 in the viewer, with browser console evidence and a screenshot.

Checks available in this environment:

| Command / action | Result |
|---|---|
| `node --check .\server.mjs` | Passed (server JavaScript syntax). |
| `node --version` | `v25.2.1`. |
| `pnpm --version` | `11.25.0`. |
| Resolve `playwright`, `@playwright/test`, `puppeteer` from Node | All missing; no automated browser runner is installed. |
| Open the local harness in the available browser runner | Failed: `http://127.0.0.1:4173/` returned `net::ERR_CONNECTION_TIMED_OUT`. The Node local fetch also timed out despite the server port already being bound (`EADDRINUSE` on a second server start), so the execution/browser sandbox cannot reach the loopback listener. |
| Load the VCD, add `top.count`, position tick 25, capture evidence | Not run. No browser version, screenshot, or successful viewer log is available. |
| Fetch pinned `v0.7.0` `pages_build` artifact | Failed: GitLab returned 404 for `https://gitlab.com/surfer-project/surfer/-/jobs/artifacts/v0.7.0/download?job=pages_build`. |

The local server and fixture have not been validated end to end; `node --check` only validates JavaScript syntax. The available browser integration did not support reaching the process bound on this worktree's loopback address. Consequently, the explicit failure is an environment access failure, **not** evidence that Blob URLs or either Surfer message fail.

## Fallback decision

For the next browser-capable spike, use a locally built, source-pinned Surfer bundle served from the IDE origin and load the VCD through an IDE-owned same-origin HTTP URL. Upstream documents iframe embedding of a local copy and URL-based loading (`LoadUrl`); this is a narrower and source-documented path than depending on Blob URL behavior or cross-origin CORS. A same-origin route also lets the application keep the waveform in its own storage and expose it to Surfer through an explicit URL.

This fallback is **validated only against upstream's documented integration model**, not by a successful browser run. Blob URL support remains unresolved. The adapter contract must stay provisional until an automated browser test with the pinned asset proves fixture load, signal addition, and cursor movement—or until a stable upstream control API is selected.

## Next tasks

1. On a browser-enabled host with Rust/Trunk and network access, build Surfer at the source SHA above (or retrieve an immutable artifact), record the exact build inputs and SHA-256, and include the EUPL license.
2. Serve that bundle and the fixture from the same origin. Replace the fixed timeout with an explicit setup hook/ready event in the local build.
3. Add a Playwright test that waits for readiness, loads the fixture, adds `top.count`, moves to tick 25, and asserts the rendered signal/cursor; save browser version, console log, and screenshot.
4. Test `blob:` loading separately. If unsupported, keep the same-origin HTTP URL route as the fallback and record the observed browser failure.
5. Freeze `SurferAdapter` only after the test verifies the pinned build's message shapes. Keep internal `InjectMessage` payloads isolated behind the adapter because upstream marks them unstable.

## Upstream references

- [Pinned integration wrapper](https://gitlab.com/surfer-project/surfer/-/blob/bd749b1f786c1c62cd67893ca71346cbe6983915/surfer/assets/integration.js)
- [Pinned Trunk configuration](https://gitlab.com/surfer-project/surfer/-/blob/bd749b1f786c1c62cd67893ca71346cbe6983915/surfer/Trunk.toml)
- [Pinned message enum](https://gitlab.com/surfer-project/surfer/-/blob/bd749b1f786c1c62cd67893ca71346cbe6983915/libsurfer/src/message.rs)
- [Surfer integration documentation and artifact instructions](https://gitlab.com/surfer-project/surfer)
- [Surfer command documentation](https://docs.surfer-project.org/book/commands/index.html)
- [Pinned license](https://gitlab.com/surfer-project/surfer/-/blob/bd749b1f786c1c62cd67893ca71346cbe6983915/LICENSE-EUPL-1.2.txt)
