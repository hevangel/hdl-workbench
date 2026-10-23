import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import * as monaco from "monaco-editor/editor/editor.api.js";
import { loader } from "@monaco-editor/react";
import editorWorker from "monaco-editor/editor/editor.worker.js?worker";
import { App } from "./App.js";
import { initialProjectFiles } from "./mockServices.js";
import { openPersistentWorkspace } from "./workspace/opfsWorkspace.js";
import { BrowserIdeController } from "./controller/BrowserIdeController.js";
import "./styles.css";

// Keep the editor and its Worker self-hosted with the app. The default loader
// otherwise points to a public CDN, which is unsuitable for an offline IDE.
const browserWindow = window as Window & {
  MonacoEnvironment: { getWorker(workerId: string, label: string): Worker };
};
browserWindow.MonacoEnvironment = {
  getWorker: () => new editorWorker(),
};
loader.config({ monaco });

const rootElement = document.getElementById("root");
if (!rootElement) throw new Error("The IDE root element is missing.");

const root = createRoot(rootElement);

async function hasWasmAssets(): Promise<boolean> {
  try {
    const [moduleResponse, wasmResponse] = await Promise.all([
      fetch("/wasm/xezim_web.js", { method: "HEAD", cache: "no-store" }),
      fetch("/wasm/xezim_web_bg.wasm", { method: "HEAD", cache: "no-store" }),
    ]);
    return moduleResponse.ok
      && wasmResponse.ok
      && moduleResponse.headers.get("content-type")?.includes("javascript") === true
      && wasmResponse.headers.get("content-type")?.includes("application/wasm") === true;
  } catch {
    return false;
  }
}

async function startIde(): Promise<void> {
  try {
    const manager = await openPersistentWorkspace(initialProjectFiles);
    const controller = new BrowserIdeController(manager.getWorkspace(), { simulatorAvailable: await hasWasmAssets() });
    window.addEventListener("pagehide", () => controller.dispose(), { once: true });
    root.render(
      <StrictMode>
        <App projectManager={manager} workspace={manager.getWorkspace()} controller={controller} />
      </StrictMode>,
    );
  } catch (error) {
    root.render(
      <main className="storage-startup-error" role="alert">
        <h1>Browser storage is unavailable</h1>
        <p>{error instanceof Error ? error.message : "The persistent workspace could not be opened."}</p>
        <p>Xezim Studio did not switch to an in-memory workspace. Try a supported browser on localhost or HTTPS, then reload.</p>
        <button type="button" onClick={() => void startIde()}>Retry storage setup</button>
      </main>,
    );
  }
}

void startIde();
