export interface SystemSnapshot {
  cpuPercent: number;
  memPercent: number;
  diskIoMbps: number;
  netRxMbps: number;
  netTxMbps: number;
  queueDepth: number;
  uptimeSecs: number;
  processCount: number;
  ipcQueued: number;
}
export interface ConnectionStatus { connected: boolean; endpoint: string; }
export interface SchedulerPolicy { timeSlicesMs: number[]; }
export interface LookupResult {
  target: string;
  targetType: string;
  verdict: string;
  threatLevel: string;
  reputationScore: number;
  timestamp: number;
  descramblerValidated: boolean;
}