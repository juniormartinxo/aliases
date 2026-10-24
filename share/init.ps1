# share/init.ps1: compatibility shim for profiles still holding the old
# `. ...\share\init.ps1` block. Run .\setup.ps1 to switch to the new integration
# (`Invoke-Expression (& aliases init pwsh | Out-String)`); this file will be removed
# in a future version.

$aliasesManagerBin = Get-Command aliases.exe -CommandType Application -ErrorAction SilentlyContinue |
    Select-Object -First 1 -ExpandProperty Source
if (-not $aliasesManagerBin) {
    $aliasesManagerBin = Join-Path $PSScriptRoot '..\target\windows\release\aliases.exe'
}

if (Test-Path -LiteralPath $aliasesManagerBin) {
    Invoke-Expression (& $aliasesManagerBin init pwsh | Out-String)
}
else {
    Write-Warning "aliases-manager: aliases.exe is not installed. Run $(Join-Path $PSScriptRoot '..\setup.ps1')"
}
Remove-Variable aliasesManagerBin
