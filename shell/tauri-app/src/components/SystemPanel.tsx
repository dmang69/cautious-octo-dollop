import { useCallback, useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { ConnectionStatus } from "../types/grpc";

type KernelStats = {
  active_capabilities: number;
  registered_handles: number;
  session_handles: number;
};

type BrokerKeyStatus = {
  present: boolean;
  algorithm: string | null;
  public_key_preview: string | null;
};

type ServiceHealth = {
  id: string;
  addr: string;
  reachable: boolean;
  detail: string;
};

type OsHealth = {
  version: string;
  root: string;
  broker: BrokerKeyStatus;
  runtime: ServiceHealth;
  verifier: ServiceHealth;
  healthy: boolean;
};

export default function SystemPanel() {
  const [health, setHealth] = useState<OsHealth | null>(null);
  const [stats, setStats] = useState<KernelStats | null>(null);
  const [grpc, setGrpc] = useState<ConnectionStatus | null>(null);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const refresh = useCallback(async () => {
    setBusy(true);
    setError(null);
    try {
      const [h, s, g] = await Promise.all([
        invoke<OsHealth>("os_health"),
        invoke<KernelStats>("kernel_stats"),
        invoke<ConnectionStatus>("grpc_connection_status"),
      ]);
      setHealth(h);
      setStats(s);
      setGrpc(g);
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setBusy(false);
    }
  }, []);

  useEffect(() => {
    refresh().catch(() => undefined);
  }, [refresh]);

  return (
    <div className="page">
      <header className="page__header">
        <div>
          <p className="eyebrow">Platform</p>
          <h2 className="page__title">System</h2>
        </div>
        <div className="page__actions">
          {health && (
            <div className={`status-pill ${health.healthy ? "online" : "offline"}`}>
              {health.healthy ? "Healthy" : "Degraded"}
            </div>
          )}
          <button type="button" onClick={() => refresh()} disabled={busy}>
            Refresh
          </button>
        </div>
      </header>

      {error && <div className="alert">{error}</div>}

      {health && (
        <section className="panel">
          <h3>Version &amp; Install</h3>
          <dl className="info-grid">
            <InfoItem label="Version" value={health.version} />
            <InfoItem label="Install root" value={health.root} mono />
          </dl>
        </section>
      )}

      {health && (
        <section className="panel">
          <h3>Broker Public Key</h3>
          <dl className="info-grid">
            <InfoItem
              label="Status"
              value={health.broker.present ? "Present" : "Missing"}
              status={health.broker.present ? "ok" : "bad"}
            />
            <InfoItem
              label="Algorithm"
              value={health.broker.algorithm ?? "—"}
            />
            <InfoItem
              label="Public key"
              value={health.broker.public_key_preview ?? "—"}
              mono
            />
          </dl>
          {!health.broker.present && (
            <p className="panel__note">
              Broker key not found at <code>config/broker.key.json</code>. Run{" "}
              <code>capd init</code> to generate signing keys.
            </p>
          )}
        </section>
      )}

      {stats && (
        <section className="panel">
          <h3>Kernel Stats</h3>
          <div className="metrics-grid">
            <Metric label="Capabilities" value={String(stats.active_capabilities)} />
            <Metric label="Handles" value={String(stats.registered_handles)} />
            <Metric label="Session" value={String(stats.session_handles)} />
          </div>
        </section>
      )}

      {health && (
        <section className="panel">
          <h3>Service Health</h3>
          <div className="service-list">
            <ServiceRow service={health.runtime} />
            <ServiceRow service={health.verifier} />
            {grpc && (
              <div className="service-row">
                <div>
                  <span className="service-row__name">gRPC client</span>
                  <span className="service-row__addr">{grpc.endpoint}</span>
                </div>
                <span className={`status-pill ${grpc.connected ? "online" : "offline"}`}>
                  {grpc.connected ? "Connected" : "Disconnected"}
                </span>
              </div>
            )}
          </div>
        </section>
      )}

      {busy && <footer className="footer-busy">Refreshing system status…</footer>}
    </div>
  );
}

function InfoItem({
  label,
  value,
  mono,
  status,
}: {
  label: string;
  value: string;
  mono?: boolean;
  status?: "ok" | "bad";
}) {
  return (
    <div className="info-item">
      <dt>{label}</dt>
      <dd className={mono ? "mono" : undefined} data-status={status}>
        {value}
      </dd>
    </div>
  );
}

function ServiceRow({ service }: { service: ServiceHealth }) {
  return (
    <div className="service-row">
      <div>
        <span className="service-row__name">{service.id}</span>
        <span className="service-row__addr">{service.addr}</span>
        <span className="service-row__detail">{service.detail}</span>
      </div>
      <span className={`status-pill ${service.reachable ? "online" : "offline"}`}>
        {service.reachable ? "Up" : "Down"}
      </span>
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