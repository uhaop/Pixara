# Pre-ship: remove stale installers, smoke-test portables, zip public release.

$ErrorActionPreference = "Stop"
$Root = Split-Path -Parent $MyInvocation.MyCommand.Path
$ProjectRoot = Join-Path $Root ".."

function Remove-StaleInstallers {
    param([string]$InstallerDir)
    if (-not (Test-Path $InstallerDir)) {
        return
    }
    Get-ChildItem -Path $InstallerDir -File | Where-Object {
        $_.Name -like "GV Image_*"
    } | ForEach-Object {
        Write-Host "Removing stale installer: $($_.FullName)"
        Remove-Item -LiteralPath $_.FullName -Force
    }
}

function Test-PixaraPortable {
    param(
        [string]$Label,
        [string]$PortableDir
    )
    $exe = Join-Path $PortableDir "gv-pixara.exe"
    if (-not (Test-Path $exe)) {
        throw "${Label}: missing $exe"
    }

    Write-Host "Smoke test ($Label): starting $exe ..."
    $proc = Start-Process -FilePath $exe -WorkingDirectory $PortableDir -PassThru
    try {
        $deadline = (Get-Date).AddSeconds(15)
        $title = ""
        while ((Get-Date) -lt $deadline) {
            if ($proc.HasExited) {
                throw "${Label}: process exited early with code $($proc.ExitCode)"
            }
            $proc.Refresh()
            if ($proc.MainWindowTitle) {
                $title = $proc.MainWindowTitle
                break
            }
            Start-Sleep -Milliseconds 500
        }
        if (-not $title) {
            throw "${Label}: no main window title within 15s (app may have failed to open)"
        }
        if ($title -notmatch "Pixara") {
            throw "${Label}: unexpected window title '$title' (expected GV Pixara)"
        }
        Write-Host "Smoke test ($Label): OK - window title '$title'"
    } finally {
        if (-not $proc.HasExited) {
            Stop-Process -Id $proc.Id -Force -ErrorAction SilentlyContinue
        }
        Start-Sleep -Milliseconds 300
    }
}

Remove-StaleInstallers (Join-Path $ProjectRoot "dist-portable\installers")
Remove-StaleInstallers (Join-Path $ProjectRoot "dist-public\installers")

$internalDir = Join-Path $ProjectRoot "dist-portable\GVPixara"
$publicDir = Join-Path $ProjectRoot "dist-public\GVPixara"

Test-PixaraPortable -Label "internal" -PortableDir $internalDir
Test-PixaraPortable -Label "public" -PortableDir $publicDir

& (Join-Path $Root "verify-portable.ps1") -PortableDir $internalDir
& (Join-Path $Root "verify-portable-public.ps1") -PortableDir $publicDir
& (Join-Path $ProjectRoot "tools\verify-public-folder.ps1")

$publicZip = Join-Path $ProjectRoot "dist-public\GVPixara-portable-win64.zip"
if (Test-Path $publicZip) {
    Remove-Item -LiteralPath $publicZip -Force
}
Write-Host "Creating $publicZip ..."
# Zip includes GVPixara/ folder (matches public/README release layout).
Compress-Archive -Path $publicDir -DestinationPath $publicZip -CompressionLevel Optimal
$zipBytes = (Get-Item $publicZip).Length
Write-Host "Public release zip: $publicZip"
Write-Host "Zip size: $zipBytes bytes"

Write-Host ""
Write-Host "Pre-ship checks complete."
