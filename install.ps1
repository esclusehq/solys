#!/usr/bin/env pwsh
# Escluse Agent Windows Installer
# Usage: powershell -c "irm https://get.esluce.com/latest/install.ps1 | iex"
#        powershell -c "irm https://get.esluce.com/latest/install.ps1 | iex" -Version 1.2.3

param(
    [string]$Version = "latest"
)

$ErrorActionPreference = "Stop"
$Repo = "escluse/escluse"

function Write-Info  { Write-Host "INFO: $args" -ForegroundColor Cyan }
function Write-Ok    { Write-Host "OK:   $args" -ForegroundColor Green }
function Write-Warn  { Write-Host "WARN: $args" -ForegroundColor Yellow }
function Write-Fail  { Write-Host "FAIL: $args" -ForegroundColor Red; exit 1 }

# --- Determine platform ---
function Get-Platform {
    $arch = switch ($env:PROCESSOR_ARCHITECTURE) {
        "AMD64" { "x86_64" }
        "ARM64" { "aarch64" }
        default { throw "Unsupported architecture: $env:PROCESSOR_ARCHITECTURE" }
    }
    Write-Info "Detected architecture: $arch"
    return $arch
}

# --- Build download URLs ---
function Get-Urls {
    param([string]$Version)

    if ($Version -eq "latest") {
        $baseUrl = "https://get.esluce.com/latest"
    } else {
        # Strip leading 'v' if present
        $v = $Version -replace '^v', ''
        $baseUrl = "https://get.esluce.com/v$v"
    }

    $archive = "solys-windows-${Script:arch}.zip"
    $binary = "escluse-agent.exe"

    return @{
        BaseUrl     = $baseUrl
        Archive     = $archive
        Binary      = $binary
        ArchiveUrl  = "$baseUrl/$archive"
        ChecksumUrl = "$baseUrl/SHA256SUMS.txt"
        BundleUrl   = "$baseUrl/SHA256SUMS.txt.bundle"
    }
}

# --- Determine install directory ---
function Get-InstallDir {
    $primary = Join-Path $env:ProgramFiles "Escluse"
    $fallback = Join-Path $env:LOCALAPPDATA "Escluse"

    try {
        $testPath = Join-Path $env:ProgramFiles "."
        if (-not (Test-Path $testPath)) {
            New-Item -ItemType Directory -Force -Path $env:ProgramFiles | Out-Null
        }
        return $primary
    } catch {
        Write-Warn "ProgramFiles not writable, using LOCALAPPDATA instead"
        return $fallback
    }
}

# --- Download artifacts ---
function Download-Artifacts {
    param($Urls)

    $Script:tmpDir = Join-Path $env:TEMP "escluse-install"
    if (Test-Path $Script:tmpDir) {
        Remove-Item -Recurse -Force $Script:tmpDir
    }
    New-Item -ItemType Directory -Force -Path $Script:tmpDir | Out-Null

    Write-Info "Downloading $($Urls.Archive)..."
    try {
        Invoke-WebRequest -Uri $Urls.ArchiveUrl -OutFile (Join-Path $Script:tmpDir $Urls.Archive) -UseBasicParsing
    } catch {
        throw "Failed to download $($Urls.ArchiveUrl): $_"
    }

    Write-Info "Downloading SHA256SUMS.txt..."
    try {
        Invoke-WebRequest -Uri $Urls.ChecksumUrl -OutFile (Join-Path $Script:tmpDir "SHA256SUMS.txt") -UseBasicParsing
    } catch {
        throw "Failed to download $($Urls.ChecksumUrl): $_"
    }
}

# --- Verify SHA256 checksum ---
function Verify-Checksum {
    param($Urls)

    Write-Info "Verifying SHA256 checksum..."
    $checksums = Get-Content (Join-Path $Script:tmpDir "SHA256SUMS.txt")
    $expectedHash = ($checksums | Select-String -Pattern $Urls.Binary) -split '\s+' | Select-Object -First 1

    if (-not $expectedHash) {
        throw "Checksum entry not found for $($Urls.Binary) in SHA256SUMS.txt"
    }

    $actualHash = (Get-FileHash (Join-Path $Script:tmpDir $Urls.Archive) -Algorithm SHA256).Hash.ToLower()
    $expectedHash = $expectedHash.ToLower()

    if ($actualHash -ne $expectedHash) {
        throw "Checksum mismatch for $($Urls.Archive). Expected: $expectedHash, Actual: $actualHash"
    }

    Write-Ok "Checksum verified successfully."
}

# --- Optional cosign verification ---
function Verify-Cosign {
    param($Urls)

    $cosignPath = Get-Command "cosign.exe" -ErrorAction SilentlyContinue
    if (-not $cosignPath) {
        $cosignPath = Get-Command "cosign" -ErrorAction SilentlyContinue
    }

    if ($cosignPath) {
        Write-Info "Cosign detected. Verifying signature..."
        try {
            Invoke-WebRequest -Uri $Urls.BundleUrl -OutFile (Join-Path $Script:tmpDir "SHA256SUMS.txt.bundle") -UseBasicParsing
            & $cosignPath.Source verify-blob (Join-Path $Script:tmpDir "SHA256SUMS.txt") `
                --bundle (Join-Path $Script:tmpDir "SHA256SUMS.txt.bundle") `
                --certificate-identity-regexp "https://github.com/${Repo}/.github/workflows/release.yml@refs/tags/v" `
                --certificate-oidc-issuer "https://token.actions.githubusercontent.com"
            Write-Ok "Cosign signature verified."
        } catch {
            Write-Warn "Cosign verification warning: signature could not be verified (non-fatal): $_"
        }
    } else {
        Write-Info "Cosign not found. Skipping signature verification."
    }
}

# --- Extract and install binary ---
function Install-Binary {
    param($Urls, $InstallDir)

    Write-Info "Extracting archive..."
    try {
        Expand-Archive -Path (Join-Path $Script:tmpDir $Urls.Archive) -DestinationPath $Script:tmpDir -Force
    } catch {
        throw "Failed to extract $($Urls.Archive): $_"
    }

    $binaryPath = Join-Path $Script:tmpDir $Urls.Binary
    if (-not (Test-Path $binaryPath)) {
        throw "Binary $($Urls.Binary) not found in extracted archive."
    }

    Write-Info "Installing $($Urls.Binary) to $InstallDir..."
    New-Item -ItemType Directory -Force -Path $InstallDir | Out-Null
    Copy-Item $binaryPath (Join-Path $InstallDir $Urls.Binary) -Force

    Write-Ok "Installed $($Urls.Binary) to $(Join-Path $InstallDir $Urls.Binary)"
}

# --- Update User PATH ---
function Update-Path {
    param($InstallDir)

    $userPath = [Environment]::GetEnvironmentVariable("Path", "User")
    if ($userPath -notlike "*$InstallDir*") {
        Write-Info "Adding $InstallDir to User PATH..."
        [Environment]::SetEnvironmentVariable("Path", "$userPath;$InstallDir", "User")
        Write-Ok "PATH updated. Restart your terminal for changes to take effect."
    } else {
        Write-Info "$InstallDir is already in User PATH."
    }
}

# --- Main ---
function Main {
    Write-Host ""
    Write-Host "Escluse Agent Windows Installer" -ForegroundColor Cyan
    Write-Host "================================" -ForegroundColor Cyan
    Write-Host ""

    try {
        $Script:arch = Get-Platform
        $urls = Get-Urls -Version $Version
        $installDir = Get-InstallDir

        Write-Info "Install directory: $installDir"
        if ($Version -eq "latest") {
            Write-Info "Using latest version"
        } else {
            Write-Info "Using version: v$($Version -replace '^v', '')"
        }

        Download-Artifacts -Urls $urls
        Verify-Checksum -Urls $urls
        Verify-Cosign -Urls $urls
        Install-Binary -Urls $urls -InstallDir $installDir
        Update-Path -InstallDir $installDir

        Write-Host ""
        Write-Ok "┌──────────────────────────────────────────────────────────┐"
        Write-Ok "│  Escluse Agent installed successfully!                   │"
        Write-Ok "│                                                          │"
        Write-Ok "│  Binary:  $(Join-Path $installDir $urls.Binary)"
        Write-Ok "│  Version: $Version"
        Write-Ok "│  Usage:   escluse-agent --help                           │"
        Write-Ok "└──────────────────────────────────────────────────────────┘"
        Write-Host ""
        Write-Info "Run 'escluse-agent --help' to get started."

    } catch {
        Write-Fail "Installation failed: $_"
    }
}

Main
