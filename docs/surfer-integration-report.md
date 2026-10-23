# LUNA-00B — Surfer browser integration feasibility

Date: 2026-10-22

Status: Browser proof passed on the isolated `codex/luna-00b-surfer-integration` branch at commit `65bc8f0323c8c31122175c8a4d6fe4e45762132b`.

The branch contains the pinned viewer bundle, license, real VCD fixture, Playwright test, screenshots, log, and full reproduction report. To inspect that complete report from this checkout:

```sh
git show codex/luna-00b-surfer-integration:docs/surfer-integration-report.md
```

## Pins and evidence

| Item | Result |
| --- | --- |
| Surfer | v0.7.0, source commit `bd749b1f786c1c62cd67893ca71346cbe6983915` |
| License | EUPL-1.2; the delegated branch retains the upstream license with the bundle |
| WASM SHA-256 | `1341ad01c59f5169209239728417d14f430183abd92515f22bf07cd7c86937ae` |
| Browser | Playwright 1.59.0 with Camoufox 0.4.11; Firefox 152.0.4-beta.30 under Xvfb |
| Fixture | Real VCD with `top.count`; its four-bit value is 5 at 25 ns |
| Result | Same-origin HTTP and Blob workflows loaded the VCD, added `top.count`, and moved the visible cursor to 25 ns; no page or console errors |

The pinned wrapper supports `LoadUrl` and `InjectMessage`. The browser test observed `WaveBodyLoaded`, `Loaded 1 variables`, and `CursorSet(25)`. These inner payloads worked in this exact build:

```json
{"AddVariables":[{"path":{"strs":["top"],"id":"None"},"name":"count","id":"None","index":null}]}
```

```json
{"CursorSet":[1,[25]]}
```

Surfer marks the inner `InjectMessage` API unstable. These payloads are version-specific test evidence, not a stable upstream contract. Keep them in the versioned `SurferAdapter` only.

## Reproduction

In the delegated worktree, use the checked-in `spikes/surfer-embed/test_surfer_integration.py` and bundle. The tested container invocation was:

```powershell
docker cp .\spikes\surfer-embed\. xezim-luna04-builder:/tmp/surfer-localhost
docker network connect knowledge_base_default xezim-luna04-builder
docker exec -d xezim-luna04-builder sh -lc 'python3 -m http.server 4173 --bind 0.0.0.0 --directory /tmp/surfer-localhost'
Get-Content .\spikes\surfer-embed\test_surfer_integration.py -Raw | docker exec -i kb_camoufox sh -c 'cat > /tmp/test_surfer_integration.py'
docker exec -e DISPLAY=:99 kb_camoufox python /tmp/test_surfer_integration.py --headed --url http://xezim-luna04-builder:4173/ --output-dir /tmp/surfer-artifacts
```

The build recipe, submodule SHAs, asset checksums, browser log, and screenshots are in the full branch-local report and its `spikes/surfer-embed/` files.

## Remaining limits

- The test covers one small fixture and Firefox/Camoufox. Chromium and a production Firefox build remain untested.
- Large-waveform performance and asynchronous worker behavior remain untested; the build logged a `ThreadPoolBuildError` in this container even though the fixture rendered.
- `zoom` is in the proposed adapter shape but was not verified. WCP source functions are not exposed to the parent by the tested Trunk glue.
- The delegated bundle and evidence remain on the isolated LUNA-00B branch; they are not copied into this LUNA-06 worktree. LUNA-05 should integrate only the verified operations and preserve the EUPL license.
