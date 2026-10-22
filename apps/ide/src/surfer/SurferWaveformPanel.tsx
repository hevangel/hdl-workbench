import { useEffect, useRef, useState } from "react";
import type { SurferAdapter } from "@xezim-ide/protocol";
import { PINNED_SURFER_VERSION, SurferAdapterError, SurferIframeAdapter, type WaveformUrlResolver } from "./SurferIframeAdapter.js";

interface SurferWaveformPanelProps {
  onAdapterChange?: (adapter: SurferAdapter | null) => void;
  resolveWaveformUrl?: WaveformUrlResolver;
}

export function SurferWaveformPanel({ onAdapterChange, resolveWaveformUrl }: SurferWaveformPanelProps) {
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
      ...(resolveWaveformUrl ? { resolveWaveformUrl } : {}),
      onReadyChange: (nextReady, error) => {
        setReady(nextReady);
        if (nextReady) setReadyGeneration((generation) => generation + 1);
        if (error) setMessage(error.message);
        else if (nextReady) setMessage("");
      },
    });
    adapterRef.current = adapter;
    void adapter.ready().then(() => {
      if (adapterRef.current === adapter) onAdapterChange?.(adapter);
    }).catch((error: unknown) => {
      setMessage(error instanceof SurferAdapterError ? error.message : "Surfer could not be initialized.");
    });
    return () => {
      adapter.dispose();
      adapterRef.current = null;
      onAdapterChange?.(null);
    };
  }, [onAdapterChange, resolveWaveformUrl]);

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
      <div className="surfer-attribution" aria-label="Surfer license and source information">
        <span>Surfer v{PINNED_SURFER_VERSION} · EUPL-1.2 · modified by Xezim Studio on 2026-10-22</span>
        <a href="/surfer/LICENSE-EUPL-1.2.txt" target="_blank" rel="noreferrer">License</a>
        <a href="https://gitlab.com/surfer-project/surfer/-/tree/bd749b1f786c1c62cd67893ca71346cbe6983915" target="_blank" rel="noreferrer">Pinned upstream source</a>
        <a href="/surfer/index.html" download="surfer-index.html">Modified entrypoint source</a>
        <a href="/surfer/UPSTREAM.md" target="_blank" rel="noreferrer">Provenance</a>
      </div>
    </section>
  );
}
