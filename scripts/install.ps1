<#
.SYNOPSIS
    Build and install squrl from source on Windows.

.PARAMETER Prefix
    Installation prefix directory. Defaults to "$env:LOCALAPPDATA\squrl".

.PARAMETER Uninstall
    Remove installed files.

.EXAMPLE
    .\install.ps1
    .\install.ps1 -Prefix "C:\Tools\squrl"
    .\install.ps1 -Uninstall
#>

param(
    [string]$Prefix = "$env:LOCALAPPDATA\squrl",
    [switch]$Uninstall
)

$ErrorActionPreference = "Stop"

$BinDir = Join-Path $Prefix "bin"
$CompletionDir = Join-Path $Prefix "completions"

$InstalledFiles = @(
    (Join-Path $BinDir "squrl.exe"),
    (Join-Path $CompletionDir "squrl.ps1"),
    (Join-Path $CompletionDir "_squrl"),
    (Join-Path $CompletionDir "squrl.bash"),
    (Join-Path $CompletionDir "squrl.fish")
)

if ($Uninstall) {
    Write-Host "Uninstalling squrl from $Prefix..."
    foreach ($f in $InstalledFiles) {
        if (Test-Path $f) {
            Remove-Item $f -Force
            Write-Host "  Removed $f"
        }
    }

    # Clean up empty directories
    foreach ($dir in @($CompletionDir, $BinDir, $Prefix)) {
        if ((Test-Path $dir) -and @(Get-ChildItem $dir).Count -eq 0) {
            Remove-Item $dir -Force
            Write-Host "  Removed $dir"
        }
    }

    Write-Host "Done."
    exit 0
}

# Check prerequisites
if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
    Write-Error "cargo is not installed. Install Rust from https://rustup.rs/"
}

$nightlyInstalled = & rustup toolchain list 2>$null | Select-String "nightly"
if (-not $nightlyInstalled) {
    Write-Error "Rust nightly toolchain is not installed. Install it with: rustup toolchain install nightly"
}

# Build
Write-Host "Building squrl in release mode..."
& cargo build --release
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

$SqurlBin = "target\release\squrl.exe"

if (-not (Test-Path $SqurlBin)) {
    Write-Error "Build did not produce $SqurlBin"
}

# Install binary
Write-Host "Installing binary to $BinDir..."
New-Item -ItemType Directory -Path $BinDir -Force | Out-Null
Copy-Item $SqurlBin (Join-Path $BinDir "squrl.exe") -Force

# Install completions
$TmpDir = Join-Path ([System.IO.Path]::GetTempPath()) "squrl-install-$([System.Guid]::NewGuid().ToString('N'))"
New-Item -ItemType Directory -Path $TmpDir -Force | Out-Null

try {
    Write-Host "Installing shell completions..."
    New-Item -ItemType Directory -Path $CompletionDir -Force | Out-Null

    $squrl = Join-Path $BinDir "squrl.exe"

    # PowerShell completions
    & $squrl completions powershell $TmpDir 2>$null
    $psSrc = Join-Path $TmpDir "squrl.ps1"
    if (Test-Path $psSrc) {
        Copy-Item $psSrc (Join-Path $CompletionDir "squrl.ps1") -Force
        Write-Host "  Installed PowerShell completions to $CompletionDir\squrl.ps1"
    }

    # Also generate completions for other shells if they happen to be available
    foreach ($entry in @(
        @{ Shell = "bash";  Src = "squrl.bash";  Dest = "squrl.bash" },
        @{ Shell = "zsh";   Src = "_squrl";      Dest = "_squrl" },
        @{ Shell = "fish";  Src = "squrl.fish";  Dest = "squrl.fish" }
    )) {
        & $squrl completions $entry.Shell $TmpDir 2>$null
        $srcPath = Join-Path $TmpDir $entry.Src
        if (Test-Path $srcPath) {
            Copy-Item $srcPath (Join-Path $CompletionDir $entry.Dest) -Force
            Write-Host "  Installed $($entry.Shell) completions to $CompletionDir\$($entry.Dest)"
        }
    }
}
finally {
    Remove-Item $TmpDir -Recurse -Force -ErrorAction SilentlyContinue
}

# PATH check
$UserPath = [Environment]::GetEnvironmentVariable("Path", "User")
if ($UserPath -notlike "*$BinDir*") {
    Write-Host ""
    Write-Host "Warning: $BinDir is not in your PATH."
    Write-Host "Add it with:"
    Write-Host "  [Environment]::SetEnvironmentVariable('Path', '$BinDir;' + [Environment]::GetEnvironmentVariable('Path', 'User'), 'User')"
    Write-Host ""

    $answer = Read-Host "Add to PATH now? (y/N)"
    if ($answer -eq "y" -or $answer -eq "Y") {
        [Environment]::SetEnvironmentVariable("Path", "$BinDir;$UserPath", "User")
        $env:Path = "$BinDir;$env:Path"
        Write-Host "  Added $BinDir to user PATH. Restart your terminal for it to take effect in new sessions."
    }
}

Write-Host ""
Write-Host "squrl installed successfully!"
Write-Host "Run 'squrl --version' to verify."
