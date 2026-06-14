# Tauri Commands Reference

All commands are invoked from the React frontend via `@tauri-apps/api/tauri`'s `invoke()`.

## Commands

### `run_command`
Execute a shell command and return combined stdout/stderr.

| Param | Type | Description |
|-------|------|-------------|
| `command` | `string` | Shell command to execute |

Returns: `string`

---

### `list_processes`
Return the current process list with optional AI priority suggestions.

Returns: `ProcessInfo[]`

---

### `set_priority`
Adjust a process's scheduling priority.

| Param | Type | Description |
|-------|------|-------------|
| `pid` | `number` | Process ID |
| `priority` | `number` | New priority (−20 to 19 on Unix) |

Returns: `void`

---

### `get_telemetry_snapshot`
Return a single telemetry snapshot from the AI Runtime.

Returns: `TelemetrySnapshot`

---

### `get_scheduler_recommendations`
Return AI scheduling recommendations for a given snapshot.

| Param | Type | Description |
|-------|------|-------------|
| `snapshot` | `TelemetrySnapshot` | System telemetry input |

Returns: `PriorityRecommendation[]`

---

### `interpret_command`
Send a raw command string to the NLP interpreter.

| Param | Type | Description |
|-------|------|-------------|
| `rawCommand` | `string` | User input |

Returns: `{ intent: string; structured_json: string; confidence: number }`
