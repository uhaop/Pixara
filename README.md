<h1 align="center">Pixara</h1>

<p align="center">
  <strong>Batch image conversion for Windows — local, private, and fast.</strong>
</p>

<p align="center"><em>by Grasp Visual</em></p>

<p align="center">
  <a href="https://github.com/uhaop/pixara/releases/latest"><img src="https://img.shields.io/github/v/release/uhaop/pixara?label=Release" alt="Latest release" /></a>
  <a href="https://github.com/uhaop/pixara/blob/main/LICENSE"><img src="https://img.shields.io/badge/License-MIT-blue.svg" alt="MIT License" /></a>
  <a href="https://github.com/uhaop/pixara/actions"><img src="https://img.shields.io/github/actions/workflow/status/uhaop/pixara/ci-public.yml?branch=main&label=CI" alt="CI" /></a>
</p>

<p align="center">
  <a href="#download">Download</a> ·
  <a href="#quick-start">Quick start</a> ·
  <a href="#features">Features</a> ·
  <a href="#supported-formats">Formats</a> ·
  <a href="#privacy">Privacy</a> ·
  <a href="#build-from-source">Developers</a>
</p>

---

**Pixara** is a lightweight Windows desktop app for converting images in bulk. Drop files, folders, or ZIP archives onto the window, choose output formats and quality presets, and convert entirely on your machine. Nothing is uploaded to the cloud.

| | |
|---|---|
| **Platform** | Windows 10 or 11 (64-bit) |
| **License** | [MIT](LICENSE) |
| **Privacy policy** | [docs/PRIVACY.md](docs/PRIVACY.md) |
| **Third-party notices** | [THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md) |

---

## Download

Install the latest build from **[GitHub Releases](https://github.com/uhaop/pixara/releases/latest)**.

| Option | Release asset | How to run |
|--------|---------------|------------|
| **Portable** *(recommended)* | `Pixara-portable-win64.zip` | Unzip, open the `Pixara` folder, run `pixara.exe` |
| **MSI installer** | `Pixara_*_x64_en-US.msi` | Run the installer, then launch **Pixara** from the Start Menu |
| **Setup (NSIS)** | `Pixara_*_x64-setup.exe` | Alternative installer if you prefer the setup wizard |

> **Portable tip:** Keep `pixara.exe` inside the `Pixara` folder. Do not move the executable out on its own.

**Runtime requirements**

- Windows 10 or 11, 64-bit
- [Microsoft Edge WebView2](https://developer.microsoft.com/microsoft-edge/webview2/) (usually pre-installed on Windows 11)
- No Node.js, Rust, or developer tools required for the downloaded app

---

## Quick start

1. **Add images** — Drag and drop images, a folder, or a `.zip` onto the window, or use **Images**, **Folder**, or **ZIP** to browse.
2. **Set options** — In the right panel (**Convert** tab), choose **From** / **To** formats, a **Preset** (Web, High, or Smallest), and **Output** location (same folder or custom).
3. **Convert** — Click **Convert all**, or select rows and use **Convert selected**. Use **Estimate** to preview approximate output sizes before a large batch.
4. **Finish** — Use **Open output folder** when done. **Retry failed** handles errors; **Clear** empties the queue.

The queue shows each file with source and target formats. Switch between **List** and **Grid** view, sort and filter rows, and rename outputs from the queue toolbar.

Use the right sidebar **Convert** tab for format and output options; **Settings** (tab or header gear icon) for defaults that persist between sessions at  
`%APPDATA%/com.uhaop.pixara/pixara/config.json`.

---

## Features

- **Batch conversion** — Process many files at once with per-file progress and before/after sizes
- **Flexible input** — Single files, whole folders (with subfolder structure preserved), or ZIP archives
- **Format control** — Filter ingestion with **From** and set **To** (WebP, JPEG, PNG, AVIF, GIF, BMP, TIFF)
- **Quality presets** — **Web**, **High**, and **Smallest** for JPEG, WebP, and AVIF when supported
- **Queue tools** — List or grid view, sort and filter, select rows, inline and batch rename
- **Privacy-first exports** — GPS, camera, and other EXIF identity tags removed from converted outputs by default ([details](#privacy))
- **Output flexibility** — Same folder as sources or a custom directory; optional `_converted` suffix or extension replace
- **Resize & optimize** — Optional max width/height, PNG optimization, slow-drive mode, and re-zip for ZIP batches
- **Skip unchanged** — **Skip same format** leaves matching files untouched on disk (including original metadata)

Original files on disk are not modified unless you choose to overwrite existing outputs.

---

## Supported formats

### Public release (default download)

| Role | Formats |
|------|---------|
| **Input / output** | PNG, JPEG, WebP, AVIF, GIF, BMP, TIFF |
| **HEIC / HEIF (iPhone)** | Not bundled in the default public build (smaller download, no codec DLLs) |

### iPhone / HEIC photos

The standard GitHub release does not include HEIC decode libraries. You can:

1. **Convert on the device** — Export photos as JPEG from the Photos app, or use another tool first.
2. **Prepare Windows (optional)** — Run [setup-windows-heic.ps1](setup-windows-heic.ps1) once to install Microsoft HEIF/HEVC extensions via winget (useful if native HEIC support is added later).

Microsoft Store links: [HEIF Image Extensions](https://apps.microsoft.com/detail/9pm4mvwc71mp) · [HEVC Video Extensions](https://apps.microsoft.com/detail/9nmzlz57r3t7)

---

## Privacy

Converted files have **GPS, location, and camera EXIF removed** by default. Orientation is baked into pixels so outputs do not depend on an Orientation tag. See [docs/PRIVACY.md](docs/PRIVACY.md) for the full policy.

| Setting | Behavior |
|---------|----------|
| **Default conversion** | New outputs are stripped of GPS and EXIF identity tags |
| **Skip same format** | Matching files are **not** re-encoded; originals (and all metadata) stay unchanged |
| **Keep color profile (ICC)** | Optional for PNG when color accuracy matters |

To strip metadata from files that already match the target format, turn **Skip same format** off so they are re-encoded.

---

## Tips & limitations

- **JPEG → JPEG** always loses quality (generation loss). Keep a PNG or WebP master if you plan to edit again.
- **Transparency** — JPEG targets flatten alpha using the background color in **Settings**.
- **Animated GIF / WebP** — Only the first frame is converted.
- **ZIP batches** — Archives extract under `%TEMP%/pixara/`; ensure enough free disk space. Outputs go beside the archive or your chosen output folder.
- **Overwrite** — Configure overwrite behavior in **Settings** to avoid accidental replacement.

---

## Build from source

For contributors and developers who want to run or package the app locally.

### Prerequisites

- [Node.js](https://nodejs.org/) 18+
- [Rust](https://rustup.rs/) (stable)
- Visual Studio 2022 Build Tools with the **Desktop development with C++** workload

The public repository is built **without** the default HEIC feature — no vcpkg or codec DLLs required for development.

### Development

```powershell
git clone https://github.com/uhaop/pixara.git
cd pixara
npm install
npm run tauri dev
```

### Release-style build (matches GitHub portable)

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\scripts\build-public.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File .\scripts\pre-ship.ps1
```

Artifacts:

- `dist-public/Pixara/pixara.exe`
- `dist-public/Pixara-portable-win64.zip`

Maintainers: see [PUBLISHING.md](PUBLISHING.md).

---

## Links

| | URL |
|---|-----|
| **Latest release** | https://github.com/uhaop/pixara/releases/latest |
| **Source code** | https://github.com/uhaop/pixara |
| **Report an issue** | https://github.com/uhaop/pixara/issues |
| **Privacy** | [docs/PRIVACY.md](docs/PRIVACY.md) |
| **Third-party notices** | [THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md) |

---

<p align="center">
  <sub>Pixara — convert images on your PC, not in the cloud.</sub>
</p>
