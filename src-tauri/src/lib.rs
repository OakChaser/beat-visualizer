mod analyzer;
mod commands;

use analyzer::BeatAnalyzer;
use commands::AppState;
use std::sync::Arc;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // A missing model is reported in the UI rather than being fatal.
    let analyzer = match BeatAnalyzer::load() {
        Ok(analyzer) => Ok(Arc::new(analyzer)),
        Err(error) => {
            eprintln!("beat-visualizer: {error}");
            Err(error)
        }
    };

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .manage(AppState { analyzer })
        .invoke_handler(tauri::generate_handler![
            commands::analyzer_status,
            commands::list_songs,
            commands::analyze_beat,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
