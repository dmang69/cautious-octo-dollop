using System;
using System.ServiceProcess;
using System.Diagnostics;
using System.IO;

namespace AIContextManager
{
    public class AIContextService : ServiceBase
    {
        private Process? _process;
        private const string BinaryName = "ai-runtime.exe";
        private const string ConfigPath = @"C:\ProgramData\AIOS\config.toml";

        public AIContextService()
        {
            ServiceName = "AIContextManager";
            CanStop = true;
            CanPauseAndContinue = false;
            AutoLog = true;
        }

        protected override void OnStart(string[] args)
        {
            EventLog.WriteEntry("AIContextManager", "Starting AI Context Manager...", EventLogEntryType.Information);

            var binaryPath = Path.Combine(AppContext.BaseDirectory, BinaryName);
            _process = new Process
            {
                StartInfo = new ProcessStartInfo
                {
                    FileName = binaryPath,
                    Arguments = $"--config \"{ConfigPath}\"",
                    UseShellExecute = false,
                    RedirectStandardOutput = true,
                    RedirectStandardError = true,
                    CreateNoWindow = true,
                }
            };
            _process.Start();
            EventLog.WriteEntry("AIContextManager", $"Started with PID {_process.Id}", EventLogEntryType.Information);
        }

        protected override void OnStop()
        {
            EventLog.WriteEntry("AIContextManager", "Stopping AI Context Manager...", EventLogEntryType.Information);
            if (_process is { HasExited: false })
            {
                _process.Kill(entireProcessTree: true);
                _process.WaitForExit(5000);
            }
        }

        public static void Main()
        {
            ServiceBase.Run(new AIContextService());
        }
    }
}
