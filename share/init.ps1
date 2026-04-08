Set-StrictMode -Version Latest

$xdgConfigHome = [Environment]::GetEnvironmentVariable('XDG_CONFIG_HOME', 'User')
if ([string]::IsNullOrWhiteSpace($xdgConfigHome)) {
    $xdgConfigHome = Join-Path $HOME '.config'
}

$global:ALIASES_ROOT = Join-Path $xdgConfigHome 'aliases'
$global:ALIASES_DIR = Join-Path $global:ALIASES_ROOT 'aliases.d'
$global:ALIASES_PROJECT_ROOT = (Resolve-Path (Join-Path $PSScriptRoot '..')).Path
$global:ALIASES_LOADED = [System.Collections.Generic.List[string]]::new()

function global:_aliases_manager_quote_argument {
    param([Parameter(Mandatory)][AllowEmptyString()][string]$Value)

    return "'" + $Value.Replace("'", "''") + "'"
}

function global:_aliases_manager_invoke_alias_command {
    param(
        [Parameter(Mandatory)][string]$Command,
        [object[]]$Arguments = @()
    )

    $argumentList = @($Arguments)
    if ($argumentList.Count -eq 1) {
        $firstArgument = $argumentList[0]
        if ($null -eq $firstArgument) {
            $argumentList = @()
        }
        elseif ($firstArgument -is [string] -and $firstArgument.Length -eq 0) {
            $argumentList = @()
        }
    }
    $invocation = $Command

    if ($argumentList.Count -gt 0) {
        $quotedArguments = @($argumentList | ForEach-Object {
                _aliases_manager_quote_argument -Value ([string]$_)
            })
        $invocation = "$Command $($quotedArguments -join ' ')"
    }

    if ($invocation -match '^(?<path>[A-Za-z]:\\.*?\.(?:exe|cmd|bat|com|ps1|psm1))(?<rest>(?:\s+.*)?)$') {
        $escapedPath = $matches['path'].Replace("'", "''")
        $invocation = "& '$escapedPath'$($matches['rest'])"
    }

    Invoke-Expression $invocation
}

function global:_aliases_manager_find_alias_files {
    param([Parameter(Mandatory)][string]$Name)

    if (-not (Test-Path $global:ALIASES_DIR)) {
        return @()
    }

    $pattern = "^\s*alias\s+$([regex]::Escape($Name))\s*="
    return @(Get-ChildItem -Path $global:ALIASES_DIR -Filter '*.alias' -File |
        Where-Object { Select-String -Path $_.FullName -Pattern $pattern -Quiet })
}

function global:_aliases_manager_define_alias {
    param(
        [Parameter(Mandatory)][string]$Name,
        [Parameter(Mandatory)][string]$Command
    )

    $functionPath = "Function:\global:$Name"

    if (Test-Path $functionPath) {
        Remove-Item $functionPath -Force
    }

    $escapedCommand = $Command.Replace("'", "''")
    $scriptText = @"
param([Parameter(ValueFromRemainingArguments=`$true)][object[]]`$AliasesArgs)
_aliases_manager_invoke_alias_command -Command '$escapedCommand' -Arguments `$AliasesArgs
"@

    Set-Item -Path $functionPath -Value ([scriptblock]::Create($scriptText)) -Force | Out-Null
    if (-not ($global:ALIASES_LOADED -contains $Name)) {
        $global:ALIASES_LOADED.Add($Name)
    }
}

function global:_aliases_manager_load_alias_file {
    param([Parameter(Mandatory)][string]$Path)

    foreach ($line in Get-Content -Path $Path -ErrorAction SilentlyContinue) {
        $line = $line.Trim()
        if ([string]::IsNullOrWhiteSpace($line) -or $line.StartsWith('#')) {
            continue
        }

        if ($line -match '^\s*alias\s+([A-Za-z_][A-Za-z0-9_\-]*)\s*=\s*''(.+)''\s*$') {
            $name = $matches[1]
            $command = $matches[2] -replace "'\\''", "'"
            _aliases_manager_define_alias -Name $name -Command $command
            continue
        }

        if ($line -match '^\s*alias\s+([A-Za-z_][A-Za-z0-9_\-]*)\s*=\s*"(.+)"\s*$') {
            $name = $matches[1]
            $command = $matches[2]
            _aliases_manager_define_alias -Name $name -Command $command
            continue
        }

        if ($line -match '^\s*alias\s+([A-Za-z_][A-Za-z0-9_\-]*)\s*=\s*(.+)\s*$') {
            $name = $matches[1]
            $command = $matches[2]
            _aliases_manager_define_alias -Name $name -Command $command
        }
    }
}

function global:alias-reload {
    foreach ($name in $global:ALIASES_LOADED) {
        $functionPath = "Function:\global:$name"
        if (Test-Path $functionPath) {
            Remove-Item $functionPath -Force
        }
    }
    $global:ALIASES_LOADED = [System.Collections.Generic.List[string]]::new()

    if (-not (Test-Path $global:ALIASES_DIR)) {
        Write-Host '⚠️  Aliases directory not found.' -ForegroundColor Yellow
        return
    }

    $files = Get-ChildItem -Path $global:ALIASES_DIR -Filter '*.alias' -File | Sort-Object Name

    foreach ($file in ($files | Where-Object { $_.Name -ne 'local.alias' })) {
        _aliases_manager_load_alias_file -Path $file.FullName
    }
    foreach ($file in ($files | Where-Object { $_.Name -eq 'local.alias' })) {
        _aliases_manager_load_alias_file -Path $file.FullName
    }

    Write-Host '✅ Aliases loaded.' -ForegroundColor Green
}

function global:alias-create {
    param(
        [Parameter(Position = 0)] [string]$Name,
        [Parameter(Position = 1)] [string]$Command,
        [Parameter(Position = 2)] [string]$File = 'local',
        [Alias('h')] [switch]$Help
    )

    if ($Help -or [string]::IsNullOrWhiteSpace($Name) -or [string]::IsNullOrWhiteSpace($Command)) {
        @'
Usage: alias-create <name> <command> [file]

Examples:
  alias-create gst "git status -sb" git
  alias-create pfle "& 'C:\Program Files\Notepad++\notepad++.exe' '$PROFILE'" system
  alias-create syncdocs "& 'C:\tools\sync-docs.ps1'" scripts
'@ | Write-Host

        if ($Help) {
            return
        }

        throw 'alias-create requires <name> and <command>.'
    }

    if ($File -notmatch '\.alias$') { $File = "$File.alias" }

    if (-not (Test-Path $global:ALIASES_DIR)) {
        New-Item -ItemType Directory -Path $global:ALIASES_DIR -Force | Out-Null
    }

    $target = Join-Path $global:ALIASES_DIR $File
    if (Test-Path $target) {
        if (Select-String -Path $target -Pattern "^\s*alias\s+$([regex]::Escape($Name))\s*=" -Quiet) {
            throw "Alias '$Name' already exists in $File"
        }
    }
    else {
        "# $File" | Set-Content -Path $target -Encoding UTF8
    }

    $safe = $Command.Replace("'", "'\\''")
    Add-Content -Path $target -Value "alias $Name='$safe'" -Encoding UTF8
    Write-Host "✅ Alias '$Name' added to $target" -ForegroundColor Green
    alias-reload
}

function global:alias-remove {
    param(
        [Parameter(Mandatory, Position = 0)] [string]$Name,
        [Parameter(Position = 1)] [string]$File
    )

    if (-not (Test-Path $global:ALIASES_DIR)) {
        throw 'Aliases directory not found.'
    }

    if ($File) {
        if ($File -notmatch '\.alias$') { $File = "$File.alias" }
        $target = Join-Path $global:ALIASES_DIR $File
        if (-not (Test-Path $target)) {
            throw "File $File does not exist."
        }
    }
    else {
        $found = _aliases_manager_find_alias_files -Name $Name

        if ($found.Count -eq 0) { throw "Alias '$Name' not found." }
        if ($found.Count -gt 1) { throw "Alias '$Name' exists in multiple files. Specify file name." }
        $target = $found[0].FullName
    }

    $pattern = "^alias\s+$([regex]::Escape($Name))="
    $content = Get-Content -Path $target
    $filtered = @($content | Where-Object { $_ -notmatch $pattern })
    if ($filtered.Count -eq $content.Count) {
        throw "Failed to remove alias '$Name' (pattern mismatch)."
    }

    Set-Content -Path $target -Value $filtered -Encoding UTF8
    Write-Host "✅ Removed '$Name' from $(Split-Path $target -Leaf)" -ForegroundColor Green
    alias-reload
}

function global:alias-edit {
    param([Parameter(Mandatory, Position = 0)] [string]$Name)

    if (-not (Test-Path $global:ALIASES_DIR)) {
        throw 'Aliases directory not found.'
    }

    $target = $null

    foreach ($candidate in @($Name, "$Name.alias")) {
        $candidatePath = Join-Path $global:ALIASES_DIR $candidate
        if (Test-Path $candidatePath) {
            $target = $candidatePath
            break
        }
    }

    if (-not $target) {
        $found = _aliases_manager_find_alias_files -Name $Name

        if ($found.Count -eq 1) {
            $target = $found[0].FullName
        }
        elseif ($found.Count -gt 1) {
            throw "Alias '$Name' exists in multiple files. Specify file name instead."
        }
    }

    if (-not $target) {
        throw "Alias file or alias '$Name' not found."
    }

    $editor = if ($env:EDITOR) { $env:EDITOR } else { 'notepad' }
    & $editor $target
    alias-reload
}

function global:alias-list {
    if (-not (Test-Path $global:ALIASES_DIR)) {
        throw 'Aliases directory not found at aliases root.'
    }

    Write-Host 'ALIAS                         SOURCE FILE           COMMAND' -ForegroundColor Cyan
    Write-Host '======================================================'

    $files = Get-ChildItem -Path $global:ALIASES_DIR -Filter '*.alias' -File | Sort-Object Name
    foreach ($file in $files) {
        foreach ($line in Get-Content $file.FullName) {
            if ($line -match '^\s*alias\s+([A-Za-z_][A-Za-z0-9_\-]*)\s*=\s*''(.+)''\s*$') {
                '{0,-28} {1,-20} {2}' -f $matches[1], $file.Name, ($matches[2] -replace "'\\''", "'")
            }
            elseif ($line -match '^\s*alias\s+([A-Za-z_][A-Za-z0-9_\-]*)\s*=\s*"(.+)"\s*$') {
                '{0,-28} {1,-20} {2}' -f $matches[1], $file.Name, $matches[2]
            }
            elseif ($line -match '^\s*alias\s+([A-Za-z_][A-Za-z0-9_\-]*)\s*=\s*(.+)\s*$') {
                '{0,-28} {1,-20} {2}' -f $matches[1], $file.Name, $matches[2]
            }
        }
    }
}

function global:alias-find {
    param([Parameter(Mandatory, Position = 0)] [string]$Query)

    if (-not (Test-Path $global:ALIASES_DIR)) {
        throw 'Aliases directory not found.'
    }

    Select-String -Path (Join-Path $global:ALIASES_DIR '*.alias') -Pattern $Query | ForEach-Object {
        Write-Host ("{0}: {1}" -f $_.Path, $_.Line)
    }
}

function global:alias-enable {
    param([Parameter(Mandatory)][string]$Name)

    $target = Join-Path $global:ALIASES_DIR "$Name.alias.disabled"
    if (-not (Test-Path $target)) {
        $alt = Join-Path $global:ALIASES_DIR "$Name.disabled"
        if (Test-Path $alt) { $target = $alt } else { throw "File $Name.alias.disabled not found." }
    }

    $item = Get-Item $target
    $newName = ($item.Name -replace '\.disabled$', '')
    Rename-Item -Path $target -NewName $newName | Out-Null
    Write-Host "✅ Enabled $Name (renamed to $newName)" -ForegroundColor Green
    alias-reload
}

function global:alias-disable {
    param([Parameter(Mandatory)][string]$Name)

    $target = Join-Path $global:ALIASES_DIR "$Name.alias"
    if (-not (Test-Path $target)) {
        $alt = Join-Path $global:ALIASES_DIR $Name
        if (Test-Path $alt) { $target = $alt } else { throw "File $Name.alias not found." }
    }

    $item = Get-Item $target
    Rename-Item -Path $target -NewName ("$($item.Name).disabled") | Out-Null
    Write-Host "✅ Disabled $Name (renamed to $($item.Name).disabled)" -ForegroundColor Green
    alias-reload
}

function global:alias-doctor {
    Write-Host 'Aliases Manager Doctor' -ForegroundColor Cyan
    Write-Host '=============================================='

    if ((Test-Path (Join-Path $global:ALIASES_PROJECT_ROOT 'bin')) -and
        (Test-Path (Join-Path $global:ALIASES_PROJECT_ROOT 'lib')) -and
        (Test-Path (Join-Path $global:ALIASES_PROJECT_ROOT 'share'))) {
        Write-Host '✅ Project structure OK (bin, lib, share)' -ForegroundColor Green
    } else {
        Write-Host '❌ Project structure is incomplete.' -ForegroundColor Red
    }

    if (Test-Path $global:ALIASES_DIR) {
        Write-Host "✅ Config directory OK: $global:ALIASES_DIR" -ForegroundColor Green
    } else {
        Write-Host "❌ Config directory missing: $global:ALIASES_DIR" -ForegroundColor Red
    }

    if (Test-Path $PROFILE -and (Select-String -Path $PROFILE -SimpleMatch '# >>> aliases-manager >>>' -Quiet)) {
        Write-Host '✅ Profile integration found.' -ForegroundColor Green
    } else {
        Write-Host '⚠️  Profile integration not found.' -ForegroundColor Yellow
    }
}

alias-reload
