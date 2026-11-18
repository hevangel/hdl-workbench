# Pinned Surfer WASM browser integration proof

This spike serves the pinned Surfer WASM bundle from the same origin as the parent harness and counter fixture. It verifies the documented iframe `LoadUrl` wrapper, the observed `InjectMessage` path for adding `top.count` and setting the cursor to 25 ns, and Blob URL loading.

## Pinned bundle

- Upstream release/source: Surfer `v0.7.0`, commit `bd749b1f786c1c62cd67893ca71346cbe6983915`.
- License: EUPL-1.2. The upstream license is included at `assets/LICENSE-EUPL-1.2.txt`.
- Browser WASM SHA-256: `1341ad01c59f5169209239728417d14f430183abd92515f22bf07cd7c86937ae` (`assets/surfer/surfer_bg.wasm`). All shipped asset checksums are in `assets/SHA256SUMS`.
- Build: Rust `1.99.1`, Cargo `1.99.1`, target `wasm32-unknown-unknown`, Trunk `0.21.14`; exact upstream `f128` and `instruction-decoder` submodule commits are recorded in the integration report.

`assets/surfer/index.html` is the upstream Trunk output with relative `/dist/` paths and a small `SURFER_SETUP_HOOKS` readiness notification, following the upstream integration instructions. The JS and WASM binaries are the pinned build output without source changes.

## Run locally

From the repository root in PowerShell:

```powershell
cd .\spikes\surfer-embed
node .\server.mjs
```

Open `http://127.0.0.1:4173/`. Click **Load fixture**, **Add top.count**, and **Set cursor to 25 ns**. The bundled test fixture uses a 1 ns timescale; `top.count` equals 5 at tick 25. The frame and fixture are served on the same origin.

## Automated Firefox test

The checked-in test uses Python Playwright `1.59.0` and Camoufox `0.4.11` (Firefox `152.0.4-beta.30`) and runs headed under Xvfb because Surfer's renderer requires WebGL. In the provisioned Docker setup used for this proof:

```powershell
docker cp .\spikes\surfer-embed\. xezim-luna04-builder:/tmp/surfer-localhost
docker exec -d xezim-luna04-builder sh -lc 'python3 -m http.server 4173 --bind 0.0.0.0 --directory /tmp/surfer-localhost'
docker network connect knowledge_base_default xezim-luna04-builder
Get-Content .\spikes\surfer-embed\test_surfer_integration.py -Raw | docker exec -i kb_camoufox sh -c 'cat > /tmp/test_surfer_integration.py'
docker exec -e DISPLAY=:99 kb_camoufox python /tmp/test_surfer_integration.py --headed --url http://xezim-luna04-builder:4173/ --output-dir /tmp/surfer-artifacts
```

The browser test asserts VCD body load, `Loaded 1 variables`, and `CursorSet(25)` in the viewer's WASM logs for the HTTP URL path and the Blob URL path. It saves screenshots and `browser-test-log.json` under `artifacts/` in this branch. `04-cursor-set.png` and `06-blob-signal-cursor.png` show the waveform, the `count [3:0]` signal, and the red cursor at 25 ns.

## Result and limits

The test passed in Firefox/Camoufox `152.0.4-beta.30` (UA reports Firefox 152.0) with no browser page errors or console errors. Both same-origin HTTP and parent-created `blob:` URLs loaded the fixture; `AddVariables` added one variable and `CursorSet(25)` moved the visible cursor in both cases.

`LoadUrl` is documented by the pinned iframe wrapper. `InjectMessage` forwards internal Rust `Message` values; upstream marks that inner API unstable. The observed `AddVariables` and `CursorSet` JSON payloads are pinned-build specific and must stay behind an adapter with versioned tests.

The WASM run logged `failed to create thread pool` with `Unsupported` in this container for asynchronous data work, but the fixture body, signal, and cursor still loaded and rendered. Larger waveform performance and worker behavior need testing in supported browsers. Headless Camoufox without WebGL fails with `WebGL isn't supported`; use the headed Xvfb setup in the test command. WCP methods in Rust are not exposed to the parent page by this Trunk glue, so this proof does not establish a browser WCP bridge.
