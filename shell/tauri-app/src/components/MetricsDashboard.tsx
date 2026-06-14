import React, { useEffect, useState } from "react";
import {
  LineChart,
  Line,
  XAxis,
  YAxis,
  CartesianGrid,
  Tooltip,
  Legend,
  ResponsiveContainer,
} from "recharts";
import { invoke } from "@tauri-apps/api/tauri";

interface Snapshot {
  timestamp_ms: number;
  cpu_avg: number;
  memory_used_bytes: number;
  memory_total_bytes: number;
}

const MAX_POINTS = 60;

const MetricsDashboard: React.FC = () => {
  const [history, setHistory] = useState<Snapshot[]>([]);

  useEffect(() => {
    const id = setInterval(async () => {
      const snap: Snapshot = await invoke("get_telemetry_snapshot");
      setHistory((prev) => [...prev.slice(-MAX_POINTS + 1), snap]);
    }, 1000);
    return () => clearInterval(id);
  }, []);

  const data = history.map((s) => ({
    time: new Date(s.timestamp_ms).toLocaleTimeString(),
    cpu: (s.cpu_avg * 100).toFixed(1),
    memPct: ((s.memory_used_bytes / s.memory_total_bytes) * 100).toFixed(1),
  }));

  return (
    <div>
      <h2>System Metrics</h2>
      <ResponsiveContainer width="100%" height={300}>
        <LineChart data={data}>
          <CartesianGrid strokeDasharray="3 3" />
          <XAxis dataKey="time" />
          <YAxis domain={[0, 100]} unit="%" />
          <Tooltip />
          <Legend />
          <Line type="monotone" dataKey="cpu" name="CPU" stroke="#8884d8" dot={false} />
          <Line type="monotone" dataKey="memPct" name="Memory" stroke="#82ca9d" dot={false} />
        </LineChart>
      </ResponsiveContainer>
    </div>
  );
};

export default MetricsDashboard;
