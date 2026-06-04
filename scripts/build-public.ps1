# GV Pixara — public GitHub release build (no vcpkg, no HEIC DLLs, no libheif link).

$ErrorActionPreference = "Stop"
$Root = Split-Path -Parent $MyInvocation.MyCommand.Path
$ProjectRoot = Join-Path $Root ".."
Set-Location $ProjectRoot

$CargoBin = Join-Path $env:USERPROFILE ".cargo\bin"
if (Test-Path $CargoBin) {
    $env:Path = "$CargoBin;$env:Path"
}

$env:GV_PIXARA_PUBLIC = "1"

Write-Host "Building public GV Pixara (GV_PIXARA_PUBLIC=1, Cargo --no-default-features)..."
npm run tauri build -- -- --no-default-features
if ($LASTEXITCODE -ne 0) {
    throw "tauri build failed (exit $LASTEXITCODE)."
}

$ReleaseDir = Join-Path $ProjectRoot "src-tauri\target\release"
$ExePath = Join-Path $ReleaseDir "gv-pixara.exe"
if (-not (Test-Path $ExePath)) {
    throw "Build did not produce $ExePath"
}

$DistRoot = Join-Path $ProjectRoot "dist-public"
$PortableDir = Join-Path $DistRoot "GVPixara"
$InstallerDir = Join-Path $DistRoot "installers"

New-Item -ItemType Directory -Force -Path $PortableDir | Out-Null
New-Item -ItemType Directory -Force -Path $InstallerDir | Out-Null

Copy-Item -Path $ExePath -Destination $PortableDir -Force

& (Join-Path $Root "verify-portable-public.ps1") -PortableDir $PortableDir

$MsiSourceDir = Join-Path $ReleaseDir "bundle\msi"
$NsisSourceDir = Join-Path $ReleaseDir "bundle\nsis"
$CopiedInstallers = @()

foreach ($sourceDir in @($MsiSourceDir, $NsisSourceDir)) {
    if (-not (Test-Path $sourceDir)) {
        continue
    }
    Get-ChildItem -Path $sourceDir -File | Where-Object {
        $_.Extension -in ".msi", ".exe"
    } | ForEach-Object {
        $destination = Join-Path $InstallerDir $_.Name
        Copy-Item -Path $_.FullName -Destination $destination -Force
        $CopiedInstallers += $destination
    }
}

Write-Host ""
Write-Host "Public distribution outputs under $DistRoot"
Write-Host "  Portable: $PortableDir  (gv-pixara.exe only)"
foreach ($installer in $CopiedInstallers) {
    Write-Host "  Installer: $installer"
}
