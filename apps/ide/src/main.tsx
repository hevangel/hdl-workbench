import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import * as monaco from "monaco-editor/editor/editor.api.js";
import { loader } from "@monaco-editor/react";
import editorWorker from "monaco-editor/editor/editor.worker.js?worker";
import { App } from "./App.js";
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

createRoot(rootElement).render(
  <StrictMode>
    <App />
  </StrictMode>,
);
