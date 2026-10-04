//! Beat tracking backed by the pure-Rust [`beat_this`] crate.
//!
//! Two ONNX models are required (see `scripts/fetch-models.sh`):
//! the log-mel front end and the beat model. They are looked up at runtime
//! instead of being linked in, so they can be swapped without a rebuild.

use std::path::{Path, PathBuf};
use std::sync::Mutex;

use beat_this::{BeatThis, RtenRuntime};
use serde::Serialize;

const MEL_MODEL: &str = "mel_spectrogram.onnx";

/// Beat models in order of preference: the full FP32 model wins when present.
const BEAT_MODELS: [&str; 2] = ["beat_this.onnx", "beat_this_small.onnx"];

/// Overrides model discovery entirely.
const MODEL_DIR_ENV: &str = "BEAT_MODEL_DIR";

#[derive(Serialize, Clone, Debug)]
pub struct BeatInfo {
    pub beats: Vec<f32>,
    pub downbeats: Vec<f32>,
    pub bpm: Option<f32>,
}

#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct AnalyzerStatus {
    pub ready: bool,
    pub model_dir: Option<String>,
    pub error: Option<String>,
}

type AnalyzeFn = Box<dyn FnMut(&Path) -> Result<BeatInfo, String> + Send>;

/// `beat_this::RtenModel` is not nameable from outside the crate, so the
/// tracker is held behind a closure instead of a concrete generic type.
pub struct BeatAnalyzer {
    model_dir: PathBuf,
    analyze: Mutex<AnalyzeFn>,
}

impl BeatAnalyzer {
    /// Loads the ONNX models. Fails when they are missing or unreadable, which
    /// the UI surfaces instead of crashing the app.
    pub fn load() -> Result<Self, String> {
        let model_dir = find_model_dir().ok_or_else(missing_models_message)?;
        let beat_model = resolve_beat_model(&model_dir).ok_or_else(|| {
            format!(
                "no beat model in {} (expected one of {})",
                model_dir.display(),
                BEAT_MODELS.join(", ")
            )
        })?;

        let mut tracker = BeatThis::new(&RtenRuntime, &model_dir.join(MEL_MODEL), &beat_model)
            .map_err(|e| format!("failed to load ONNX models: {e}"))?;

        eprintln!("beat-visualizer: using models from {}", model_dir.display());

        let analyze: AnalyzeFn = Box::new(move |path: &Path| {
            let analysis = tracker
                .analyze_file(path)
                .map_err(|e| format!("beat analysis failed for {}: {e}", path.display()))?;
            Ok(BeatInfo {
                bpm: beat_this::calculate_bpm(&analysis),
                beats: analysis.beats,
                downbeats: analysis.downbeats,
            })
        });

        Ok(Self {
            model_dir,
            analyze: Mutex::new(analyze),
        })
    }

    /// Directory the ONNX models were loaded from.
    pub fn model_dir(&self) -> &Path {
        &self.model_dir
    }

    pub fn analyze(&self, path: &Path) -> Result<BeatInfo, String> {
        let mut analyze = self
            .analyze
            .lock()
            .map_err(|_| "beat analyzer mutex poisoned".to_string())?;
        (analyze)(path)
    }
}

/// Directories that may hold the models, most specific first.
fn model_dir_candidates() -> Vec<PathBuf> {
    let mut dirs = Vec::new();

    if let Some(dir) = std::env::var_os(MODEL_DIR_ENV) {
        dirs.push(PathBuf::from(dir));
    }

    if let Ok(cwd) = std::env::current_dir() {
        dirs.push(cwd.join("models"));
    }

    // Walk up from the executable so the models are found when running
    // `src-tauri/target/{debug,release}/beat-visualizer` (or `…/deps/…` under
    // `cargo test`) as well as from inside a bundle, where the executable sits
    // next to the resource directory.
    if let Ok(exe) = std::env::current_exe() {
        let mut dir = exe.parent().map(Path::to_path_buf);
        for _ in 0..6 {
            let Some(current) = dir.clone() else { break };
            dirs.push(current.join("models"));
            dirs.push(current.join("resources").join("models"));
            dir = current.parent().map(Path::to_path_buf);
        }
    }

    dirs
}

fn find_model_dir() -> Option<PathBuf> {
    model_dir_candidates()
        .into_iter()
        .find(|dir| dir.join(MEL_MODEL).is_file() && resolve_beat_model(dir).is_some())
}

fn resolve_beat_model(dir: &Path) -> Option<PathBuf> {
    BEAT_MODELS
        .iter()
        .map(|name| dir.join(name))
        .find(|path| path.is_file())
}

fn missing_models_message() -> String {
    let searched = model_dir_candidates()
        .iter()
        .map(|d| d.display().to_string())
        .collect::<Vec<_>>()
        .join(", ");
    format!(
        "Beat This! models not found. Run `scripts/fetch-models.sh`, or point \
         {MODEL_DIR_ENV} at a directory containing {MEL_MODEL} and a beat model. \
         Searched: {searched}"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn candidates_include_models_dirs() {
        let candidates = model_dir_candidates();
        assert!(
            candidates.iter().any(|d| d.ends_with("models")),
            "expected at least one `models` directory candidate"
        );
    }

    #[test]
    fn missing_models_message_mentions_fetch_script() {
        assert!(missing_models_message().contains("fetch-models.sh"));
    }

    #[test]
    fn resolve_beat_model_prefers_full_model() {
        let dir = std::env::temp_dir().join(format!("bv-model-test-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join(BEAT_MODELS[0]), b"").unwrap();
        std::fs::write(dir.join(BEAT_MODELS[1]), b"").unwrap();
        assert_eq!(resolve_beat_model(&dir), Some(dir.join(BEAT_MODELS[0])));
        std::fs::remove_file(dir.join(BEAT_MODELS[0])).unwrap();
        assert_eq!(resolve_beat_model(&dir), Some(dir.join(BEAT_MODELS[1])));
        std::fs::remove_dir_all(&dir).unwrap();
    }

    /// Runs the real pipeline. Opt-in because it needs `models/` and a local file.
    #[test]
    #[ignore = "set BEAT_TEST_AUDIO and fetch the models first"]
    fn analyzes_a_real_file() {
        let audio = std::env::var("BEAT_TEST_AUDIO").expect("set BEAT_TEST_AUDIO");
        let analyzer = BeatAnalyzer::load().expect("load models");
        eprintln!("model dir: {}", analyzer.model_dir().display());
        let info = analyzer.analyze(Path::new(&audio)).expect("analyze");
        eprintln!(
            "bpm={:?} beats={} downbeats={}",
            info.bpm,
            info.beats.len(),
            info.downbeats.len()
        );
        assert!(info.beats.len() > 10);
        assert!(!info.downbeats.is_empty());
    }
}
