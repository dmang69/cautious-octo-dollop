mod commands;
mod state;

use tauri::Manager;

fn main() {
    tauri::Builder::default()
        .manage(state::AppState::default())
        .invoke_handler(tauri::generate_handler![
            commands::run_command,
            commands::list_processes,
            commands::set_priority,
            commands::get_telemetry_snapshot,
            commands::get_scheduler_recommendations,
            commands::interpret_command,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
