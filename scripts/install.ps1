<#
.SYNOPSIS
    Automated installer for TIDE (Terminal Integrated Development Environment) on Windows.

.DESCRIPTION
    Downloads and installs the latest pre-built TIDE release binary into ~/.tide/bin
    and automatically registers it in your User PATH environment variable.

.EXAMPLE
    irm https://raw.githubusercontent.com/kurarensu16/TIDE/main/scripts/install.ps1 | iex
#>

[CmdletBinding()]
param(
    [string]$Version = "latest",
    [string]$InstallDir = "$HOME\.tide\bin"
)

$ErrorActionPreference = 'Stop'

# Enable TLS 1.2+ for downloads
[Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12 -bor [Net.SecurityProtocolType]::Tls13

function Write-TideHeader {
    Write-Host ""
    Write-Host "  =================================================" -ForegroundColor Cyan
    Write-Host "     TIDE - Terminal Integrated Development Env    " -ForegroundColor White
    Write-Host "     Parallel Coding-Agent Orchestration Platform  " -ForegroundColor Gray
    Write-Host "  =================================================" -ForegroundColor Cyan
    Write-Host ""
}

function Get-LatestReleaseTag {
    param([string]$Repo = "kurarensu16/TIDE")
    
    $headers = @{ "User-Agent" = "TIDE-Installer-PowerShell" }
    try {
        $apiUri = "https://api.github.com/repos/$Repo/releases/latest"
        $release = Invoke-RestMethod -Uri $apiUri -Headers $headers -Method Get
        if ($release.tag_name) {
            return $release.tag_name
        }
    } catch {
        Write-Warning "Failed to query GitHub Releases API: $_"
    }
    
    # Fallback to default v0.1.0 if API is unreachable or rate-limited
    return "v0.1.0"
}

try {
    Write-TideHeader

    # 1. Architecture Check
    $is64Bit = [Environment]::Is64BitOperatingSystem
    if (-not $is64Bit) {
        throw "TIDE currently supports 64-bit Windows systems (x86_64). 32-bit is not supported."
    }

    # 2. Determine target version
    if ($Version -eq "latest") {
        Write-Host "==> Checking latest release on GitHub..." -ForegroundColor Yellow
        $tag = Get-LatestReleaseTag -Repo "kurarensu16/TIDE"
    } else {
        $tag = if ($Version.StartsWith("v")) { $Version } else { "v$Version" }
    }
    Write-Host "==> Installing TIDE version $tag..." -ForegroundColor Green

    # 3. Form download URL
    $archiveName = "tide-$tag-windows-x86_64.zip"
    $downloadUrl = "https://github.com/kurarensu16/TIDE/releases/download/$tag/$archiveName"

    $tempDir = [System.IO.Path]::Combine([System.IO.Path]::GetTempPath(), [System.Guid]::NewGuid().ToString())
    New-Item -ItemType Directory -Path $tempDir -Force | Out-Null
    $zipPath = Join-Path $tempDir $archiveName

    Write-Host "==> Downloading $archiveName from GitHub Releases..." -ForegroundColor Yellow
    try {
        Invoke-WebRequest -Uri $downloadUrl -OutFile $zipPath -UseBasicParsing
    } catch {
        throw "Download failed from $downloadUrl : $_"
    }

    # 4. Extract archive
    Write-Host "==> Extracting binary..." -ForegroundColor Yellow
    $extractDir = Join-Path $tempDir "extracted"
    Expand-Archive -Path $zipPath -DestinationPath $extractDir -Force

    # Locate tide.exe (handle possible root or subfolder extraction)
    $exeSource = Get-ChildItem -Path $extractDir -Filter "tide.exe" -Recurse | Select-Object -First 1
    if (-not $exeSource) {
        throw "Could not locate 'tide.exe' inside the downloaded archive."
    }

    # 5. Prepare target directory
    if (-not (Test-Path $InstallDir)) {
        New-Item -ItemType Directory -Path $InstallDir -Force | Out-Null
    }
    $targetExe = Join-Path $InstallDir "tide.exe"

    Copy-Item -Path $exeSource.FullName -Destination $targetExe -Force
    Write-Host "==> Installed binary to: $targetExe" -ForegroundColor Green

    # 6. Register in User PATH environment variable
    $userPath = [Environment]::GetEnvironmentVariable("Path", [EnvironmentVariableTarget]::User)
    $normalizedInstallDir = (Resolve-Path $InstallDir).Path

    $pathEntries = if ($userPath) { $userPath -split ';' } else { @() }
    $alreadyInPath = $false
    foreach ($entry in $pathEntries) {
        if ($entry.Trim().TrimEnd('\') -eq $normalizedInstallDir.TrimEnd('\')) {
            $alreadyInPath = $true
            break
        }
    }

    if (-not $alreadyInPath) {
        Write-Host "==> Adding $normalizedInstallDir to User PATH..." -ForegroundColor Yellow
        $newPath = if ($userPath) { "$userPath;$normalizedInstallDir" } else { $normalizedInstallDir }
        [Environment]::SetEnvironmentVariable("Path", $newPath, [EnvironmentVariableTarget]::User)
        Write-Host "==> User PATH updated." -ForegroundColor Green
    } else {
        Write-Host "==> $normalizedInstallDir is already in User PATH." -ForegroundColor Gray
    }

    # Also update current session PATH so user can run immediately
    if ($env:Path -notlike "*$normalizedInstallDir*") {
        $env:Path = "$normalizedInstallDir;$env:Path"
    }

    # Clean up temporary files
    Remove-Item -Path $tempDir -Recurse -Force -ErrorAction SilentlyContinue

    Write-Host ""
    Write-Host "  SUCCESS! TIDE $tag has been installed successfully." -ForegroundColor Green
    Write-Host ""
    Write-Host "  To launch TIDE in your current workspace, run:" -ForegroundColor White
    Write-Host "    tide ." -ForegroundColor Cyan
    Write-Host ""
    Write-Host "  Or open a specific project folder:" -ForegroundColor White
    Write-Host "    tide 'C:\path\to\project'" -ForegroundColor Cyan
    Write-Host ""
    Write-Host "  Documentation & Guides: https://kurarensu16.github.io/TIDE/" -ForegroundColor Gray
    Write-Host ""

} catch {
    Write-Host ""
    Write-Host "  INSTALLATION ERROR:" -ForegroundColor Red
    Write-Host "  $_" -ForegroundColor Red
    Write-Host ""
    Write-Host "  If you encounter issues, you can download the release directly from:" -ForegroundColor Yellow
    Write-Host "  https://github.com/kurarensu16/TIDE/releases" -ForegroundColor Cyan
    Write-Host ""
    exit 1
}

