<p align="center">
  <img src="gv-logo.png" alt="GV Pixara" width="120" />
</p>

# GV Pixara

**GV Pixara** is a minimal **Windows** desktop app for batch image conversion. Everything runs on your PC: drag and drop files, folders, or ZIP archives, pick formats and presets, and convert locally. No cloud upload.

**License:** [MIT](LICENSE) | **Privacy:** [docs/PRIVACY.md](docs/PRIVACY.md)

---

## Download

Get the latest build from **[GitHub Releases](https://github.com/uhaop/pixara/releases/latest)**.

| Option | File on Releases | What to do |
|--------|------------------|------------|
| **Portable (recommended)** | `GVPixara-portable-win64.zip` | Unzip, open the `GVPixara` folder, run `gv-pixara.exe` |
| **Installer** | `GV Pixara_*_x64_en-US.msi` or `GV Pixara_*_x64-setup.exe` | Run the installer, then start **GV Pixara** from the Start Menu |

> Keep `gv-pixara.exe` inside the `GVPixara` folder for the portable ZIP. Do not move the exe out alone.

**Requirements:** Windows 10 or 11 (64-bit). [WebView2](https://developer.microsoft.com/en-us/microsoft-edge/webview2/) is required (usually already installed). Node.js and Rust are **not** needed to run the downloaded app.

---

## How to use

### 1. Add images

- **Drag and drop** images, a folder, or a `.zip` onto the window, or  
- Use **Images**, **Folder**, or **ZIP** to browse.

The queue shows each file, source format, and target format. Use **List** or **Grid** view (icons in the queue toolbar).

### 2. Choose conversion options (right panel â†’ **Convert**)

| Control | Purpose |
|---------|---------|
| **From** | Filter what gets ingested (e.g. only PNG) |
| **To** | Output format (WebP, JPEG, PNG, AVIF, etc.) |
| **Preset** | **Web**, **High**, or **Smallest** (quality for JPEG / WebP / AVIF when supported) |
| **Output** | **Same folder** as sources, or **Custom** directory |

Click **Convert all** or select rows and use **Convert selected**. **Estimate** previews approximate output size before a large batch.

### 3. Defaults and privacy (**Settings** tab or gear icon in the header)

Open **Settings** to change options that are **saved for next time** (stored in `%APPDATA%/com.gv.gv-pixara/gv-pixara/config.json`):

- Output naming, overwrite behavior, max width/height  
- Skip same format, PNG optimization, slow drive mode  
- Re-zip outputs when the queue came from a ZIP  

Converted files have **GPS and camera EXIF removed** by default. See [docs/PRIVACY.md](docs/PRIVACY.md). Originals on disk are not modified unless you overwrite outputs.

### 4. After conversion

- **Open output folder** opens the folder where files were written  
- **Retry failed** if some items errored  
- **Clear** empties the queue and cleans temp extract folders under `%TEMP%/gv-pixara/`

---

## Supported formats (public build)

| | Formats |
|--|---------|
| **Input / output** | PNG, JPEG, WebP, AVIF, GIF, BMP, TIFF |
| **HEIC / HEIF (iPhone)** | **Not included** in the default public download (smaller build, no bundled codec DLLs) |

### iPhone / HEIC photos

1. Convert on the device to JPEG, or use another tool first, **or**  
2. In Pixara, pick a non-HEIC **To** format (e.g. WebP or JPEG) if your file is already readable as a raster format.  
3. **Optional:** run [setup-windows-heic.ps1](setup-windows-heic.ps1) once to install Microsoft HEIF/HEVC extensions (prepares Windows for future HEIC support in the app).

Store links: [HEIF Image Extensions](https://apps.microsoft.com/store/detail/heif-image-extensions/9pm4mvwc71mp) Â· [HEVC Video Extensions](https://apps.microsoft.com/store/detail/hevc-video-extensions/9nmzlz57r3t7)

---

## Tips

- **JPEG â†’ JPEG** always loses quality; keep a PNG/WebP master if you will edit again.  
- **Transparency** on JPEG (and HEIC when available) is flattened using the background color in **Settings**.  
- **Animated GIF/WebP:** only the first frame is converted.  
- **ZIP batches** need free disk space under `%TEMP%/gv-pixara/` during conversion; outputs are written beside the archive or your chosen output folder.  
- **Skip same format** (Settings): matching files are left unchanged on disk, including all original metadata.

---

## Build from source (developers)

```powershell
git clone https://github.com/uhaop/pixara.git
cd pixara
npm install
npm run tauri dev
```

Public clone uses **no** default HEIC feature â€” no vcpkg required for dev.

**Release-style build** (matches GitHub portable):

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\scripts\build-public.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File .\scripts\pre-ship.ps1
```

Output: `dist-public/GVPixara/gv-pixara.exe` and `dist-public/GVPixara-portable-win64.zip`.

Maintainers: see [PUBLISHING.md](PUBLISHING.md).

---

## Links

- **Latest release:** https://github.com/uhaop/pixara/releases/latest  
- **Source:** https://github.com/uhaop/pixara  
- **Issues:** https://github.com/uhaop/pixara/issues  
