# Beat Visualizer

A desktop music player that lights up on the beat. Beats and downbeats are detected
with the [Beat This!](https://github.com/CPJKU/beat_this) model, running locally
through the pure-Rust [`beat-this`](https://crates.io/crates/beat-this) crate — no
Python, no ONNX Runtime, no server.

Built with [Tauri 2](https://tauri.app), Vue 3, Pinia and GSAP.

<p align="center">
  <em>Downbeats pulse the side lights, every beat pulses the title.</em>
</p>

## Features

- Pick any local folder as your library; the choice is remembered across restarts.
- Beat and downbeat detection for WAV, MP3, FLAC, OGG, M4A and AAC.
- Beat-synced title pulse and downbeat light sweeps.
- BPM, beat count and downbeat count shown per track.
- The webview is only granted read access to the folder you actually picked.

## Requirements

- [Node.js](https://nodejs.org/) 20+ and [pnpm](https://pnpm.io/)
- [Rust](https://www.rust-lang.org/tools/install) (stable) and the
  [Tauri prerequisites](https://tauri.app/start/prerequisites/)
- `curl` for the one-off model download

## Getting started

```bash
pnpm install
scripts/fetch-models.sh      # ONNX models → models/  (~10 MB)
pnpm tauri dev
```

On first launch the app asks for a music folder. Point it at a directory of audio
files and pick a track.

To use the full-accuracy beat model instead of the bundled small one (~83 MB):

```bash
scripts/fetch-models.sh --full
```

## Building

```bash
pnpm tauri build
```

The bundle lands in `src-tauri/target/release/bundle/`.

## How the models are found

Beat tracking needs two ONNX files: `mel_spectrogram.onnx` and a beat model
(`beat_this.onnx`, or `beat_this_small.onnx` as the fallback). At startup the app
looks for them, in order, in:

1. `$BEAT_MODEL_DIR`
2. `./models` relative to the working directory
3. `models/`, then `resources/models/`, walking up from the executable — which
   covers `src-tauri/target/{debug,release}` and bundled layouts

The full model wins when both are present. `analyzer_status` reports the resolved
directory, and the UI shows the failure message if the models are missing.

The models are not baked into the bundle, so a packaged build needs them copied
next to the executable — e.g. `Beat Visualizer.app/Contents/Resources/models/`
on macOS, or `resources/models/` next to the binary elsewhere.

## Audio in your library

The app lists `.mp3`, `.wav`, `.flac`, `.ogg`, `.m4a` and `.aac` files from the
chosen folder. Sample audio is **not** included in this repository.

## Tests

```bash
cargo test --manifest-path src-tauri/Cargo.toml
```

The unit tests cover model discovery. An opt-in integration test runs the real
pipeline against a local file:

```bash
BEAT_TEST_AUDIO=/path/to/track.wav \
  cargo test --manifest-path src-tauri/Cargo.toml --lib -- --ignored --nocapture
```

## Credits

- Beat tracking: [Beat This! Accurate and Generalizable Beat Tracking](https://arxiv.org/abs/2407.21658)
  (ISMIR 2024, Johannes Kepler University Linz). Rust port: [`beat-this`](https://github.com/danigb/beat-this-rs) by
  [danigb](https://github.com/danigb), MIT licensed.
- Original implementation: [CPJKU/beat_this](https://github.com/CPJKU/beat_this).

## License

[MIT](LICENSE)