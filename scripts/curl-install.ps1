<#
.SYNOPSIS
    Download and install a pre-built squrl binary on Windows.

.DESCRIPTION
    Remote installer for squrl pre-built Windows binaries.
    Usage: irm https://raw.githubusercontent.com/FloodCreux/squrl/main/scripts/curl-install.ps1 | iex

.PARAMETER Version
    Version to install. Defaults to latest release.

.PARAMETER InstallDir
    Installation directory. Defaults to "$env:LOCALAPPDATA\squrl\bin".

.EXAMPLE
    .\curl-install.ps1
    .\curl-install.ps1 -Version "1.0.0"
    .\curl-install.ps1 -InstallDir "C:\Tools\squrl\bin"
#>

param(
    [string]$Version = $env:VERSION,
    [string]$InstallDir = $(if ($env:INSTALL_DIR) { $env:INSTALL_DIR } else { "$env:LOCALAPPDATA\squrl\bin" })
)

$ErrorActionPreference = "Stop"

$RepoApi = "https://api.github.com/repos/FloodCreux/squrl"
$RepoReleases = "https://github.com/FloodCreux/squrl/releases/download"

# Detect architecture
function Get-TargetArch {
    switch ($env:PROCESSOR_ARCHITECTURE) {
        "AMD64" { return "x86_64" }
        "ARM64" { return "aarch64" }
        default { throw "Unsupported architecture: $env:PROCESSOR_ARCHITECTURE" }
    }
}

# Resolve version
function Resolve-Version {
    if ($Version) {
        return $Version
    }

    Write-Host "Fetching latest version..."
    $release = Invoke-RestMethod -Uri "$RepoApi/releases/latest" -Headers @{ "User-Agent" = "squrl-installer" }
    $tag = $release.tag_name

    if (-not $tag) {
        throw "Could not determine latest version. Set -Version parameter manually."
    }

    # Strip leading 'v' if present
    return $tag -replace "^v", ""
}

$Arch = Get-TargetArch
$ResolvedVersion = Resolve-Version
$Target = "$Arch-pc-windows-msvc"
$Archive = "squrl-v$ResolvedVersion-$Target.zip"
$DownloadUrl = "$RepoReleases/v$ResolvedVersion/$Archive"

Write-Host "Installing squrl v$ResolvedVersion for $Target..."

# Create temp directory
$TmpDir = Join-Path ([System.IO.Path]::GetTempPath()) "squrl-install-$([System.Guid]::NewGuid().ToString('N'))"
New-Item -ItemType Directory -Path $TmpDir -Force | Out-Null

try {
    # Download
    $ArchivePath = Join-Path $TmpDir $Archive
    Write-Host "Downloading $DownloadUrl..."
    Invoke-WebRequest -Uri $DownloadUrl -OutFile $ArchivePath -UseBasicParsing

    # Extract
    Expand-Archive -Path $ArchivePath -DestinationPath $TmpDir -Force

    $ExtractedBin = Join-Path $TmpDir "squrl.exe"
    if (-not (Test-Path $ExtractedBin)) {
        throw "Archive did not contain squrl.exe"
    }

    # Install binary
    New-Item -ItemType Directory -Path $InstallDir -Force | Out-Null
    Copy-Item $ExtractedBin (Join-Path $InstallDir "squrl.exe") -Force
    Write-Host "Installed squrl to $InstallDir\squrl.exe"

    # Install completions
    $squrl = Join-Path $InstallDir "squrl.exe"
    $Prefix = Split-Path $InstallDir -Parent
    $CompletionDir = Join-Path $Prefix "completions"
    New-Item -ItemType Directory -Path $CompletionDir -Force | Out-Null

    Write-Host "Installing shell completions..."

    # PowerShell completions
    & $squrl completions powershell $TmpDir 2>$null
    $psSrc = Join-Path $TmpDir "squrl.ps1"
    if (Test-Path $psSrc) {
        Copy-Item $psSrc (Join-Path $CompletionDir "squrl.ps1") -Force
        Write-Host "  Installed PowerShell completions to $CompletionDir\squrl.ps1"
    }
}
finally {
    Remove-Item $TmpDir -Recurse -Force -ErrorAction SilentlyContinue
}

# PATH check
$UserPath = [Environment]::GetEnvironmentVariable("Path", "User")
if ($UserPath -notlike "*$InstallDir*") {
    Write-Host ""
    Write-Host "Warning: $InstallDir is not in your PATH."
    Write-Host "Add it with:"
    Write-Host "  [Environment]::SetEnvironmentVariable('Path', '$InstallDir;' + [Environment]::GetEnvironmentVariable('Path', 'User'), 'User')"
    Write-Host ""

    $answer = Read-Host "Add to PATH now? (y/N)"
    if ($answer -eq "y" -or $answer -eq "Y") {
        [Environment]::SetEnvironmentVariable("Path", "$InstallDir;$UserPath", "User")
        $env:Path = "$InstallDir;$env:Path"
        Write-Host "  Added $InstallDir to user PATH. Restart your terminal for it to take effect in new sessions."
    }
}

Write-Host ""
Write-Host "squrl v$ResolvedVersion installed successfully!"
Write-Host "Run 'squrl --version' to verify."
