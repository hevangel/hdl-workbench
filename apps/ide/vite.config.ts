import { defineConfig } from "vitest/config";
import react from "@vitejs/plugin-react";

export default defineConfig({
  plugins: [
    react(),
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
