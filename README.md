# MediaProbe

A standalone, installable desktop application built in Rust with [GPUI](https://gpui.rs) for inspecting multimedia files without requiring or wrapping `ffprobe`.

## Features

- **Pure Standalone Media Parser**: Demuxes and inspects files natively in Rust (`mp4`, `matroska`, `symphonia`, `imagesize`) with zero external binary dependencies.
- **Modern Dark Web-Style UI**: Built with Zed's GPU-accelerated **GPUI** framework featuring crisp white typography and human-readable media summaries.
- **Multiple Views**:
  - **Overview**: Plain-English media summary, format/size/duration/bitrate tiles, and track overview.
  - **Streams**: Detailed per-track specs (resolution, aspect ratio, frame rate, sample rate, channel layout, bit depth).
  - **Metadata**: Embedded container tags (title, artist, album, encoder, creation date, etc.).
  - **Raw JSON**: Structured JSON view with one-click **Copy JSON** to clipboard.
- **Desktop Integration**:
  - Drag-and-drop media files into the window, click **Open File**, or pass a path via CLI (`mediaprobe <file>`).
  - Windows installer & PowerShell setup script include the **"Inspect with MediaProbe"** right-click Explorer context menu option.

## Supported Formats

- **Video Containers**: MP4, M4V, MOV, 3GP, MKV, WebM
- **Audio Formats**: MP3, FLAC, WAV, OGG, Opus, AAC, AIFF, CAF
- **Image Formats**: PNG, JPEG, WebP, GIF, BMP, TIFF, AVIF, ICO

## Installation

### Prebuilt Installers (via GitHub Actions)
- **Windows**: Run `MediaProbe-Setup-x86_64.exe` (includes optional **"Inspect with MediaProbe"** Explorer context menu integration).
- **macOS**: Open `MediaProbe-macOS-Universal.dmg` and drag `MediaProbe.app` into `Applications`.
- **Linux**: Install `mediaprobe_<version>_amd64.deb` (`sudo apt install ./mediaprobe_*.deb`) or extract `mediaprobe-linux-x86_64.tar.gz`.

### Windows Local Install + Context Menu (PowerShell)
```powershell
cargo build --release
powershell -ExecutionPolicy Bypass -File packaging\windows\install.ps1
```
To uninstall and remove the Windows context menu entry:
```powershell
powershell -ExecutionPolicy Bypass -File packaging\windows\uninstall.ps1
```

## Building from Source

```bash
cargo test
cargo build --release
```
