param()

$ErrorActionPreference = 'Stop'

$projectRoot = Split-Path -Parent $PSCommandPath
$initScript = Join-Path $projectRoot 'share\init.ps1'
$profilePath = $PROFILE

if (-not (Test-Path $initScript)) {
    throw "init.ps1 not found: $initScript"
}

$xdgConfigHome = [Environment]::GetEnvironmentVariable('XDG_CONFIG_HOME', 'User')
if ([string]::IsNullOrWhiteSpace($xdgConfigHome)) {
    $xdgConfigHome = Join-Path $HOME '.config'
}
$aliasesRoot = Join-Path $xdgConfigHome 'aliases'
$aliasesDir = Join-Path $aliasesRoot 'aliases.d'

New-Item -ItemType Directory -Path $aliasesDir -Force | Out-Null

if (-not (Get-ChildItem -Path $aliasesDir -File | Where-Object { $_.Name -notmatch '\.bak$' } | Select-Object -First 1)) {
    "# Core aliases`nalias ll='Get-ChildItem -Force'" | Set-Content -Path (Join-Path $aliasesDir '00-core.alias') -Encoding UTF8
    Write-Host '✅ Created default alias file: 00-core.alias' -ForegroundColor Green
}

if (-not (Test-Path $profilePath)) {
    $profileDir = Split-Path -Parent $profilePath
    if (-not (Test-Path $profileDir)) {
        New-Item -ItemType Directory -Path $profileDir -Force | Out-Null
    }
    New-Item -ItemType File -Path $profilePath -Force | Out-Null
}

$profileContent = Get-Content -Raw -Path $profilePath
$startMarker = '# >>> aliases-manager >>>'
$endMarker = '# <<< aliases-manager <<<'
$escapedStart = [regex]::Escape($startMarker)
$escapedEnd = [regex]::Escape($endMarker)

if ($profileContent -match $escapedStart) {
    $profileContent = [regex]::Replace($profileContent, "(?s)$escapedStart.*?$escapedEnd\r?\n?", '')
}

$quotedInit = $initScript -replace "'", "''"
$block = @"
$startMarker
. '$quotedInit'
$endMarker
"@

if (-not [string]::IsNullOrEmpty($profileContent) -and -not $profileContent.EndsWith("`n")) {
    $profileContent += "`r`n"
}

Set-Content -Path $profilePath -Value ($profileContent + $block) -Encoding UTF8

Write-Host "✅ Added PowerShell integration block to profile: $profilePath" -ForegroundColor Green

. $initScript
Write-Host '✅ Setup complete! Run: . $PROFILE or restart shell to apply.' -ForegroundColor Green
