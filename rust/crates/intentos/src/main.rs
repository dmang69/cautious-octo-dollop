use anyhow::Result;
use intentos_shell::Session;

fn main() -> Result<()> {
    let mut session = Session::new();
    session.run_repl()
}