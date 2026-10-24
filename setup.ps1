# setup.ps1: builds aliases.exe, installs it and adds the integration to $PROFILE.
# Usage: .\setup.ps1 [-InstallDir <folder>]
param(
    [string]$InstallDir = (Join-Path $env:LOCALAPPDATA 'Programs\aliases')
)

$ErrorActionPreference = 'Stop'
$projectRoot = Split-Path -Parent $PSCommandPath

Write-Host 'Aliases Manager Setup' -ForegroundColor Cyan
Write-Host '=============================================='

if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
    throw 'cargo not found. Install Rust from https://rustup.rs and run setup.ps1 again.'
}

# A separate target folder, so a WSL build of the same checkout does not clash with this one.
$targetDir = Join-Path $projectRoot 'target\windows'
cargo build --release --locked --manifest-path (Join-Path $projectRoot 'Cargo.toml') --target-dir $targetDir
if ($LASTEXITCODE -ne 0) { throw 'cargo build failed.' }

New-Item -ItemType Directory -Path $InstallDir -Force | Out-Null
$exe = Join-Path $InstallDir 'aliases.exe'
Copy-Item -Path (Join-Path $targetDir 'release\aliases.exe') -Destination $exe -Force
Write-Host "Installed $exe" -ForegroundColor Green

& $exe setup --shell pwsh --rc $PROFILE
if ($LASTEXITCODE -ne 0) { throw 'aliases setup failed.' }

$userPath = [Environment]::GetEnvironmentVariable('Path', 'User')
$entries = @($userPath -split ';' | Where-Object { $_ })
if ($entries -notcontains $InstallDir) {
    [Environment]::SetEnvironmentVariable('Path', (($entries + $InstallDir) -join ';'), 'User')
    Write-Host "Added $InstallDir to the user PATH (new sessions)." -ForegroundColor Green
}

Write-Host 'Setup complete! Run: . $PROFILE' -ForegroundColor Green
