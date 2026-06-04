# Verify a public portable folder: gv-pixara.exe only (no HEIC codec DLLs).

param(
    [Parameter(Mandatory = $true)]
    [string]$PortableDir
)

$ErrorActionPreference = "Stop"

$ExePath = Join-Path $PortableDir "gv-pixara.exe"
if (-not (Test-Path $ExePath)) {
    throw "gv-pixara.exe not found in $PortableDir"
}

$Forbidden = @("heif.dll", "libde265.dll", "libx265.dll", "aom.dll")
foreach ($dll in $Forbidden) {
    $path = Join-Path $PortableDir $dll
    if (Test-Path $path) {
        throw "Public portable must not include $dll (found at $path)"
    }
}

Write-Host "Public portable folder verified: $PortableDir"
