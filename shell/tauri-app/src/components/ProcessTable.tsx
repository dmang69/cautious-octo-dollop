import React, { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/tauri";

interface Process {
  pid: number;
  name: string;
  cpu_usage: number;
  memory_bytes: number;
  priority: number;
  suggested_priority?: number;
}

const ProcessTable: React.FC = () => {
  const [processes, setProcesses] = useState<Process[]>([]);

  useEffect(() => {
    const fetchProcesses = async () => {
      const procs: Process[] = await invoke("list_processes");
      setProcesses(procs);
    };
    fetchProcesses();
    const id = setInterval(fetchProcesses, 2000);
    return () => clearInterval(id);
  }, []);

  const applyPriority = async (pid: number, priority: number) => {
    await invoke("set_priority", { pid, priority });
  };

  return (
    <table>
      <thead>
        <tr>
          <th>PID</th>
          <th>Name</th>
          <th>CPU %</th>
          <th>Memory</th>
          <th>Priority</th>
          <th>AI Suggestion</th>
        </tr>
      </thead>
      <tbody>
        {processes.map((p) => (
          <tr key={p.pid}>
            <td>{p.pid}</td>
            <td>{p.name}</td>
            <td>{p.cpu_usage.toFixed(1)}</td>
            <td>{(p.memory_bytes / 1024 / 1024).toFixed(1)} MB</td>
            <td>{p.priority}</td>
            <td>
              {p.suggested_priority !== undefined && p.suggested_priority !== p.priority ? (
                <button onClick={() => applyPriority(p.pid, p.suggested_priority!)}>
                  Apply {p.suggested_priority}
                </button>
              ) : (
                "—"
              )}
            </td>
          </tr>
        ))}
      </tbody>
    </table>
  );
};

export default ProcessTable;
