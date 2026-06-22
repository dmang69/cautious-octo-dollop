//! Simulates bulk file encryption against the in-memory VFS.
//! Without per-file handles, only explicitly authorized paths are writable.

use anyhow::Result;
use intentos_kernel::policy::{Intent, TrustAnchor};
use intentos_shell::Session;

fn main() -> Result<()> {
    let mut session = Session::new();

    for i in 0..5 {
        session
            .vfs
            .seed(&format!("/doc{i}.txt"), &format!("sensitive content {i}"));
    }

    println!("=== Ransomware immunity demo (in-memory VFS) ===\n");
    println!("VFS before attack:");
    for path in session.vfs.list() {
        println!("  {path}");
    }

    let targets: Vec<String> = session
        .vfs
        .list()
        .into_iter()
        .filter(|p| p.starts_with("/doc"))
        .collect();

    let mut encrypted = 0u32;
    let mut denied = 0u32;

    // Attacker has ONE write handle scoped to /doc0.txt only
    let partial_handle = mint_handle(&mut session, "/doc0.txt", "vfs:write")?;

    println!("\nAttacker attempts bulk encryption with single stolen handle...");
    for target in &targets {
        session.sequence += 1;
        match session.kernel.invoke(partial_handle, session.sequence, 0) {
            Ok(_) => match session.vfs.write_gated(
                target,
                "ENCRYPTED",
                partial_handle,
                &session.kernel,
            ) {
                Ok(()) => {
                    println!("  ENCRYPTED {target}");
                    encrypted += 1;
                }
                Err(e) => {
                    println!("  DENIED {target}: {e}");
                    denied += 1;
                }
            },
            Err(e) => {
                println!("  DENIED {target}: {e}");
                denied += 1;
            }
        }
    }

    println!("\n=== Results ===");
    println!("  files encrypted: {encrypted}");
    println!("  access denied:   {denied}");
    println!(
        "  bulk blocked:    {}/{} targets denied",
        denied,
        targets.len()
    );

    if encrypted <= 1 {
        println!("\nPASS: structural denial — attacker could not encrypt the fleet.");
    } else {
        println!("\nFAIL: unexpected bulk encryption succeeded.");
        std::process::exit(1);
    }

    Ok(())
}

fn mint_handle(session: &mut Session, resource: &str, action: &str) -> Result<u64> {
    let intent = Intent {
        action: action.into(),
        resource: resource.into(),
        anchor: TrustAnchor::UiEvent,
    };
    let token = session.kernel.mint_for_intent(&session.subject, &intent)?;
    let handle = session.kernel.register_token(&token)?;
    Ok(handle.raw)
}