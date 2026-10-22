# Surfer distribution review

Date: 2026-10-22

Status: technical inventory and visible Surfer notice complete; qualified release review remains open.

This document records the shipped asset and the applicable text of the upstream license. It does not decide whether the combined Xezim Studio application is a derivative work or approve a distribution model.

## Asset and source inventory

- The embedded viewer is Surfer v0.7.0 at commit `bd749b1f786c1c62cd67893ca71346cbe6983915`.
- Upstream identifies Surfer as EUPL-1.2 licensed. Its pinned source tree is [available on GitLab](https://gitlab.com/surfer-project/surfer/-/tree/bd749b1f786c1c62cd67893ca71346cbe6983915).
- The complete EUPL-1.2 text is bundled as [`LICENSE-EUPL-1.2.txt`](../apps/ide/public/surfer/LICENSE-EUPL-1.2.txt).
- Xezim Studio modified the upstream `index.html` on 2026-10-22 to add a same-origin, nonce-based iframe readiness probe. The original and local hashes are recorded separately; all other listed assets match their upstream hashes. The [asset provenance note](../apps/ide/public/surfer/UPSTREAM.md) records the exact change and pin.
- The bundle is served inside a same-origin iframe. The application communicates with it through the version-specific adapter. This describes the architecture only; it does not establish the legal boundary between separate or combined works.

## License requirements relevant to deployment

The [official EUPL-1.2 text](https://interoperable-europe.ec.europa.eu/sites/default/files/custom-page/attachment/2020-03/EUPL-1.2%20EN.txt) defines distribution/communication to include making copies available online and providing access to essential functionality (Article 1). Article 3 says executable code must be accompanied by a machine-readable source copy or a notice naming a repository where source remains easily and freely accessible while the work is distributed. Article 5 requires preservation of copyright and license notices, a copy of the license, prominent modification notices with modification dates, and provision of source or an accessible repository. Article 11 also addresses information provided by the electronic distribution channel.

The European Commission's [EUPL guidance](https://interoperable-europe.ec.europa.eu/collection/eupl/how-use-eupl) explains that public web access is within the license's communication language and discusses source availability for modified works offered as a service. The EUPL's treatment of merged works and the applicable law can depend on the facts; an iframe boundary alone is not a legal conclusion.

## Current release-channel gaps

- The license text and provenance are in the repository's static Surfer asset directory. Source files are in this repository, and the pinned upstream source is linked above.
- The Surfer panel now presents the pinned version, EUPL-1.2, the local modification date, and links to the license and source/provenance note. Unit and browser acceptance checks cover those links and text.
- The public deployment URL and the continued public availability of the corresponding source repository have not been specified. The release process must keep the license and source/provenance URLs available for as long as the viewer is distributed or communicated.
- The runtime bundle contains generated WASM and JavaScript. A release should point recipients to the pinned upstream source and preserve access to the local HTML modification and exact build provenance for as long as the bundle is served.

These are release checks, not findings that distribution is prohibited. Before external distribution, choose the deployment/package model and obtain a qualified review of the bundled viewer and the combined application's license position.

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
