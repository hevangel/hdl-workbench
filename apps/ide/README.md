# Xezim Studio IDE shell

This is the LUNA-02 React/Vite shell. It uses Monaco from the pinned `monaco-editor` npm package and creates a same-origin editor Worker; the default public CDN loader is not used.

The current UI deliberately runs against an in-memory `WorkspaceAdapter` and a mock `SimulationService`. The header says **MOCK SERVICES · NO HDL SIMULATION**. The mock may return a clearly labeled fixture diagnostic, and valid input ends with `SIMULATOR_UNAVAILABLE`; it never claims that RTL ran or falls back to native simulation.

From the repository root:

```powershell
pnpm install --frozen-lockfile
pnpm --filter @xezim-ide/app dev
pnpm --filter @xezim-ide/app test
pnpm --filter @xezim-ide/app typecheck
pnpm --filter @xezim-ide/app build
```

The editor layout preference is stored separately from project data in localStorage. Source contents remain in the mock workspace and are not persistent across reloads.
