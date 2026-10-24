# End-to-end check of the PowerShell integration, in a throwaway config directory.
# Usage: pwsh -NoProfile -File tests/smoke.ps1 -Binary target/debug/aliases.exe
param([Parameter(Mandatory)][string]$Binary)

$ErrorActionPreference = 'Stop'
$Binary = (Resolve-Path $Binary).Path

$sandbox = Join-Path ([IO.Path]::GetTempPath()) ("aliases-smoke-" + [guid]::NewGuid())
New-Item -ItemType Directory -Path $sandbox | Out-Null
$env:ALIASES_ROOT = Join-Path $sandbox 'config'
$testProfile = Join-Path $sandbox 'profile.ps1'
$script:failures = 0

function Check([string]$Name, [scriptblock]$Condition) {
    if (& $Condition) {
        Write-Host "ok   $Name"
    }
    else {
        Write-Host "FAIL $Name" -ForegroundColor Red
        $script:failures++
    }
}

try {
    & $Binary setup --shell pwsh --rc $testProfile | Out-Null
    Check 'setup writes the profile block' { (Get-Content -Raw $testProfile) -match 'init pwsh' }

    Invoke-Expression (& $Binary init pwsh | Out-String)
    Check 'default alias is loaded' { Get-Command ll -CommandType Function -ErrorAction SilentlyContinue }

    alias-create hello "Write-Output 'olá mundo'" greet | Out-Null
    Check 'alias-create defines the function' { (hello) -eq 'olá mundo' }

    alias-create echoargs 'Write-Output' | Out-Null
    Check 'arguments are forwarded' { ((echoargs a 'b c') -join '|') -eq 'a|b c' }

    # Windows PowerShell turns redirected native stderr into errors, which Stop would throw.
    $ErrorActionPreference = 'Continue'
    alias-create hello 'x' greet 2>$null | Out-Null
    $ErrorActionPreference = 'Stop'
    Check 'duplicate alias is refused' { $LASTEXITCODE -eq 1 }

    alias-remove hello | Out-Null
    Check 'alias-remove drops the function' { -not (Get-Command hello -ErrorAction SilentlyContinue) }

    alias-disable 00-core | Out-Null
    Check 'alias-disable unloads the file' { -not (Get-Command ll -ErrorAction SilentlyContinue) }
    alias-enable 00-core | Out-Null
    Check 'alias-enable loads it again' { Get-Command ll -CommandType Function -ErrorAction SilentlyContinue }

    Check 'alias-find finds by command' { (alias-find write-output) -match 'echoargs' }
    Check 'alias-create -Help shows usage' { (alias-create -Help | Out-String) -match 'Usage' }
}
finally {
    Remove-Item Env:\ALIASES_ROOT -ErrorAction SilentlyContinue
    Remove-Item -Recurse -Force $sandbox -ErrorAction SilentlyContinue
}

if ($script:failures -gt 0) {
    throw "$script:failures check(s) failed"
}
Write-Host 'All PowerShell checks passed.'
