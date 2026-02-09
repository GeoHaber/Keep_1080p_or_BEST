# Keep_1080p_or_BEST — Visual AI Duplicate Video Finder

Scans video libraries, detects duplicate files via **perceptual hashing**, and helps you keep only the **best quality** version of each video.

## Features

- **Two-stage duplicate detection** — fast visual fingerprints (pHash) then optional temporal rhythm scan
- **O(n log n) comparison** — bucket-based by duration instead of brute-force O(n²)
- **NumPy-accelerated pHash** — 10–50× faster than pure Python DCT
- **Side-by-side VLC comparison** — color-coded quality info for easy decisions
- **Zoom/magnify** — press `Z` for pixel-level inspection
- **Background fingerprinting** — progress bar with ETA during long NAS scans
- **SQLite cache** — fingerprints cached with schema versioning for instant re-scans
- **Power management** — prevents Windows sleep during long scans
- **Codec ranking** — automatically determines best quality version

## Quick Start

```bash
python KeepBest_VisualAI.py
```

Launches a **Tkinter GUI** → select a folder → scan for duplicates → review in VLC comparison view.

## Requirements

- **FFmpeg / FFprobe** (system) — video analysis
- **VLC** (system) — side-by-side playback
- `numpy` — accelerated hashing
- `Pillow` (optional) — zoom comparison window
- `tkinter`, `sqlite3` (stdlib)

## License

MIT
