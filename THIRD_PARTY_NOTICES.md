# Third-party notices (Pixara — public build)

The **public** Windows download (`pixara.exe` portable or MSI) is built **without**
LGPL/GPL HEIC codec libraries. This file covers components that still apply to that build.

## Application stack

Pixara is a Tauri 2 desktop app. Major components include:

- **Tauri** — https://github.com/tauri-apps/tauri (Apache-2.0 / MIT)
- **Rust** ecosystem crates — see `src-tauri/Cargo.lock` after `cargo build`
- **React / Vite / TypeScript** — see `package-lock.json`
- **image** (Rust) — https://github.com/image-rs/image
- **oxipng** — https://github.com/oxipng/oxipng (optional PNG optimization)
- **WebView2** — Microsoft runtime, installed separately on Windows

Source code for this project is available under the MIT License — see [LICENSE](LICENSE).

## HEIC / iPhone photos

The default public build does **not** ship `heif.dll`, `libde265.dll`, or other codec DLLs.

To prepare Windows for future native HEIC support, run [setup-windows-heic.ps1](setup-windows-heic.ps1)
(installs Microsoft HEIF Image Extensions and HEVC Video Extensions via winget).

## Optional components (not in default download)

If you build with HEIC support enabled (`--features heic` and vcpkg), additional libraries apply
(libheif, libde265, libaom, etc.). Those builds require separate LGPL compliance and are not
part of the default GitHub release assets.

## Bundled FFmpeg (optional)

If you place `ffmpeg.exe` beside `pixara.exe`, FFmpeg’s license applies to that binary.
Use a build configuration and notice file matching your FFmpeg distribution.
