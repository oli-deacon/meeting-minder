# Meeting Minder

Meeting Minder is a local macOS desktop app that records meeting audio and estimates what percentage of detected speech time each speaker used.

## What it does
- Records microphone audio to a local WAV file.
- Runs local (offline) speaker segmentation and clustering.
- Shows per-speaker speaking percentages (`Speaker 1`, `Speaker 2`, ...).
- Exports analysis as CSV + JSON.

## Tech stack
- Frontend: React + TypeScript + Vite
- Desktop shell: Tauri (Rust)
- Analyzer: Python (stdlib only, heuristic diarization)

## Requirements (macOS)
Install these before running the app:

1. Xcode Command Line Tools
```bash
xcode-select --install
```

2. Homebrew (if not already installed)
- [https://brew.sh](https://brew.sh)

3. Node.js 20+ and npm
```bash
brew install node
```

4. Python 3.10+
```bash
brew install python
```

5. Rust toolchain
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source "$HOME/.cargo/env"
```

## Verify your environment
From a new terminal:

```bash
node -v
npm -v
python3 --version
rustc --version
cargo --version
```

## Run locally (development)
From the project directory:

```bash
cd /Users/olideacon/code/meeting-minder
npm install
npm run tauri:dev
```

This starts:
- Vite dev server (frontend)
- Tauri desktop app window

## First run on macOS: microphone permissions
When you start recording, macOS should prompt for microphone access.

If recording fails or is silent:
1. Open `System Settings -> Privacy & Security -> Microphone`
2. Enable access for your terminal and/or the app process launched by Tauri
3. Restart the app and try again

## How to use
1. Click `Start Recording`
2. Speak in the meeting
3. Click `Stop Recording`
4. Select the session in the left panel
5. Click `Analyze Session`
6. Review percentages and click `Export CSV/JSON` if needed

## Data location
Session artifacts are stored in:

`~/Library/Application Support/com.meetingminder.app/sessions/<session_id>/`

Each session folder contains:
- `audio.wav`
- `session.json`
- `analysis.json` (after analysis)
- `analysis.csv` (after export)

## Run tests
```bash
npm test
python3 -m unittest discover -s tests/python
```

## Troubleshooting

### `command not found: rustc` or `cargo`
Rust is not installed or shell profile is not loaded.

```bash
source "$HOME/.cargo/env"
```

### `proc macro panicked ... icon ... not RGBA`
The icons in `src-tauri/icons/` are missing or invalid. Re-sync the project files and run again.

### Analysis returns 0% speech
- Check microphone input level in macOS settings.
- Move closer to mic or increase input gain.
- Re-run `Analyze Session` (re-analysis is enabled).

### Push to GitHub fails with HTTPS credentials
Use SSH remote:

```bash
git remote set-url origin git@github.com:oli-deacon/meeting-minder.git
git push -u origin main
```

## Notes on accuracy
- Results are percentages of **detected speech time**, not total meeting wall-clock duration.
- The current analyzer is heuristic (`heuristic-v2`) and fully offline.
- It works best for clean mic audio and distinct speaker voices.

## Project structure
- `src/` React frontend
- `src-tauri/src/` Rust backend
- `python/analyzer/` analysis pipeline
- `tests/` frontend + Python tests
