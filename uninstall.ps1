# uninstall.ps1: removes the $PROFILE integration, aliases.exe and its PATH entry.
# Usage: .\uninstall.ps1 [-RemoveConfig] [-InstallDir <folder>]
param(
    [switch]$RemoveConfig,
    [string]$InstallDir = (Join-Path $env:LOCALAPPDATA 'Programs\aliases')
)

$ErrorActionPreference = 'Stop'
$exe = Join-Path $InstallDir 'aliases.exe'

Write-Host 'Aliases Manager Uninstall' -ForegroundColor Red
Write-Host '=============================================='

if (-not (Test-Path -LiteralPath $exe)) {
    throw "$exe not found. Pass -InstallDir with the folder setup.ps1 installed to."
}

$arguments = @('uninstall', '--shell', 'pwsh', '--rc', $PROFILE)
if ($RemoveConfig) { $arguments += '--remove-config' }
& $exe @arguments
if ($LASTEXITCODE -ne 0) { throw 'aliases uninstall failed.' }

Remove-Item -LiteralPath $InstallDir -Recurse -Force
Write-Host "Removed $InstallDir" -ForegroundColor Green

$userPath = [Environment]::GetEnvironmentVariable('Path', 'User')
$entries = @($userPath -split ';' | Where-Object { $_ -and $_ -ne $InstallDir })
[Environment]::SetEnvironmentVariable('Path', ($entries -join ';'), 'User')

Write-Host 'Uninstallation complete.' -ForegroundColor Green
