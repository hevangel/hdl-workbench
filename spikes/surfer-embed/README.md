# Surfer browser integration smoke harness

This package-free harness observes the upstream browser integration protocol. It embeds the public Surfer app, serves a real counter VCD with CORS enabled, and sends the message shapes observed in pinned source. The public app is unpinned; a green run here is not release acceptance for vendored Surfer assets.

## Run

In PowerShell:

    cd .\spikes\surfer-embed
    node .\server.mjs

Open http://127.0.0.1:4173/ in Chromium and wait at least 15 seconds. The page logs each posted message. Inspect the Surfer frame for the top.count waveform and a cursor at tick 25. The fixture uses a 1 ns timescale and transitions from 0 to 15 every 5 ns.

The script waits after the iframe document's load event because the pinned source provides no documented readiness callback for a parent page. This delay is only a smoke-test accommodation; production should add a readiness hook when building pinned assets. This harness has not passed in the available environment: the browser runner cannot reach the local loopback server, and the hosted iframe is not pinned or same-origin. See `session_history/initial_build/surfer-integration-report.md` before treating any message as supported.

## Pin and license

- Upstream: https://gitlab.com/surfer-project/surfer
- Release: v0.7.0
- Source SHA: bd749b1f786c1c62cd67893ca71346cbe6983915
- License: EUPL-1.2; preserve upstream LICENSE-EUPL-1.2.txt beside redistributed assets.
- Upstream documents its WASM artifact as the pages_build CI artifact, not as a v0.7.0 release asset. The v0.7.0 tag artifact URL returned GitLab 404 during this spike. The source can be built at the pinned SHA, but no reproducible built-asset digest was obtained in this environment.

Production should serve the locally built files under this app's origin and iframe that same-origin path. This harness deliberately uses the public origin so it can inspect the candidate message flow while the pinned artifact is unavailable; it does not validate same-origin packaging or successful message handling.
