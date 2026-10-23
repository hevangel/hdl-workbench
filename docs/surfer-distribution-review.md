# Surfer distribution review

Date: 2026-10-22

Status: technical inventory and served notice complete. The selected channel is a public hosted website; deployment-source publication and qualified release review remain open.

This document records the shipped asset and the applicable text of the upstream license. It does not decide whether the combined Xezim Studio application is a derivative work or approve a distribution model.

## Asset and source inventory

- The embedded viewer is Surfer v0.7.0 at commit `bd749b1f786c1c62cd67893ca71346cbe6983915`.
- Upstream identifies Surfer as EUPL-1.2 licensed. Its pinned source tree is [available on GitLab](https://gitlab.com/surfer-project/surfer/-/tree/bd749b1f786c1c62cd67893ca71346cbe6983915).
- The complete EUPL-1.2 text is bundled as [`LICENSE-EUPL-1.2.txt`](../apps/ide/public/surfer/LICENSE-EUPL-1.2.txt).
- Xezim Studio modified the upstream `index.html` on 2026-10-22 to add a same-origin, nonce-based iframe readiness probe. The original and local hashes are recorded separately; all other listed assets match their upstream hashes. The [asset provenance note](../apps/ide/public/surfer/UPSTREAM.md) records the exact change and pin.
- The bundle is served inside a same-origin iframe. The application communicates with it through the version-specific adapter. This describes the architecture only; it does not establish the legal boundary between separate or combined works.

## License requirements relevant to deployment

The [official EUPL-1.2 text](https://interoperable-europe.ec.europa.eu/collection/eupl/eupl-text-eupl-12) defines “Distribution or Communication” to include making copies available online or providing access to essential functionality (Article 1). Article 3 permits executable-code delivery when a machine-readable source copy is provided or a notice identifies a repository where source stays easily and freely available while the work is communicated. Article 5 requires retained copyright/license/warranty notices, a license copy, prominent modification notices with dates, and source availability. Article 11 addresses information the electronic distribution channel must provide. The Commission's [EUPL guidance](https://interoperable-europe.ec.europa.eu/collection/eupl/how-use-eupl) expressly discusses public web services and source availability for modified EUPL software. This is a technical compliance checklist, not a legal conclusion that Xezim Studio as a whole is a derivative work or that every source obligation applies to the combined application.

The European Commission's [EUPL guidance](https://interoperable-europe.ec.europa.eu/collection/eupl/how-use-eupl) explains that public web access is within the license's communication language and discusses source availability for modified works offered as a service. The EUPL's treatment of merged works and the applicable law can depend on the facts; an iframe boundary alone is not a legal conclusion.

## Current release-channel gaps

- The Surfer panel presents the pinned version, EUPL-1.2, the local modification date, and direct links to the license, pinned upstream source, downloadable modified `index.html`, and provenance note. Browser acceptance checks fetch and verify the served license, provenance, and modified entrypoint in Chromium and Firefox.
- The release channel is a public hosted website, but no canonical deployment origin has been supplied and this branch has not been published. Consequently, the current source revision is not yet available from a public source repository and production URLs have not been verified.
- Before launch, publish a versioned source repository or source package containing the exact local wrapper and build/provenance records; keep the pinned upstream source link, `/surfer/UPSTREAM.md`, `/surfer/LICENSE-EUPL-1.2.txt`, and `/surfer/index.html` publicly reachable without authentication for as long as the viewer is communicated. Run a production-origin check for all four URLs and retain the tested origin with the release record.
- The runtime bundle contains generated WASM and JavaScript. Those assets are unchanged from the pinned upstream build; the local HTML entrypoint change and its date/hash are recorded separately. A legal reviewer must confirm whether the hosted app's integration or any combined-work facts create broader source/licensing obligations.

These are release checks, not findings that distribution is prohibited. The channel choice is recorded; release still needs an actual deployment origin, durable public source access, and qualified review of the bundled viewer and the combined application's license position.

## Reproducible provenance checks

Run from the repository root in PowerShell:

```powershell
$badHashes = @()
foreach ($line in Get-Content apps/ide/public/surfer/UPSTREAM-SHA256SUMS) {
  $parts = $line -split '\s+', 2
  $relative = $parts[1] -replace '^surfer/', ''
  if ($relative -eq 'index.html') { continue }
  $file = Join-Path 'apps/ide/public/surfer' $relative
  $actual = (Get-FileHash -LiteralPath $file -Algorithm SHA256).Hash.ToLowerInvariant()
  if ($actual -ne $parts[0]) { $badHashes += "$relative expected=$($parts[0]) actual=$actual" }
}
foreach ($line in Get-Content apps/ide/public/surfer/LOCAL-WRAPPER-SHA256SUMS) {
  $parts = $line -split '\s+', 2
  $file = Join-Path 'apps/ide/public/surfer' $parts[1]
  $actual = (Get-FileHash -LiteralPath $file -Algorithm SHA256).Hash.ToLowerInvariant()
  if ($actual -ne $parts[0]) { $badHashes += "$($parts[1]) expected=$($parts[0]) actual=$actual" }
}
if ($badHashes.Count) { $badHashes; throw 'Surfer asset provenance check failed.' }
'All unchanged upstream assets and the local wrapper hash match.'
```

The original upstream `index.html` was also checked against the delegated feasibility branch:

```powershell
node -e 'const crypto=require("node:crypto"),cp=require("node:child_process"); const b=cp.execFileSync("git",["show","codex/luna-00b-surfer-integration:spikes/surfer-embed/assets/surfer/index.html"]); console.log(crypto.createHash("sha256").update(b).digest("hex"));'
```

Expected original upstream hash: `a007033bc838f624f8172d547401654c45fece88d4c0ea6c53d456a90931ebb7`. Expected local modified `index.html` hash: `8c30e0d542fab3afd265b2dac308cd3173f03ec9520eae0bef2251215d1cffaa`. The upstream index entry is intentionally skipped by the first check because the deployed file is modified; the local hash check covers it separately.
