import { useCallback, useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { ConnectionStatus } from "../types/grpc";

type KernelStats = {
  active_capabilities: number;
  registered_handles: number;
  session_handles?: number;
};

type InvokeCapabilityResult = {
  allowed: boolean;
  resourceType: number;
  denialReason: string;
};

type FlowStep = "verify" | "register" | "invoke";

type StepState = {
  status: "idle" | "running" | "ok" | "error";
  message: string | null;
};

const INITIAL_STEP: StepState = { status: "idle", message: null };

export default function CapabilityPanel() {
  const [tokenPath, setTokenPath] = useState("/tmp/demo.token");
  const [handle, setHandle] = useState<string | null>(null);
  const [stats, setStats] = useState<KernelStats | null>(null);
  const [connected, setConnected] = useState(false);
  const [steps, setSteps] = useState<Record<FlowStep, StepState>>({
    verify: { ...INITIAL_STEP },
    register: { ...INITIAL_STEP },
    invoke: { ...INITIAL_STEP },
  });
  const [busy, setBusy] = useState(false);
  const [copied, setCopied] = useState(false);

  const setStep = (step: FlowStep, patch: Partial<StepState>) => {
    setSteps((prev) => ({ ...prev, [step]: { ...prev[step], ...patch } }));
  };

  const refreshConnection = useCallback(async () => {
    const s = await invoke<ConnectionStatus>("grpc_connection_status");
    setConnected(s.connected);
    return s.connected;
  }, []);

  const refreshStats = useCallback(async (remote?: boolean) => {
    const useRemote = remote ?? connected;
    if (useRemote) {
      const s = await invoke<KernelStats>("grpc_get_kernel_stats");
      setStats(s);
    } else {
      const s = await invoke<KernelStats>("kernel_stats");
      setStats(s);
    }
  }, [connected]);

  useEffect(() => {
    refreshConnection()
      .then((isConnected) => refreshStats(isConnected))
      .catch(() => undefined);
  }, [refreshConnection, refreshStats]);

  const runStep = async (step: FlowStep, fn: () => Promise<void>) => {
    setBusy(true);
    setStep(step, { status: "running", message: null });
    try {
      await fn();
    } catch (e) {
      const msg = e instanceof Error ? e.message : String(e);
      setStep(step, { status: "error", message: msg });
    } finally {
      setBusy(false);
    }
  };

  const verify = () =>
    runStep("verify", async () => {
      const msg = await invoke<string>("kernel_verify_token", { tokenPath });
      setStep("verify", { status: "ok", message: msg });
      setStep("register", { ...INITIAL_STEP });
      setStep("invoke", { ...INITIAL_STEP });
      setHandle(null);
    });

  const register = () =>
    runStep("register", async () => {
      const isConnected = await refreshConnection();
      const h = isConnected
        ? await invoke<string>("grpc_register_token", {
            tokenPath,
            resourceType: 1,
          })
        : await invoke<string>("kernel_register_token", {
            tokenPath,
            resourceType: 1,
          });
      setHandle(h);
      setStep("register", { status: "ok", message: `Handle ${h} registered` });
      setStep("invoke", { ...INITIAL_STEP });
      await refreshStats(isConnected);
    });

  const invokeHandle = () =>
    runStep("invoke", async () => {
      if (!handle) {
        throw new Error("No handle — register a token first");
      }
      const isConnected = await refreshConnection();
      const msg = isConnected
        ? await (async () => {
            const result = await invoke<InvokeCapabilityResult>("grpc_invoke_capability", {
              handle,
              action: 0,
            });
            return result.allowed
              ? `ALLOWED type=${result.resourceType}`
              : `DENIED: ${result.denialReason}`;
          })()
        : await invoke<string>("kernel_invoke", {
            handleHex: handle,
            action: 0,
          });
      setStep("invoke", { status: "ok", message: msg });
      await refreshStats(isConnected);
    });

  const copyHandle = async () => {
    if (!handle) return;
    try {
      await navigator.clipboard.writeText(handle);
      setCopied(true);
      window.setTimeout(() => setCopied(false), 2000);
    } catch {
      setCopied(false);
    }
  };

  const verifyDone = steps.verify.status === "ok";
  const registerDone = steps.register.status === "ok";

  return (
    <div className="page">
      <header className="page__header">
        <div>
          <p className="eyebrow">RFC-INTENT-001</p>
          <h2 className="page__title">Capabilities</h2>
        </div>
        <div className="page__actions">
          <div className={`status-pill ${connected ? "online" : "offline"}`}>
            {connected ? "Remote Runtime" : "Local Kernel"}
          </div>
          <button type="button" onClick={() => refreshStats()} disabled={busy}>
            Refresh stats
          </button>
        </div>
      </header>

      <section className="panel">
        <h3>Token path</h3>
        <div className="row">
          <input
            value={tokenPath}
            onChange={(e) => setTokenPath(e.target.value)}
            placeholder="Path to signed capability token"
          />
        </div>
      </section>

      <section className="panel">
        <h3>Capability flow</h3>
        <ol className="flow-steps">
          <FlowStepCard
            step={1}
            title="Verify"
            description="Validate broker signature and token TTL"
            state={steps.verify}
            actionLabel="Verify token"
            onAction={() => verify()}
            disabled={busy}
          />
          <FlowStepCard
            step={2}
            title="Register"
            description={
              connected
                ? "Register token with remote ai-runtime via gRPC"
                : "Load token into the local kernel gate"
            }
            state={steps.register}
            actionLabel="Register"
            onAction={() => register()}
            disabled={busy || !verifyDone}
            locked={!verifyDone}
          />
          <FlowStepCard
            step={3}
            title="Invoke"
            description="Execute syscall against registered handle"
            state={steps.invoke}
            actionLabel="Invoke"
            onAction={() => invokeHandle()}
            disabled={busy || !registerDone || !handle}
            locked={!registerDone}
          />
        </ol>
      </section>

      {handle && (
        <section className="panel">
          <h3>Active handle</h3>
          <div className="handle-row">
            <code className="handle-row__value">{handle}</code>
            <button type="button" className="btn-secondary" onClick={() => copyHandle()}>
              {copied ? "Copied" : "Copy"}
            </button>
          </div>
        </section>
      )}

      {stats && (
        <section className="panel">
          <h3>Kernel stats</h3>
          <div className="metrics-grid">
            <Metric label="Capabilities" value={String(stats.active_capabilities)} />
            <Metric label="Handles" value={String(stats.registered_handles)} />
            {stats.session_handles !== undefined && (
              <Metric label="Session" value={String(stats.session_handles)} />
            )}
          </div>
        </section>
      )}
    </div>
  );
}

function FlowStepCard({
  step,
  title,
  description,
  state,
  actionLabel,
  onAction,
  disabled,
  locked,
}: {
  step: number;
  title: string;
  description: string;
  state: StepState;
  actionLabel: string;
  onAction: () => void;
  disabled?: boolean;
  locked?: boolean;
}) {
  return (
    <li className={`flow-step flow-step--${state.status}${locked ? " flow-step--locked" : ""}`}>
      <div className="flow-step__head">
        <span className="flow-step__num">{step}</span>
        <div>
          <strong className="flow-step__title">{title}</strong>
          <p className="flow-step__desc">{description}</p>
        </div>
        <span className={`flow-step__badge flow-step__badge--${state.status}`}>
          {state.status === "idle" && (locked ? "Locked" : "Pending")}
          {state.status === "running" && "Running"}
          {state.status === "ok" && "Done"}
          {state.status === "error" && "Failed"}
        </span>
      </div>
      {state.message && (
        <div className={`flow-step__result${state.status === "error" ? " alert" : " notice"}`}>
          {state.message}
        </div>
      )}
      <button type="button" onClick={onAction} disabled={disabled}>
        {actionLabel}
      </button>
    </li>
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