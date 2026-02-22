# Meeting Minder

Local macOS desktop app for meeting recording and per-speaker speaking-time percentages.

## Stack
- React + TypeScript + Vite frontend
- Tauri (Rust) backend for native mic capture and filesystem orchestration
- Local Python analyzer for speech segmentation and anonymous speaker clustering

## Current behavior
- Start/stop microphone recording
- Session persistence under app data directory (`sessions/<session_id>/`)
- Post-meeting analysis output (`analysis.json`)
- Per-speaker percentage view (% of detected speech time)
- CSV/JSON export

## Prerequisites
1. Node.js 20+
2. Python 3.10+
3. Rust toolchain (`rustup`, `cargo`, `rustc`)
4. Tauri system dependencies for macOS (Xcode Command Line Tools)

## Install
```bash
npm install
```

## Run frontend tests
```bash
npm test
python3 -m unittest discover -s tests/python
```

## Run app (dev)
```bash
npm run tauri:dev
```

## Key paths
- Frontend: `src/`
- Tauri backend: `src-tauri/src/`
- Python analyzer: `python/analyzer/`
- Tests: `tests/`

## Analyzer note
The analyzer is intentionally offline and stdlib-based (`heuristic-v2`) to keep setup simple and local-only.
