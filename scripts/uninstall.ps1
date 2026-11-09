<#
.SYNOPSIS
    Automated uninstaller for TIDE (Terminal Integrated Development Environment) on Windows.

.DESCRIPTION
    Removes the TIDE binary from ~/.tide/bin, cleans up User PATH environment variable,
    and optionally removes global configuration files.

.EXAMPLE
    irm https://raw.githubusercontent.com/kurarensu16/TIDE/main/scripts/uninstall.ps1 | iex
#>

[CmdletBinding()]
param(
    [switch]$KeepConfig = $false
)

$ErrorActionPreference = 'SilentlyContinue'

Write-Host ""
Write-Host "  =================================================" -ForegroundColor Cyan
Write-Host "     TIDE - Uninstaller                            " -ForegroundColor White
Write-Host "  =================================================" -ForegroundColor Cyan
Write-Host ""

$tideDir = "$HOME\.tide"
$tideBin = "$HOME\.tide\bin"

# 1. Remove binary and ~/.tide directory
if (Test-Path $tideDir) {
    Write-Host "==> Removing TIDE files from $tideDir..." -ForegroundColor Yellow
    Remove-Item -Recurse -Force $tideDir -ErrorAction SilentlyContinue
    Write-Host "==> Removed $tideDir." -ForegroundColor Green
} else {
    Write-Host "==> No installation found at $tideDir." -ForegroundColor Gray
}

# 2. Clean User PATH
Write-Host "==> Checking User PATH environment variable..." -ForegroundColor Yellow
$userPath = [Environment]::GetEnvironmentVariable("Path", [EnvironmentVariableTarget]::User)

if ($userPath) {
    $normalizedTideBin = $tideBin.TrimEnd('\').ToLower()
    $pathEntries = $userPath -split ';'
    $filteredEntries = @()
    $removed = $false

    foreach ($entry in $pathEntries) {
        $trimmed = $entry.Trim()
        if ($trimmed.Length -eq 0) {
            continue
        }
        if ($trimmed.TrimEnd('\').ToLower() -eq $normalizedTideBin -or $trimmed.ToLower().EndsWith("\.tide\bin")) {
            $removed = $true
        } else {
            $filteredEntries += $trimmed
        }
    }

    if ($removed) {
        $newPath = $filteredEntries -join ';'
        [Environment]::SetEnvironmentVariable("Path", $newPath, [EnvironmentVariableTarget]::User)
        Write-Host "==> Cleaned up TIDE from User PATH." -ForegroundColor Green
    } else {
        Write-Host "==> TIDE was not in User PATH." -ForegroundColor Gray
    }
}

# Also remove from current session PATH
if ($env:Path) {
    $sessionEntries = ($env:Path -split ';') | Where-Object { 
        $_.Trim().TrimEnd('\').ToLower() -ne $tideBin.TrimEnd('\').ToLower() -and -not $_.ToLower().EndsWith("\.tide\bin") 
    }
    $env:Path = $sessionEntries -join ';'
}

# 3. Clean global config unless -KeepConfig is specified
if (-not $KeepConfig) {
    $configDir = "$env:APPDATA\tide"
    if (Test-Path $configDir) {
        Write-Host "==> Removing global configuration at $configDir..." -ForegroundColor Yellow
        Remove-Item -Recurse -Force $configDir -ErrorAction SilentlyContinue
        Write-Host "==> Removed configuration directory." -ForegroundColor Green
    }
} else {
    Write-Host "==> Preserving global configuration (-KeepConfig specified)." -ForegroundColor Gray
}

Write-Host ""
Write-Host "  SUCCESS: TIDE has been completely uninstalled from this system." -ForegroundColor Green
Write-Host ""

