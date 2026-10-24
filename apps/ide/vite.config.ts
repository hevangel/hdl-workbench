import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { defineConfig } from "vitest/config";
import react from "@vitejs/plugin-react";

// The user guide lives in the repository's docs/ folder, outside this app's root.
// Serve it at /docs/index.html in development and copy it into production builds,
// so the IDE's help link works without keeping a second copy under public/.
const guidePath = fileURLToPath(new URL("../../docs/index.html", import.meta.url));
const guideUrls = new Set(["/docs", "/docs/", "/docs/index.html"]);

export default defineConfig({
  plugins: [
    react(),
    {
      name: "serve-workbench-guide",
      configureServer(server) {
        server.middlewares.use((request, response, next) => {
          if (!guideUrls.has(request.url?.split("?")[0] ?? "")) return next();
          response.setHeader("Content-Type", "text/html; charset=utf-8");
          response.end(readFileSync(guidePath));
        });
      },
      generateBundle() {
        this.emitFile({ type: "asset", fileName: "docs/index.html", source: readFileSync(guidePath) });
      },
    },
    {
      name: "omit-unused-browsercc-pch",
      apply: "build",
      generateBundle(_options, bundle) {
        // browsercc exposes an optional precompiled C++ header but compile()
        // does not load it. The 19 MiB PCH is therefore not part of this IDE.
        for (const [fileName, asset] of Object.entries(bundle)) {
          if (asset.type === "asset" && asset.fileName.endsWith(".pch")) delete bundle[fileName];
        }
      },
    },
  ],
  build: {
    rollupOptions: {
      input: {
        app: "index.html",
        "worker-harness": "worker-harness.html",
        "memory-harness": "memory-harness.html",
      },
    },
  },
  test: {
    environment: "jsdom",
    setupFiles: ["./test/setup.ts"],
    restoreMocks: true,
    clearMocks: true,
  },
});
