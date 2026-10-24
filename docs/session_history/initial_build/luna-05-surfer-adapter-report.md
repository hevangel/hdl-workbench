# LUNA-05 — Surfer WASM adapter

Date: 2026-10-22
Branch: `codex/luna-05-surfer-adapter`
Status: pinned viewer integration and available Firefox acceptance passed.

## Pinned viewer and changes

- Surfer v0.7.0, upstream commit `bd749b1f786c1c62cd67893ca71346cbe6983915`, EUPL-1.2.
- `surfer_bg.wasm` SHA-256: `1341ad01c59f5169209239728417d14f430183abd92515f22bf07cd7c86937ae`. The upstream asset digests and local readiness-wrapper digest are recorded under `apps/ide/public/surfer/`.
- The redistributed license is `apps/ide/public/surfer/LICENSE-EUPL-1.2.txt`. The only upstream generated file changed is `index.html`, for a nonce-based ready probe. The wasm and generated `surfer.js` are unchanged.
- `SurferIframeAdapter` implements the frozen adapter methods. It checks iframe source, same-origin, readiness nonce, URL origin, artifact completeness, signal paths, and time ranges. It requires an injected artifact URL resolver; it never guesses an artifact path. `dispatchSurferCommand` returns `UNKNOWN_COMMAND` for unsupported operations.
- The IDE has a WAVEFORMS panel with a clearly labeled bundled VCD fixture. It is a viewer fixture, not simulation output; the product still identifies simulation as mock-only.

## Tested integration details

The adapter contains all viewer-private message shapes. It uses the pinned wrapper's `LoadUrl` and the observed v0.7.0 `InjectMessage` forms for `AddVariables`, `CursorSet`, and `ZoomToRange`. Surfer upstream marks the inner message format unstable, so these payloads must remain in the adapter and be retested on upgrades.

The browser acceptance loaded the real fixture on the IDE's same origin, added `top.count`, moved the cursor to tick 25 (the fixture timescale is 1 ns), zoomed to 0–40 ns, and reloaded the iframe. Firefox logs showed `Loaded 1 variables`, `CursorSet(25)`, and `ZoomToRange { start: 0, end: 40, viewport_idx: 0 }`. The before/after iframe screenshots changed from the full 0–80 ns view to 0–40 ns, confirming the zoom affected the rendered waveform. The iframe became ready again through the nonce handshake after reload.

The static Surfer page uses the upstream `#dev` switch to skip its service worker. The pinned worker rewrites response headers and does not cache assets; the IDE can manage offline behavior at its own origin.

## Commands and results

Run from the repository root:

```sh
pnpm --filter @xezim-ide/app typecheck
pnpm --filter @xezim-ide/app test
pnpm --filter @xezim-ide/app build
git diff --check
```

Typecheck passed; 20 tests across 4 files passed; production build passed. Vite reports the large Monaco app chunk (about 3.09 MB minified / 811 KB gzip).

The automated real-browser check is `apps/ide/scripts/surfer-browser-acceptance.py`. In the provisioned `xezim-luna04-builder` / `kb_camoufox` Docker network, build the app first, then run:

```powershell
docker exec -d xezim-luna04-builder python3 -m http.server 4180 --bind 0.0.0.0 --directory /host/apps/ide/dist
docker exec -d xezim-luna04-builder python3 -m http.server 4175 --bind 0.0.0.0 --directory /host/apps/ide/scripts
docker exec kb_camoufox python3 -c 'import urllib.request; source=urllib.request.urlopen("http://xezim-luna04-builder:4175/stage-browser-site.py").read(); exec(compile(source,"stage-browser-site.py","exec"), {"__name__":"__main__"})'
docker exec -d kb_camoufox python3 -m http.server 4216 --bind 127.0.0.1 --directory /tmp/luna06-local
docker exec kb_camoufox python3 -c 'import os,sys,urllib.request; os.environ["DISPLAY"] = ":99"; os.environ["XEZIM_SURFER_TEST_URL"] = "http://127.0.0.1:4216/"; sys.argv = ["surfer-browser-acceptance.py", "--headed", "--output-dir", "/tmp/surfer-app-acceptance"]; source=urllib.request.urlopen("http://xezim-luna04-builder:4175/surfer-browser-acceptance.py").read(); exec(compile(source,"surfer-browser-acceptance.py","exec"), {"__name__":"__main__"})'
```

Result: passed in Camoufox Firefox `152.0.4-beta.30` under Xvfb, same-origin HTTP, with no page or console errors. The app entry module required the same test-only import workaround documented by LUNA-06; ordinary page-entry startup remains a separate browser release gate. The run logged Surfer's non-fatal `ThreadPoolBuildError` warning in this container, as did the feasibility spike.

Screenshots and the complete browser log are checked in under `apps/ide/test-artifacts/surfer/`, including [the zoomed waveform](../apps/ide/test-artifacts/surfer/04-zoomed.png) and [the reloaded viewer](../apps/ide/test-artifacts/surfer/05-reloaded-and-controlled.png).

## Limits and follow-on work

- `loadWaveform` awaits the injected URL resolver and confirms the wrapper command was posted. The pinned Surfer host wrapper does not provide an asynchronous VCD parse-completion acknowledgment; the acceptance test observes the real viewer logs. The application still needs an artifact store/resolver wired from simulation results.
- The browser test covers one small fixture in Camoufox Firefox. Production Firefox, Chromium, large waveform performance, and WCP host exports remain unverified.
- Load/signal/cursor/zoom use the pinned unstable inner message API; upgrading Surfer requires rerunning the browser acceptance and reviewing the new source message schema.
- Continue with LUNA-07 MCP bridge, LUNA-08 historical VCD queries, and LUNA-09 integration. Do not advertise historical signal queries, WCP, or simulation output waveforms before their own acceptance gates pass.
