# Surfer WASM asset provenance

- Upstream: Surfer v0.7.0, commit `bd749b1f786c1c62cd67893ca71346cbe6983915`.
- Source license: EUPL-1.2. The upstream license text is retained as `LICENSE-EUPL-1.2.txt`.
- Browser WASM SHA-256: `1341ad01c59f5169209239728417d14f430183abd92515f22bf07cd7c86937ae`.
- Other unmodified upstream asset hashes are listed in `UPSTREAM-SHA256SUMS`.
- Assets were copied from the completed LUNA-00B feasibility branch (`65bc8f0323c8c31122175c8a4d6fe4e45762132b`). Its build used Rust/Cargo 1.99.1, `wasm32-unknown-unknown`, and Trunk 0.21.14.

`index.html` retains the same-origin relative asset paths and WASM-ready notification from the feasibility build. Xezim Studio adds a nonce-based readiness probe listener so the parent can recover readiness after iframe reloads and reject messages from other origins. Its local SHA-256 is recorded in `LOCAL-WRAPPER-SHA256SUMS`; the original upstream hash remains in `UPSTREAM-SHA256SUMS`. The iframe uses the upstream `#dev` switch to skip Surfer's service-worker registration; the pinned worker rewrites response headers but does not cache files, while the IDE will own app-level offline behavior. The WASM and generated `surfer.js` are unchanged. `integration.js` is the pinned wrapper; its `InjectMessage` payloads are unstable upstream details and are used only inside `apps/ide/src/surfer/SurferIframeAdapter.ts`.
