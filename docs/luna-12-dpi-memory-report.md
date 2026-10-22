# LUNA-12 — C/C++ compiler browser-process profile

Date: 2026-10-22
Branch: `codex/luna-12-dpi-memory`

## Result

The selected workstation completed a first-use browser C/C++ compile and a
real Xezim DPI simulation in Chrome. The compiler built one C and one C++
translation unit, then the Xezim Worker returned `DPI_C=30` and
`DPI_CPP=42`. The profile also passed the existing Xezim memory-limit and
fresh-Worker recovery scenarios.

This is a measurement of the user's selected computer, not a minimum hardware
requirement. It does not qualify lower-memory devices, another OS, Firefox with
the new compiler, or public-network asset download time.

## Host and browser

| Field | Value |
| --- | --- |
| OS | Windows `10.0.26300` |
| CPU | AMD Ryzen 9 5950X, 32 logical CPUs |
| RAM | 63.9 GiB |
| Browser | Google Chrome `156.0.8078.47` |
| Browser process set | 7 processes, selected from Chrome DevTools process info |

The selected machine identity can be reproduced with:

```powershell
node -e 'const os=require("node:os"); console.log(JSON.stringify({platform:os.platform(),release:os.release(),totalRamGiB:+(os.totalmem()/1024**3).toFixed(1),logicalCpuCount:os.cpus().length,cpuModel:os.cpus()[0]?.model},null,2))'
```

## Memory and compile results

The profiler launches a fresh Playwright browser context and a local Vite
server. It samples the Chrome process set every 50 ms using `Get-Process`.
Resident and private bytes are independently sampled peaks and may occur at
different instants.

| Phase | Samples | Peak aggregate resident | Peak aggregate private |
| --- | ---: | ---: | ---: |
| IDE page loaded, before compile | 20 | 690.9 MiB | 470.5 MiB |
| First C/C++ compile and actual DPI run | 20 | 1,160.7 MiB | 1,426.9 MiB |
| Full profile, including the later Xezim memory-limit and recovery cases | 60 | 1,318.2 MiB | 1,426.9 MiB |

The C and C++ compile took 2,434 ms, produced two modules totaling 1,268 bytes,
and completed simulation with the expected output. This timing includes compiler
asset fetches from the local Vite server; it does not measure an internet-hosted
first download. The built compiler assets total 94,377,252 bytes (94.4 MB / 90.0
MiB) before transport compression. The unused 19.36 MB C++ PCH is not emitted.

The same profile observed a 256,114,688-byte direct Xezim WASM allocation
trap, a supported Worker run, `RESOURCE_LIMIT` for the over-limit Worker run,
and successful recovery in a fresh Worker.

## Reproduce

Use the Chrome executable installed on the selected workstation and allow the
profile script to open its local `127.0.0.1` Vite server:

```powershell
$env:XEZIM_CHROME_EXECUTABLE = 'C:\Program Files\Google\Chrome\Application\chrome.exe'
pnpm --filter @xezim-ide/mcp-bridge profile:browser-memory -- chromium
```

The script in `services/mcp-bridge/scripts/profile-browser-memory.mjs` runs the
C/C++ and DPI scenario before the Xezim memory stress cases. It verifies the
compile result, both DPI output values, the memory-limit response, and recovery
before reporting the samples.

## Limits and next release checks

- Process-memory figures are workload-specific sampled peaks. They are not
  absolute OS high-water marks or a guarantee for arbitrary C/C++ inputs.
- The test used a local Vite server, so it does not establish public CDN
  transfer time, compression, cache behavior, or production-origin availability.
- The new compiler path has been measured in Chrome only. The earlier LUNA-10
  Firefox memory profile predates the C/C++ compiler assets.
- The additional compiler files now have a technical license inventory and
  copied license texts in `apps/ide/public/licenses/`. The exact Emscripten
  version used by the upstream npm build is not recorded, and a qualified
  combined-work review remains a public-release gate.
- No canonical public deployment origin is configured or verified.
