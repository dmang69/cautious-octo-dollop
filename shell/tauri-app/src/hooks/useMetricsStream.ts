import { useCallback, useEffect, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type { SystemSnapshot } from "../types/grpc";

export function useMetricsStream(intervalMs = 1000) {
  const [snapshot, setSnapshot] = useState<SystemSnapshot | null>(null);
  const [streaming, setStreaming] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const unlisten = useRef<UnlistenFn[]>([]);

  const start = useCallback(async () => {
    await invoke("grpc_start_metrics_stream", { intervalMs });
    setStreaming(true);
    setError(null);
  }, [intervalMs]);

  const stop = useCallback(async () => {
    await invoke("grpc_stop_metrics_stream");
    setStreaming(false);
  }, []);

  useEffect(() => {
    let alive = true;
    (async () => {
      unlisten.current = [
        await listen<SystemSnapshot>("metrics-snapshot", (e) => {
          if (alive) setSnapshot(e.payload);
        }),
        await listen<string>("metrics-error", (e) => {
          if (alive) { setError(e.payload); setStreaming(false); }
        }),
      ];
    })();
    return () => { alive = false; unlisten.current.forEach((u) => u()); };
  }, []);

  return { snapshot, streaming, error, start, stop };
}