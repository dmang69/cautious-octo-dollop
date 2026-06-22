import { useState } from "react";
import CapabilityPanel from "./CapabilityPanel";
import MonitoringDashboard from "./MonitoringDashboard";
import SystemPanel from "./SystemPanel";

export type ShellView = "dashboard" | "capabilities" | "system";

const NAV: { id: ShellView; label: string; hint: string }[] = [
  { id: "dashboard", label: "Dashboard", hint: "Telemetry & gRPC" },
  { id: "capabilities", label: "Capabilities", hint: "Verify · Register · Invoke" },
  { id: "system", label: "System", hint: "Health & kernel" },
];

export default function AiOsShell() {
  const [view, setView] = useState<ShellView>("dashboard");

  return (
    <div className="ai-os-shell">
      <aside className="shell-sidebar">
        <div className="shell-sidebar__brand">
          <img className="shell-sidebar__logo" src="/logo-dark.png" alt="Intent Kernel" />
          <div>
            <p className="eyebrow">Intent Kernel</p>
            <h1 className="shell-sidebar__title">AI OS</h1>
          </div>
        </div>

        <nav className="shell-nav" aria-label="Main navigation">
          {NAV.map((item) => (
            <button
              key={item.id}
              type="button"
              className={`shell-nav__item${view === item.id ? " shell-nav__item--active" : ""}`}
              onClick={() => setView(item.id)}
              aria-current={view === item.id ? "page" : undefined}
            >
              <span className="shell-nav__label">{item.label}</span>
              <span className="shell-nav__hint">{item.hint}</span>
            </button>
          ))}
        </nav>

        <footer className="shell-sidebar__footer">
          <span>RFC-INTENT-001</span>
          <span>Zero ambient authority</span>
        </footer>
      </aside>

      <main className="shell-main">
        {view === "dashboard" && <MonitoringDashboard />}
        {view === "capabilities" && <CapabilityPanel />}
        {view === "system" && <SystemPanel />}
      </main>
    </div>
  );
}