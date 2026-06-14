import React, { useState } from "react";
import Terminal from "./components/Terminal";
import ProcessTable from "./components/ProcessTable";
import MetricsDashboard from "./components/MetricsDashboard";

type View = "terminal" | "processes" | "metrics";

const App: React.FC = () => {
  const [view, setView] = useState<View>("terminal");

  return (
    <div className="app">
      <nav>
        <button onClick={() => setView("terminal")} aria-current={view === "terminal"}>
          Terminal
        </button>
        <button onClick={() => setView("processes")} aria-current={view === "processes"}>
          Processes
        </button>
        <button onClick={() => setView("metrics")} aria-current={view === "metrics"}>
          Metrics
        </button>
      </nav>
      <main>
        {view === "terminal" && <Terminal />}
        {view === "processes" && <ProcessTable />}
        {view === "metrics" && <MetricsDashboard />}
      </main>
    </div>
  );
};

export default App;
