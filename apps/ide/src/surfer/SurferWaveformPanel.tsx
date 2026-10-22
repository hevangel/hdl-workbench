import { useEffect, useRef, useState } from "react";
import type { SurferAdapter } from "@xezim-ide/protocol";
import { PINNED_SURFER_VERSION, SurferAdapterError, SurferIframeAdapter } from "./SurferIframeAdapter.js";

interface SurferWaveformPanelProps {
  onAdapterChange?: (adapter: SurferAdapter | null) => void;
}

export function SurferWaveformPanel({ onAdapterChange }: SurferWaveformPanelProps) {
  const iframeRef = useRef<HTMLIFrameElement>(null);
  const adapterRef = useRef<SurferIframeAdapter | null>(null);
  const [ready, setReady] = useState(false);
  const [readyGeneration, setReadyGeneration] = useState(0);
  const [loadedSample, setLoadedSample] = useState(false);
  const [pending, setPending] = useState(false);
  const [message, setMessage] = useState("");

  useEffect(() => {
    const iframe = iframeRef.current;
    if (!iframe) return;
    const adapter = new SurferIframeAdapter(iframe, {
      onReadyChange: (nextReady, error) => {
        setReady(nextReady);
        if (nextReady) setReadyGeneration((generation) => generation + 1);
        if (error) setMessage(error.message);
        else if (nextReady) setMessage("");
      },
    });
    adapterRef.current = adapter;
    onAdapterChange?.(adapter);
    void adapter.ready().catch((error: unknown) => {
      setMessage(error instanceof SurferAdapterError ? error.message : "Surfer could not be initialized.");
    });
    return () => {
      adapter.dispose();
      adapterRef.current = null;
      onAdapterChange?.(null);
    };
  }, [onAdapterChange]);

  const perform = async (action: () => Promise<void>, after?: () => void) => {
    setPending(true);
    setMessage("");
    try {
      await action();
      after?.();
    } catch (error) {
      setMessage(error instanceof Error ? error.message : "The Surfer operation failed.");
    } finally {
      setPending(false);
    }
  };

  const adapter = adapterRef.current;
  const sampleUrl = new URL("/fixtures/counter.vcd", window.location.origin).href;

  return (
    <section className="surfer-panel" aria-label="Surfer waveform panel">
      <div className="surfer-toolbar">
        <span className={`surfer-readiness ${ready ? "ready" : "loading"}`} role="status">
          <span data-ready-generation={readyGeneration}>{ready ? `Surfer v${PINNED_SURFER_VERSION} ready` : `Loading Surfer v${PINNED_SURFER_VERSION}…`}</span>
        </span>
        <button type="button" onClick={() => void perform(() => adapter?.loadUrl(sampleUrl) ?? Promise.reject(new Error("Surfer is not ready.")), () => setLoadedSample(true))} disabled={!ready || pending}>
          Load sample VCD
        </button>
        <button type="button" onClick={() => void perform(() => adapter?.addSignals(["top.count"]) ?? Promise.reject(new Error("Surfer is not ready.")))} disabled={!ready || !loadedSample || pending}>
          Add top.count
        </button>
        <button type="button" onClick={() => void perform(() => adapter?.setCursor({ ticks: "25" }) ?? Promise.reject(new Error("Surfer is not ready.")))} disabled={!ready || !loadedSample || pending}>
          Set cursor 25 ns
        </button>
        <button type="button" onClick={() => void perform(() => adapter?.zoom({ ticks: "0" }, { ticks: "40" }) ?? Promise.reject(new Error("Surfer is not ready.")))} disabled={!ready || !loadedSample || pending}>
          Zoom 0–40 ns
        </button>
        {message && <span className="surfer-message" role="alert">{message}</span>}
      </div>
      <iframe
        ref={iframeRef}
        id="surferFrame"
        className="surfer-frame"
        src="/surfer/index.html#dev"
        title="Surfer waveform viewer"
        referrerPolicy="no-referrer"
        allow="fullscreen"
      />
    </section>
  );
}
