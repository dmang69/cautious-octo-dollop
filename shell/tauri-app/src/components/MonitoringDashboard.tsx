import { useCallback, useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useMetricsStream } from "../hooks/useMetricsStream";
import type {
  ConnectionStatus,
  LookupResult,
  SchedulerPolicy,
  SystemSnapshot,
} from "../types/grpc";

const DEFAULT_ENDPOINT = "http://127.0.0.1:50051";

export default function MonitoringDashboard() {
  const [endpoint, setEndpoint] = useState(DEFAULT_ENDPOINT);
  const [status, setStatus] = useState<ConnectionStatus>({
    connected: false,
    endpoint: DEFAULT_ENDPOINT,
  });
  const [snapshot, setSnapshot] = useState<SystemSnapshot | null>(null);
  const [policy, setPolicy] = useState<SchedulerPolicy | null>(null);
  const [lookupTarget, setLookupTarget] = useState("8.8.8.8");
  const [lookupResult, setLookupResult] = useState<LookupResult | null>(null);
  const [busy, setBusy] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);

  const { snapshot: streamSnapshot, streaming, error: streamError, start, stop } =
    useMetricsStream(1000);

  const run = useCallback(
    async <T,>(label: string, fn: () => Promise<T>) => {
      setBusy(label);
      setError(null);
      try {
        return await fn();
      } catch (e) {
        const msg = e instanceof Error ? e.message : String(e);
        setError(msg);
        throw e;
      } finally {
        setBusy(null);
      }
    },
    [],
  );

  const refreshStatus = useCallback(async () => {
    const s = await invoke<ConnectionStatus>("grpc_connection_status");
    setStatus(s);
  }, []);

  useEffect(() => {
    refreshStatus().catch(() => undefined);
  }, [refreshStatus]);

  const connect = () =>
    run("connect", async () => {
      const s = await invoke<ConnectionStatus>("grpc_connect", { endpoint });
      setStatus(s);
    });

  const disconnect = () =>
    run("disconnect", async () => {
      if (streaming) await stop();
      const s = await invoke<ConnectionStatus>("grpc_disconnect");
      setStatus(s);
    });

  const fetchSnapshot = () =>
    run("snapshot", async () => {
      const s = await invoke<SystemSnapshot>("grpc_get_system_snapshot");
      setSnapshot(s);
    });

  const fetchPolicy = () =>
    run("policy", async () => {
      const p = await invoke<SchedulerPolicy>("grpc_get_scheduler_policy");
      setPolicy(p);
    });

  const optimize = () =>
    run("optimize", async () => {
      const cpu = streamSnapshot?.cpuPercent ?? snapshot?.cpuPercent ?? 50;
      const p = await invoke<SchedulerPolicy>("grpc_optimize_scheduler_policy", {
        telemetry: [cpu],
      });
      setPolicy(p);
    });

  const lookup = () =>
    run("lookup", async () => {
      const r = await invoke<LookupResult>("grpc_lookup", { target: lookupTarget });
      setLookupResult(r);
    });

  const activeSnapshot = streamSnapshot ?? snapshot;

  return (
    <div className="page">
      <header className="page__header">
        <div>
          <p className="eyebrow">Telemetry</p>
          <h2 className="page__title">Dashboard</h2>
        </div>
        <div className={`status-pill ${status.connected ? "online" : "offline"}`}>
          {status.connected ? "Connected" : "Disconnected"}
        </div>
      </header>

      {(error || streamError) && (
        <div className="alert">{error ?? streamError}</div>
      )}

      <section className="panel">
        <h3>Connect</h3>
        <div className="row">
          <input
            value={endpoint}
            onChange={(e) => setEndpoint(e.target.value)}
            placeholder="gRPC endpoint"
          />
          <button onClick={() => connect()} disabled={!!busy || status.connected}>
            Connect
          </button>
          <button onClick={() => disconnect()} disabled={!!busy || !status.connected}>
            Disconnect
          </button>
        </div>
      </section>

      <section className="panel">
        <h3>Stream Metrics</h3>
        <div className="row">
          <button
            onClick={() => run("stream", start)}
            disabled={!!busy || !status.connected || streaming}
          >
            Start Stream
          </button>
          <button onClick={() => run("stop", stop)} disabled={!!busy || !streaming}>
            Stop Stream
          </button>
          <button onClick={() => fetchSnapshot()} disabled={!!busy || !status.connected}>
            One-shot Snapshot
          </button>
        </div>
        {activeSnapshot && (
          <div className="metrics-grid">
            <Metric label="CPU" value={`${activeSnapshot.cpuPercent.toFixed(1)}%`} />
            <Metric label="Memory" value={`${activeSnapshot.memPercent.toFixed(1)}%`} />
            <Metric label="Disk I/O" value={`${activeSnapshot.diskIoMbps.toFixed(2)} Mbps`} />
            <Metric label="Net RX" value={`${activeSnapshot.netRxMbps.toFixed(2)} Mbps`} />
            <Metric label="Net TX" value={`${activeSnapshot.netTxMbps.toFixed(2)} Mbps`} />
            <Metric label="Queue" value={String(activeSnapshot.queueDepth)} />
            <Metric label="Processes" value={String(activeSnapshot.processCount)} />
            <Metric label="IPC Queued" value={String(activeSnapshot.ipcQueued)} />
          </div>
        )}
      </section>

      <section className="panel">
        <h3>Lookup</h3>
        <div className="row">
          <input
            value={lookupTarget}
            onChange={(e) => setLookupTarget(e.target.value)}
            placeholder="target host or IP"
          />
          <button onClick={() => lookup()} disabled={!!busy || !status.connected}>
            Lookup
          </button>
        </div>
        {lookupResult && (
          <dl className="lookup-result">
            <div><dt>Verdict</dt><dd>{lookupResult.verdict}</dd></div>
            <div><dt>Threat</dt><dd>{lookupResult.threatLevel}</dd></div>
            <div><dt>Reputation</dt><dd>{lookupResult.reputationScore.toFixed(2)}</dd></div>
            <div><dt>Validated</dt><dd>{lookupResult.descramblerValidated ? "yes" : "no"}</dd></div>
          </dl>
        )}
      </section>

      <section className="panel">
        <h3>Scheduler Optimize</h3>
        <div className="row">
          <button onClick={() => fetchPolicy()} disabled={!!busy || !status.connected}>
            Get Policy
          </button>
          <button onClick={() => optimize()} disabled={!!busy || !status.connected}>
            Optimize
          </button>
        </div>
        {policy && (
          <p className="policy">
            Time slices (ms): {policy.timeSlicesMs.map((v) => v.toFixed(1)).join(", ")}
          </p>
        )}
      </section>

      {busy && <footer className="footer-busy">Working: {busy}…</footer>}
    </div>
  );
}

function Metric({ label, value }: { label: string; value: string }) {
  return (
    <div className="metric">
      <span className="metric__label">{label}</span>
      <span className="metric__value">{value}</span>
    </div>
  );
}