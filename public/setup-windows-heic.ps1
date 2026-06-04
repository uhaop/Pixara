# Install Microsoft HEIF/HEVC codecs for iPhone-style photos on Windows.
# Run once per machine (admin may be required for winget). Does not install GV Pixara.

$ErrorActionPreference = "Stop"

function Test-Winget {
    return [bool](Get-Command winget -ErrorAction SilentlyContinue)
}

if (-not (Test-Winget)) {
    Write-Host "winget is not available. Install manually from Microsoft Store:"
    Write-Host "  HEIF Image Extensions: https://apps.microsoft.com/store/detail/heif-image-extensions/9pm4mvwc71mp"
    Write-Host "  HEVC Video Extensions: https://apps.microsoft.com/store/detail/hevc-video-extensions/9nmzlz57r3t7"
    exit 1
}

Write-Host "Installing HEIF Image Extensions..."
winget install --id Microsoft.HEIFImageExtension -e --accept-source-agreements --accept-package-agreements

Write-Host "Installing HEVC Video Extensions (required for most iPhone HEIC)..."
winget install --id Microsoft.HEVCVideoExtension -e --accept-source-agreements --accept-package-agreements

Write-Host "Done. Restart GV Pixara if it was open."
Write-Host "Note: HEIC import in the default public build is not enabled yet; this prepares your PC for upcoming Windows-native HEIC support."
