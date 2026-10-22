# Surfer WASM asset provenance

- Upstream: Surfer v0.7.0, commit `bd749b1f786c1c62cd67893ca71346cbe6983915`.
- Pinned source tree: <https://gitlab.com/surfer-project/surfer/-/tree/bd749b1f786c1c62cd67893ca71346cbe6983915>.
- Source license: EUPL-1.2. The upstream license text is retained as `LICENSE-EUPL-1.2.txt`.
- The locally modified browser entry point is [`index.html`](./index.html), served as `/surfer/index.html`. The upstream JavaScript and WASM assets are unmodified; their source is the pinned GitLab tree above.
- Browser WASM SHA-256: `1341ad01c59f5169209239728417d14f430183abd92515f22bf07cd7c86937ae`.
- Other unmodified upstream asset hashes are listed in `UPSTREAM-SHA256SUMS`.
- Assets were copied from the completed LUNA-00B feasibility branch (`65bc8f0323c8c31122175c8a4d6fe4e45762132b`). Its build used Rust/Cargo 1.99.1, `wasm32-unknown-unknown`, and Trunk 0.21.14.

Xezim Studio modified the upstream `index.html` on 2026-10-22 (commit `b3061a3`) to add a nonce-based readiness probe. This lets the parent recover readiness after iframe reloads and reject messages from other origins. The original upstream hash is recorded in `UPSTREAM-SHA256SUMS`; the modified file hash is recorded in `LOCAL-WRAPPER-SHA256SUMS`. The remaining listed assets, including the WASM and generated `surfer.js`, are unchanged. `integration.js` is the pinned wrapper; its `InjectMessage` payloads are unstable upstream details and are used only inside `apps/ide/src/surfer/SurferIframeAdapter.ts`.

The iframe uses the upstream `#dev` switch to skip Surfer's service-worker registration; the pinned worker rewrites response headers but does not cache files, while the IDE owns app-level offline behavior. Distribution obligations and the notice/source presentation in the release channel are tracked in [`docs/surfer-distribution-review.md`](../../../../docs/surfer-distribution-review.md).
