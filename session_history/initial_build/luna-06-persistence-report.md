# LUNA-06 — Persistent browser workspace

Date: 2026-10-22
Branch: `codex/luna-06-persistence`
Status: implementation and the available Firefox persistence acceptance passed.

## Implemented

- Browser projects persist source bytes in OPFS and project metadata, active-project selection, settings, and editor session state in IndexedDB. Web Locks serialize writes and a `BroadcastChannel` announces cross-tab changes.
- Writes use revisions and explicit conflict handling. A stale editor draft stays visible until the user reloads the stored version or confirms replacing it. Debounced saves are canceled when resolving a conflict.
- The IDE can create/select projects, save, import/export ZIP archives, rename or delete the active file, and rename or delete projects. ZIP import accepts stored and deflate-raw entries, validates paths/CRC/sizes/count, and handles data descriptors.
- Storage initialization errors are shown to the user. There is no in-memory persistence fallback. Quota errors keep the editor draft available for recovery. The browser persistence request is advisory; the app still reports when browser storage may be cleared.
- Browser simulation remains visibly marked as mock-only, and Connected mode stays disabled until an MCP bridge is configured.

## Limits

- The UI test injects a quota-style `WorkspaceStorageError`; it does not exhaust the browser's physical OPFS quota or prove behavior under actual disk-full conditions.
- The static Camoufox acceptance harness did not execute the page's entry module on ordinary navigation. The test imports the served entry module itself after each navigation (five workarounds in the passing run). This is a test-harness limitation, but normal startup still needs verification in a supported production browser.
- The browser acceptance used Camoufox Firefox `152.0.4-beta.30`, not production Firefox or Chromium. Cross-tab tests used two pages in one browser context.
- Storage initialization awaits `navigator.storage.persist()` when that API exists. In the test browser the nonblocking `.catch()` form caused `Permission denied to access property "then"`; the awaited form passed. This observed behavior is specific to the current Firefox test environment and should be checked in supported browsers.
- ZIP import/export is limited to the implemented bounded ZIP32 subset. Archives exceeding the documented per-file, project, or file-count limits are rejected.

## Reproduction and results

Run from the repository root:

```sh
pnpm --filter @xezim-ide/app typecheck
pnpm --filter @xezim-ide/app test
pnpm --filter @xezim-ide/app build
git diff --check
```

Results on this branch: typecheck passed; 14 tests across 3 files passed; production build passed. Vite reports the existing large Monaco application chunk (about 3.08 MB minified / 808 KB gzip).

The real-browser acceptance script is `apps/ide/scripts/workspace-browser-acceptance.py`. In the provisioned `xezim-luna04-builder` / `kb_camoufox` Docker network used for this run, start the static servers, stage the built site, and serve that site on browser-container loopback:

```powershell
docker exec -d xezim-luna04-builder python3 -m http.server 4180 --bind 0.0.0.0 --directory /host/apps/ide/dist
docker exec -d xezim-luna04-builder python3 -m http.server 4175 --bind 0.0.0.0 --directory /host/apps/ide/scripts
docker exec kb_camoufox python3 -c 'import urllib.request; source=urllib.request.urlopen("http://xezim-luna04-builder:4175/stage-browser-site.py").read(); exec(compile(source,"stage-browser-site.py","exec"), {"__name__":"__main__"})'
docker exec -d kb_camoufox python3 -m http.server 4216 --bind 127.0.0.1 --directory /tmp/luna06-local
```

Then run the acceptance test:

```powershell
docker exec kb_camoufox python3 -c 'import os,urllib.request; os.environ["DISPLAY"] = ":99"; os.environ["XEZIM_WORKSPACE_TEST_URL"] = "http://127.0.0.1:4216/"; source=urllib.request.urlopen("http://xezim-luna04-builder:4175/workspace-browser-acceptance.py").read(); exec(compile(source,"workspace-browser-acceptance.py","exec"), {"__name__":"__main__"})'
```

```text
Browser version: 152.0.4-beta.30
PASS: reload, save receipt, cross-tab conflict, explicit replace, ZIP export/import, project selection, rename, and delete (Camoufox entry-import workarounds: 5)
```

The same acceptance run was repeated after restoring the awaited persistence request; the initial run with nonblocking `.catch()` failed to load the project tree and was not counted as passing.

## Recommended next tasks

1. Verify ordinary page-entry startup and persistence in production Chromium and Firefox, without the Camoufox harness import workaround.
2. Exercise real storage exhaustion, browser eviction behavior, and recovery in a controlled browser profile.
3. Implement LUNA-05 against the pinned Surfer build, enabling only the already verified load, signal-add, and cursor operations; keep zoom disabled until it passes its acceptance gate.
4. Complete the Xezim `SimulationService` mapping, memory/work enforcement, and MCP bridge before enabling non-mock simulation or connected mode.
