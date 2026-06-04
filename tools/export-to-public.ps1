# Copy a public-safe tree for pushing to the GitHub repository.
# Does not duplicate source into public/ — copies app paths per export-manifest.json.

param(
    [Parameter(Mandatory = $true)]
    [string]$Destination,

    [string]$ManifestPath = ""
)

$ErrorActionPreference = "Stop"

function Write-Utf8NoBom {
    param([string]$Path, [string]$Content)
    $utf8NoBom = New-Object System.Text.UTF8Encoding $false
    [System.IO.File]::WriteAllText($Path, $Content, $utf8NoBom)
}

function Remove-Utf8BomFromFile {
    param([string]$Path)
    if (-not (Test-Path $Path)) {
        return
    }
    $bytes = [System.IO.File]::ReadAllBytes($Path)
    if ($bytes.Length -ge 3 -and $bytes[0] -eq 0xEF -and $bytes[1] -eq 0xBB -and $bytes[2] -eq 0xBF) {
        $text = [System.IO.File]::ReadAllText($Path)
        if ($text.Length -gt 0 -and [int][char]$text[0] -eq 0xFEFF) {
            $text = $text.Substring(1)
        }
        Write-Utf8NoBom -Path $Path -Content $text
    }
}

$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$ProjectRoot = Split-Path -Parent $ScriptDir
if (-not $ManifestPath) {
    $ManifestPath = Join-Path $ProjectRoot "public\export-manifest.json"
}
$Manifest = Get-Content $ManifestPath -Raw | ConvertFrom-Json

$Owner = $Manifest.github.owner
$Repo = $Manifest.github.repo
$RepoUrl = "https://github.com/$Owner/$Repo"

if ($Owner -eq "YOUR_ORG" -or $Repo -eq "YOUR_REPO") {
    Write-Warning "Set github.owner and github.repo in public/export-manifest.json before publishing."
}

if (Test-Path $Destination) {
    try {
        Remove-Item -Path $Destination -Recurse -Force -ErrorAction Stop
    } catch {
        throw "Cannot clear export destination (files may be locked). Choose an empty folder or close processes using: $Destination"
    }
}
New-Item -ItemType Directory -Force -Path $Destination | Out-Null

foreach ($rel in $Manifest.includePaths) {
    $source = Join-Path $ProjectRoot $rel
    if (-not (Test-Path $source)) {
        Write-Warning "Skipping missing path: $rel"
        continue
    }
    $target = Join-Path $Destination $rel
    $parent = Split-Path $target -Parent
    if ($parent -and -not (Test-Path $parent)) {
        New-Item -ItemType Directory -Force -Path $parent | Out-Null
    }
    if (Test-Path $source -PathType Container) {
        $excludeDirs = @("target", "node_modules")
        New-Item -ItemType Directory -Force -Path $target | Out-Null
        $robocopyArgs = @(
            $source,
            $target,
            "/E",
            "/NFL", "/NDL", "/NJH", "/NJS", "/nc", "/ns", "/np"
        )
        foreach ($dir in $excludeDirs) {
            if (Test-Path (Join-Path $source $dir)) {
                $robocopyArgs += "/XD"
                $robocopyArgs += (Join-Path $source $dir)
            }
        }
        & robocopy @robocopyArgs | Out-Null
        if ($LASTEXITCODE -ge 8) {
            throw "robocopy failed for $rel (exit $LASTEXITCODE)"
        }
    } else {
        Copy-Item -Path $source -Destination $target -Force
    }
}

$ReadmeSource = Join-Path $ProjectRoot $Manifest.readmeSource
$ReadmeDest = Join-Path $Destination "README.md"
$readme = Get-Content $ReadmeSource -Raw -Encoding UTF8
$readme = $readme -replace "YOUR_ORG", $Owner
$readme = $readme -replace "YOUR_REPO", $Repo
Write-Utf8NoBom -Path $ReadmeDest -Content $readme

$LicenseSource = Join-Path $ProjectRoot $Manifest.licenseSource
Copy-Item -Path $LicenseSource -Destination (Join-Path $Destination "LICENSE") -Force

$NoticesSource = Join-Path $ProjectRoot "public\THIRD_PARTY_NOTICES.md"
if (Test-Path $NoticesSource) {
    Copy-Item -Path $NoticesSource -Destination (Join-Path $Destination "THIRD_PARTY_NOTICES.md") -Force
}

$LogoSource = Join-Path $ProjectRoot "public\gv-logo.png"
if (Test-Path $LogoSource) {
    Copy-Item -Path $LogoSource -Destination (Join-Path $Destination "gv-logo.png") -Force
}

# Public package.json: allow publishing metadata
$PkgPath = Join-Path $Destination "package.json"
if (Test-Path $PkgPath) {
    $pkg = Get-Content $PkgPath -Raw | ConvertFrom-Json
    $pkg.private = $false
    if (-not $pkg.repository) {
        $pkg | Add-Member -NotePropertyName repository -NotePropertyValue "git+$RepoUrl.git" -Force
    }
    if (-not $pkg.homepage) {
        $pkg | Add-Member -NotePropertyName homepage -NotePropertyValue $RepoUrl -Force
    }
    Write-Utf8NoBom -Path $PkgPath -Content ($pkg | ConvertTo-Json -Depth 10)
}

foreach ($json in @("tsconfig.json", "tsconfig.node.json", "components.json")) {
    Remove-Utf8BomFromFile -Path (Join-Path $Destination $json)
}

foreach ($junk in @("node_modules", "src-tauri\target", "dist", "dist-public", "dist-portable")) {
    $junkPath = Join-Path $Destination $junk
    if (Test-Path $junkPath) {
        Remove-Item -Path $junkPath -Recurse -Force -ErrorAction SilentlyContinue
    }
}

# Public repo: default build without vcpkg / HEIC DLLs
$CargoPath = Join-Path $Destination "src-tauri\Cargo.toml"
if (Test-Path $CargoPath) {
    $cargo = Get-Content $CargoPath -Raw
    $cargo = $cargo -replace 'default = \["heic"\]', 'default = []'
    Write-Utf8NoBom -Path $CargoPath -Content $cargo
}

Write-Host "Public export written to $Destination"
Write-Host "Repository URL: $RepoUrl"
Write-Host "Next: cd $Destination; git init; git remote add origin $RepoUrl.git"
