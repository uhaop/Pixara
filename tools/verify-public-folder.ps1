# Verify public/ contains only the expected release-kit files.

$ErrorActionPreference = "Stop"
$ProjectRoot = Split-Path -Parent (Split-Path -Parent $MyInvocation.MyCommand.Path)
$PublicDir = Join-Path $ProjectRoot "public"

$ExpectedFiles = @(
    "export-manifest.json",
    "README.md",
    "LICENSE",
    "THIRD_PARTY_NOTICES.md",
    "PUBLISHING.md",
    "setup-windows-heic.ps1",
    "pixara-icon.png",
    "EXPECTED.md"
)

if (-not (Test-Path $PublicDir)) {
    throw "Missing public/ directory at $PublicDir"
}

$actual = Get-ChildItem -Path $PublicDir -File | ForEach-Object { $_.Name }
$unexpected = $actual | Where-Object { $_ -notin $ExpectedFiles }
$missing = $ExpectedFiles | Where-Object { $_ -ne "EXPECTED.md" -and -not (Test-Path (Join-Path $PublicDir $_)) }

if ($unexpected.Count -gt 0) {
    throw "Unexpected files in public/: $($unexpected -join ', ')"
}
if ($missing.Count -gt 0) {
    throw "Missing required files in public/: $($missing -join ', ')"
}

$manifestPath = Join-Path $PublicDir "export-manifest.json"
$manifest = Get-Content $manifestPath -Raw | ConvertFrom-Json
foreach ($rel in $manifest.includePaths) {
    $source = Join-Path $ProjectRoot $rel
    if (-not (Test-Path $source)) {
        throw "export-manifest.json references missing path: $rel"
    }
}

$tauriIcon = Join-Path $ProjectRoot "src-tauri\icons\128x128.png"
$publicIcon = Join-Path $PublicDir "pixara-icon.png"
if ((Test-Path $tauriIcon) -and (Test-Path $publicIcon)) {
    $a = (Get-FileHash $tauriIcon).Hash
    $p = (Get-FileHash $publicIcon).Hash
    if ($a -ne $p) {
        Write-Warning "src-tauri/icons/128x128.png and public/pixara-icon.png differ - sync favicon before release."
    }
}

if ($manifest.github.owner -eq "YOUR_ORG" -or $manifest.github.repo -eq "YOUR_REPO") {
    Write-Warning "Set github.owner and github.repo in public/export-manifest.json before publishing."
}

Write-Host ('public/ folder OK ({0} files; manifest paths exist).' -f $ExpectedFiles.Count)
