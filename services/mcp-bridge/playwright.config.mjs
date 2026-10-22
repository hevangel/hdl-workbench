import path from "node:path";
import { fileURLToPath } from "node:url";
import { defineConfig } from "@playwright/test";

const packageRoot = path.dirname(fileURLToPath(import.meta.url));
const repositoryRoot = path.resolve(packageRoot, "../..");
const browserProjects = [
  {
    name: "chromium",
    use: {
      browserName: "chromium",
      launchOptions: {
        args: ["--enable-webgl", "--use-gl=angle", "--use-angle=swiftshader", "--enable-unsafe-swiftshader"],
      },
    },
  },
  { name: "firefox", use: { browserName: "firefox" } },
];
const selectedProject = process.env.PLAYWRIGHT_PROJECT;
const projects = selectedProject
  ? browserProjects.filter((project) => project.name === selectedProject)
  : browserProjects;
if (selectedProject && projects.length === 0) throw new Error(`Unknown PLAYWRIGHT_PROJECT: ${selectedProject}`);

export default defineConfig({
  testDir: "./e2e",
  timeout: 180_000,
  expect: { timeout: 30_000 },
  workers: 1,
  reporter: "line",
  projects,
  use: { baseURL: "http://127.0.0.1:5174" },
  webServer: {
    command: "pnpm exec vite --host 127.0.0.1 --port 5174 --strictPort",
    cwd: path.join(repositoryRoot, "apps/ide"),
    url: "http://127.0.0.1:5174",
    reuseExistingServer: !process.env.CI,
    timeout: 60_000,
  },
});
