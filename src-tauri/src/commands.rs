use std::path::{Path, PathBuf};
use std::sync::Arc;

use serde::Serialize;
use tauri::{AppHandle, Manager, State};

use crate::analyzer::{AnalyzerStatus, BeatAnalyzer, BeatInfo};

const AUDIO_EXTENSIONS: [&str; 6] = ["mp3", "wav", "flac", "ogg", "m4a", "aac"];

pub struct AppState {
    pub analyzer: Result<Arc<BeatAnalyzer>, String>,
}

#[derive(Serialize, Clone)]
pub struct SongInfo {
    pub name: String,
    pub path: String,
}

#[tauri::command]
pub fn analyzer_status(state: State<'_, AppState>) -> AnalyzerStatus {
    AnalyzerStatus {
        ready: state.analyzer.is_ok(),
        model_dir: state
            .analyzer
            .as_ref()
            .ok()
            .map(|a| a.model_dir().display().to_string()),
        error: state.analyzer.as_ref().err().cloned(),
    }
}

/// Grants the asset protocol access to `music_dir` and lists the audio files in it.
#[tauri::command]
pub fn list_songs(app: AppHandle, music_dir: String) -> Result<Vec<SongInfo>, String> {
    let dir = Path::new(&music_dir);
    if !dir.is_dir() {
        return Err(format!("not a directory: {music_dir}"));
    }

    // The webview may only read audio from the folder the user picked.
    app.asset_protocol_scope()
        .allow_directory(dir, true)
        .map_err(|e| format!("cannot grant asset access to {music_dir}: {e}"))?;

    let mut songs = Vec::new();
    let entries = std::fs::read_dir(dir).map_err(|e| format!("{music_dir}: {e}"))?;
    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or_default()
            .to_lowercase();
        if !AUDIO_EXTENSIONS.contains(&ext.as_str()) {
            continue;
        }
        let name = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or_default()
            .to_string();
        songs.push(SongInfo {
            name,
            path: path.to_string_lossy().into_owned(),
        });
    }
    songs.sort_by_key(|s| s.name.to_lowercase());
    Ok(songs)
}

/// Runs beat tracking on a blocking thread pool: inference takes seconds.
#[tauri::command]
pub async fn analyze_beat(path: String, state: State<'_, AppState>) -> Result<BeatInfo, String> {
    let analyzer = Arc::clone(state.analyzer.as_ref().map_err(|e| e.clone())?);
    let path = PathBuf::from(path);
    tauri::async_runtime::spawn_blocking(move || analyzer.analyze(&path))
        .await
        .map_err(|e| format!("beat analysis task failed: {e}"))?
}
