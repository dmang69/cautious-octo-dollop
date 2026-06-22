use std::io::{self, Write};

use anyhow::Result;
use intentos_kernel::policy::{Intent, TrustAnchor};
use intentos_kernel::{Kernel, KernelError};
use intentos_utilities::{AiGateway, VirtualFs};
use serde::Serialize;

pub struct Session {
    pub kernel: Kernel,
    pub vfs: VirtualFs,
    pub subject: String,
    pub last_handle: Option<u64>,
    pub sequence: u64,
}

impl Session {
    pub fn new() -> Self {
        Self {
            kernel: Kernel::new(),
            vfs: VirtualFs::new(),
            subject: "intentos-session".into(),
            last_handle: None,
            sequence: 0,
        }
    }

    pub fn run_repl(&mut self) -> Result<()> {
        println!("IntentOS shell — event-scoped capability reference runtime");
        println!("Type `help` for commands.");
        let stdin = io::stdin();
        loop {
            print!("intentos> ");
            io::stdout().flush()?;
            let mut line = String::new();
            if stdin.read_line(&mut line)? == 0 {
                break;
            }
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            match self.dispatch(line) {
                Ok(Some(out)) => println!("{out}"),
                Ok(None) => {}
                Err(e) => eprintln!("error: {e:#}"),
            }
        }
        Ok(())
    }

    pub fn dispatch(&mut self, line: &str) -> Result<Option<String>> {
        let mut parts = line.split_whitespace();
        let cmd = parts.next().unwrap_or("");
        match cmd {
            "help" => {
                print_help();
                Ok(None)
            }
            "exit" | "quit" => std::process::exit(0),
            "status" => Ok(Some(self.status_json()?)),
            "flow" => self.cmd_flow(parts.collect()),
            "ls" => {
                let paths = self.vfs.list();
                Ok(Some(paths.join("\n")))
            }
            "cat" => {
                let path = parts.next().ok_or_else(|| anyhow::anyhow!("usage: cat <path>"))?;
                let handle = self.require_handle()?;
                self.sequence += 1;
                self.kernel.invoke(handle, self.sequence, 0)?;
                let content = self.vfs.read_gated(path, handle, &self.kernel)?;
                Ok(Some(content))
            }
            "write" => {
                let path = parts.next().ok_or_else(|| anyhow::anyhow!("usage: write <path> <text...>"))?;
                let text: String = parts.collect::<Vec<_>>().join(" ");
                let handle = self.require_handle()?;
                self.sequence += 1;
                self.kernel.invoke(handle, self.sequence, 0)?;
                self.vfs.write_gated(path, &text, handle, &self.kernel)?;
                Ok(Some(format!("wrote {path}")))
            }
            "ai" => {
                let sub = parts.next().unwrap_or("");
                if sub != "infer" {
                    anyhow::bail!("usage: ai infer <prompt...>");
                }
                let prompt: String = parts.collect::<Vec<_>>().join(" ");
                let handle = self.require_handle()?;
                self.sequence += 1;
                self.kernel.invoke(handle, self.sequence, 0)?;
                let resp = AiGateway::infer_gated(&prompt, handle, &self.kernel)?;
                Ok(Some(resp.output))
            }
            "lease" => self.cmd_lease(parts.collect()),
            other => Ok(Some(format!("unknown command: {other}"))),
        }
    }

    fn cmd_flow(&mut self, args: Vec<&str>) -> Result<Option<String>> {
        if args.len() < 2 {
            anyhow::bail!("usage: flow <action> <resource> [anchor]");
        }
        let anchor = match args.get(2).copied().unwrap_or("ui") {
            "ui" => TrustAnchor::UiEvent,
            "none" => TrustAnchor::None,
            "bio" => TrustAnchor::Biometric,
            "hw" => TrustAnchor::Hardware,
            other => anyhow::bail!("unknown anchor: {other}"),
        };
        let intent = Intent {
            action: args[0].to_string(),
            resource: args[1].to_string(),
            anchor,
        };
        let token = self.kernel.mint_for_intent(&self.subject, &intent)?;
        let handle = self.kernel.register_token(&token)?;
        self.last_handle = Some(handle.raw);
        self.sequence = 0;
        Ok(Some(format!(
            "flow ok handle=0x{:016X} jti={} exp={}",
            handle.raw, token.payload.jti, token.payload.exp
        )))
    }

    fn cmd_lease(&mut self, args: Vec<&str>) -> Result<Option<String>> {
        match args.first().copied().unwrap_or("list") {
            "list" => {
                let leases = self.kernel.leases.list();
                Ok(Some(serde_json::to_string_pretty(leases)?))
            }
            "tick" => {
                self.kernel.tick_leases();
                Ok(Some("lease tick complete".into()))
            }
            "grant" => {
                let resource = args.get(1).unwrap_or(&"lease:background");
                let intent = Intent {
                    action: "lease:grant".into(),
                    resource: (*resource).to_string(),
                    anchor: TrustAnchor::UiEvent,
                };
                let token = self.kernel.mint_for_intent(&self.subject, &intent)?;
                self.kernel.register_token(&token)?;
                Ok(Some(format!("lease granted jti={}", token.payload.jti)))
            }
            "renew" => {
                let jti = args.get(1).ok_or_else(|| anyhow::anyhow!("usage: lease renew <jti>"))?;
                let new_exp = Kernel::now_ms() + 300_000;
                self.kernel
                    .leases
                    .renew(jti, new_exp)
                    .map_err(|e| anyhow::anyhow!("{e}"))?;
                Ok(Some(format!("renewed {jti} exp={new_exp}")))
            }
            other => Ok(Some(format!("unknown lease subcommand: {other}"))),
        }
    }

    fn require_handle(&self) -> Result<u64, KernelError> {
        self.last_handle
            .ok_or_else(|| KernelError::Denied("no handle — run `flow` first".into()))
    }

    fn status_json(&self) -> Result<String> {
        #[derive(Serialize)]
        struct Status<'a> {
            subject: &'a str,
            handle: Option<String>,
            stats: intentos_kernel::KernelStats,
            vfs_entries: usize,
        }
        let stats = self.kernel.stats();
        let status = Status {
            subject: &self.subject,
            handle: self.last_handle.map(|h| format!("0x{h:016X}")),
            stats,
            vfs_entries: self.vfs.list().len(),
        };
        Ok(serde_json::to_string_pretty(&status)?)
    }
}

fn print_help() {
    println!(
        r#"Commands:
  help                         Show this help
  status                       JSON session status
  flow <action> <resource>     Mint token + register handle
  ls                           List in-memory VFS paths
  cat <path>                   Read file (requires handle)
  write <path> <text>          Write file (requires handle)
  ai infer <prompt>            Stub AI inference (requires handle)
  lease list|tick|grant|renew  Lease tracking commands
  exit                         Quit"#
    );
}