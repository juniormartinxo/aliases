param(
    [switch]$RemoveConfig
)

$ErrorActionPreference = 'Stop'

$profilePath = $PROFILE
$startMarker = '# >>> aliases-manager >>>'
$endMarker = '# <<< aliases-manager <<<'
$escapedStart = [regex]::Escape($startMarker)
$escapedEnd = [regex]::Escape($endMarker)

$xdgConfigHome = [Environment]::GetEnvironmentVariable('XDG_CONFIG_HOME', 'User')
if ([string]::IsNullOrWhiteSpace($xdgConfigHome)) {
    $xdgConfigHome = Join-Path $HOME '.config'
}
$aliasesRoot = Join-Path $xdgConfigHome 'aliases'

Write-Host 'Aliases Manager Uninstall' -ForegroundColor Red
Write-Host '=============================================='

if (Test-Path -LiteralPath $profilePath) {
    $profileContent = Get-Content -Raw -LiteralPath $profilePath
    $cleanContent = [regex]::Replace($profileContent, "(?s)$escapedStart.*?$escapedEnd\r?\n?", '')
    $cleanContent = $cleanContent.TrimEnd("`r", "`n")

    if ($cleanContent -ne $profileContent.TrimEnd("`r", "`n")) {
        if ([string]::IsNullOrWhiteSpace($cleanContent)) {
            Set-Content -LiteralPath $profilePath -Value '' -Encoding UTF8
        }
        else {
            Set-Content -LiteralPath $profilePath -Value ($cleanContent + "`r`n") -Encoding UTF8
        }
        Write-Host "Removed aliases-manager block from $profilePath" -ForegroundColor Green
    }
    else {
        Write-Host "No aliases-manager block found in $profilePath" -ForegroundColor Yellow
    }
}
else {
    Write-Host "PowerShell profile not found: $profilePath" -ForegroundColor Yellow
}

if (-not $RemoveConfig) {
    $response = Read-Host "Remove the configuration directory ($aliasesRoot)? [y/N]"
    if ($response -match '^(?i:y|yes)$') {
        $RemoveConfig = $true
    }
}

if ($RemoveConfig) {
    if (Test-Path -LiteralPath $aliasesRoot) {
        Remove-Item -LiteralPath $aliasesRoot -Recurse -Force
        Write-Host "Removed $aliasesRoot" -ForegroundColor Green
    }
    else {
        Write-Host "Configuration directory not found: $aliasesRoot" -ForegroundColor Yellow
    }
}
else {
    Write-Host "Configuration directory kept at $aliasesRoot" -ForegroundColor Cyan
}

Write-Host 'Uninstallation complete.' -ForegroundColor Green
