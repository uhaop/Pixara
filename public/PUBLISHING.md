# Publishing GV Pixara to GitHub (public)

Public repository: **https://github.com/uhaop/pixara**

## One-time setup (internal repo)

1. Confirm [export-manifest.json](export-manifest.json): `github.owner` = `uhaop`, `github.repo` = `pixara`.
2. Verify the release kit:

   ```powershell
   powershell -NoProfile -ExecutionPolicy Bypass -File .\tools\verify-public-folder.ps1
   ```

3. Export a clean tree (no `target/`, no `node_modules/`):

   ```powershell
   powershell -NoProfile -ExecutionPolicy Bypass -File .\tools\export-to-public.ps1 -Destination ..\pixara
   ```

4. Push to GitHub (empty repo [uhaop/pixara](https://github.com/uhaop/pixara)):

   ```powershell
   cd ..\pixara
   git init
   git add .
   git commit -m "Initial public export of GV Pixara"
   git branch -M main
   git remote add origin https://github.com/uhaop/pixara.git
   git push -u origin main
   ```

## Each release (internal repo)

1. Build public artifacts:

   ```powershell
   powershell -NoProfile -ExecutionPolicy Bypass -File .\scripts\build-public.ps1
   ```

2. Pre-ship checks (smoke test, stale installer cleanup, release ZIP):

   ```powershell
   powershell -NoProfile -ExecutionPolicy Bypass -File .\scripts\pre-ship.ps1
   ```

3. Create a [GitHub Release](https://github.com/uhaop/pixara/releases/new) (e.g. tag `v0.1.0`) and upload:

   | Asset | Source |
   |-------|--------|
   | `GVPixara-portable-win64.zip` | `dist-public/GVPixara-portable-win64.zip` |
   | `GV Pixara_*_x64_en-US.msi` | `dist-public/installers/*.msi` |
   | `GV Pixara_*_x64-setup.exe` | `dist-public/installers/*-setup.exe` |

4. Release notes: link to [README](README.md) quick start and optional [setup-windows-heic.ps1](setup-windows-heic.ps1) for iPhone users.

5. Re-export source if the release includes code changes:

   ```powershell
   powershell -NoProfile -ExecutionPolicy Bypass -File .\tools\export-to-public.ps1 -Destination ..\pixara
   cd ..\pixara
   git add -A
   git commit -m "Release v0.1.0"
   git tag v0.1.0
   git push origin main --tags
   ```

## What must never appear on public Releases

- `heif.dll`, `libde265.dll`, `libx265.dll`, `aom.dll` (LGPL / internal HEIC stack)
- Artifacts from internal `scripts/build.ps1` / `dist-portable/` (full HEIC portable)

Internal GV builds stay in the private repo using `scripts/build.ps1`.

## Export safety checks

The export script:

- Copies only paths listed in `export-manifest.json`
- Strips `default = ["heic"]` from `src-tauri/Cargo.toml`
- Removes `node_modules`, `target`, `dist`, `dist-portable`, `dist-public` from the export tree
- Does **not** copy `scripts/build.ps1`, `scripts/copy-heic-dlls.ps1`, or internal-only docs
