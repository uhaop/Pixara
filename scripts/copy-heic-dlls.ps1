# Copy libheif runtime DLLs from vcpkg next to pixara.exe.
# Required because pixara links heif.dll dynamically (VCPKGRS_DYNAMIC=1).

param(
    [Parameter(Mandatory = $true)]
    [string]$Destination,

    [string]$VcpkgRoot = $(if ($env:VCPKG_ROOT) { $env:VCPKG_ROOT } else { Join-Path $env:USERPROFILE "vcpkg" })
)

$ErrorActionPreference = "Stop"

if ($env:PIXARA_PUBLIC -eq "1") {
    Write-Host "PIXARA_PUBLIC=1 - skipping HEIC DLL copy (public build)."
    exit 0
}

$RequiredDlls = @(
    "heif.dll",
    "libde265.dll",
    "libx265.dll"
)

$SourceDir = Join-Path $VcpkgRoot "installed\x64-windows\bin"
if (-not (Test-Path $SourceDir)) {
    throw "vcpkg bin directory not found: $SourceDir. Install with: vcpkg install libheif:x64-windows"
}

New-Item -ItemType Directory -Force -Path $Destination | Out-Null

foreach ($name in $RequiredDlls) {
    $source = Join-Path $SourceDir $name
    if (-not (Test-Path $source)) {
        throw "Missing required DLL in vcpkg: $source"
    }
    $dest = Join-Path $Destination $name
    try {
        Copy-Item -Path $source -Destination $dest -Force
    } catch [System.IO.IOException] {
        if ($name -eq (Split-Path $dest -Leaf) -and (Test-Path $dest)) {
            Write-Warning "Could not overwrite locked $name; existing copy kept. Close pixara.exe and re-run if updating."
            continue
        }
        throw
    }
    Write-Host "Copied $name"
}

Write-Host "HEIC DLLs copied to $Destination"
