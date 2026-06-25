$ErrorActionPreference = 'Stop'

$command = if ($args.Length -gt 0) { $args[0] } else { 'install' }
$remaining = if ($args.Length -gt 1) { $args[1..($args.Length - 1)] } else { @() }

$channel = if ($env:RUNSEAL_CHANNEL) { $env:RUNSEAL_CHANNEL } else { 'stable' }
$version = if ($env:RUNSEAL_VERSION) { $env:RUNSEAL_VERSION } else { '' }
$publicUrl = if ($env:RUNSEAL_RELEASES_PUBLIC_URL) { $env:RUNSEAL_RELEASES_PUBLIC_URL } else { 'https://releases.runseal.perish.uk' }
$defaultInstallBase = if ($env:LOCALAPPDATA) { $env:LOCALAPPDATA } elseif ($env:HOME) { Join-Path $env:HOME '.local/share' } else { '.' }
$defaultBinBase = if ($env:USERPROFILE) { $env:USERPROFILE } elseif ($env:HOME) { $env:HOME } else { '.' }
$defaultBinLeaf = if ($env:USERPROFILE) { '.local\bin' } else { '.local/bin' }
$installRoot = if ($env:RUNSEAL_INSTALL_ROOT) { $env:RUNSEAL_INSTALL_ROOT } else { Join-Path $defaultInstallBase 'runseal' }
$localBinDir = if ($env:RUNSEAL_LOCAL_BIN_DIR) { $env:RUNSEAL_LOCAL_BIN_DIR } else { Join-Path $defaultBinBase $defaultBinLeaf }
$retain = if ($env:RUNSEAL_RETAIN) { $env:RUNSEAL_RETAIN } else { '' }

for ($i = 0; $i -lt $remaining.Length; $i++) {
    $arg = $remaining[$i]
    switch -Regex ($arg) {
        '^--channel$' { $i++; $channel = $remaining[$i]; continue }
        '^--channel=(.+)$' { $channel = $Matches[1]; continue }
        '^--version$' { $i++; $version = $remaining[$i]; continue }
        '^--version=(.+)$' { $version = $Matches[1]; continue }
        '^--public-url$' { $i++; $publicUrl = $remaining[$i]; continue }
        '^--public-url=(.+)$' { $publicUrl = $Matches[1]; continue }
        '^--install-root$' { $i++; $installRoot = $remaining[$i]; continue }
        '^--install-root=(.+)$' { $installRoot = $Matches[1]; continue }
        '^--bin-dir$' { $i++; $localBinDir = $remaining[$i]; continue }
        '^--bin-dir=(.+)$' { $localBinDir = $Matches[1]; continue }
        '^--retain$' { $retain = 'true'; continue }
        '^--retain=(.+)$' { $retain = $Matches[1]; continue }
        '^(-h|--help|help)$' {
            @'
runseal manager

Usage:
  manage.ps1 install [--channel stable|beta] [--version vX.Y.Z] [--retain[=true|false]]
  manage.ps1 uninstall [--version vX.Y.Z]

Options:
  --public-url <url>     release metadata and artifact base URL
  --install-root <path>  versioned install root
  --bin-dir <path>       directory for the runseal executable

Environment:
  RUNSEAL_RELEASES_PUBLIC_URL  # default: https://releases.runseal.perish.uk
  RUNSEAL_CHANNEL
  RUNSEAL_VERSION
  RUNSEAL_INSTALL_ROOT
  RUNSEAL_LOCAL_BIN_DIR
  RUNSEAL_RETAIN
'@ | Write-Output
            exit 0
        }
        default { throw "unknown argument: $arg" }
    }
}

function Normalize-Version {
    param([string]$Value)
    return "v$($Value.TrimStart('v'))"
}

function Normalize-Bool {
    param([string]$Value)
    switch -Regex ($Value) {
        '^(true|1|yes|y|on)$' { return $true }
        '^(false|0|no|n|off)$' { return $false }
        default { throw "invalid --retain value: $Value" }
    }
}

function Installed-Versions {
    param([string]$Current)
    if (![System.IO.Directory]::Exists($installRoot)) {
        return @()
    }
    return @(Get-ChildItem -LiteralPath $installRoot -Directory | Where-Object { $_.Name -ne $Current } | ForEach-Object { $_.Name })
}

function Should-Retain {
    param([string[]]$OldVersions)
    if ($OldVersions.Length -eq 0) {
        return $true
    }
    if (![string]::IsNullOrWhiteSpace($retain)) {
        return Normalize-Bool $retain
    }
    if ([Environment]::UserInteractive -and -not [Console]::IsInputRedirected) {
        $answer = Read-Host 'runseal: remove previously installed versions after install? [y/N]'
        if ($answer -match '^(y|yes)$') {
            return $false
        }
        return $true
    }
    [Console]::Error.WriteLine('runseal: preserving previous versions; pass --retain=false to prune after install')
    return $true
}

function Install-Runseal {
    $resolvedPublicUrl = $publicUrl.TrimEnd('/')
    $resolvedVersion = $version
    if ([string]::IsNullOrWhiteSpace($resolvedVersion)) {
        $metadataUrl = "$resolvedPublicUrl/$channel/latest/metadata.json"
        $metadata = Invoke-RestMethod -Uri $metadataUrl
        $resolvedVersion = $metadata.releaseVersion
        if ([string]::IsNullOrWhiteSpace($resolvedVersion)) {
            throw 'failed to resolve latest runseal version'
        }
    }
    $resolvedVersion = Normalize-Version $resolvedVersion
    $oldVersions = Installed-Versions $resolvedVersion
    $retainOld = Should-Retain $oldVersions

    $archive = 'runseal-x86_64-pc-windows-msvc.zip'
    $tmpdir = Join-Path ([System.IO.Path]::GetTempPath()) ("runseal-" + [System.Guid]::NewGuid().ToString('N'))
    New-Item -ItemType Directory -Path $tmpdir | Out-Null
    try {
        $archivePath = Join-Path $tmpdir $archive
        Invoke-WebRequest -Uri "$resolvedPublicUrl/$channel/versions/$resolvedVersion/$archive" -OutFile $archivePath
        $versionRoot = Join-Path $installRoot $resolvedVersion
        Remove-Item -Recurse -Force -ErrorAction SilentlyContinue $versionRoot
        New-Item -ItemType Directory -Force -Path $versionRoot | Out-Null
        Expand-Archive -LiteralPath $archivePath -DestinationPath $versionRoot -Force
        New-Item -ItemType Directory -Force -Path $localBinDir | Out-Null
        Copy-Item -Force (Join-Path $versionRoot 'runseal.exe') (Join-Path $localBinDir 'runseal.exe')
        & (Join-Path $localBinDir 'runseal.exe') --version

        if (!$retainOld) {
            foreach ($oldVersion in $oldVersions) {
                Remove-Item -Recurse -Force -ErrorAction SilentlyContinue (Join-Path $installRoot $oldVersion)
                Write-Output "removed old runseal $oldVersion from $installRoot"
            }
        }

        Write-Output "installed runseal to $(Join-Path $localBinDir 'runseal.exe')"
    }
    finally {
        Remove-Item -Recurse -Force -ErrorAction SilentlyContinue $tmpdir
    }
}

function Remove-EmptyDir {
    param([string]$Path)
    if ([System.IO.Directory]::Exists($Path)) {
        try {
            Remove-Item -Force -ErrorAction Stop $Path
        }
        catch [System.IO.IOException] {}
    }
}

function Installed-Version {
    $binPath = Join-Path $localBinDir 'runseal.exe'
    if (![System.IO.File]::Exists($binPath)) {
        return ''
    }
    try {
        $output = & $binPath --version
        if ($output -match 'v?([0-9]+\.[0-9]+\.[0-9]+(?:[-.][A-Za-z0-9]+)*)') {
            return "v$($Matches[1].TrimStart('v'))"
        }
    }
    catch {}
    return ''
}

function Uninstall-Runseal {
    $binPath = Join-Path $localBinDir 'runseal.exe'
    if (![string]::IsNullOrWhiteSpace($version)) {
        $normalizedVersion = Normalize-Version $version
        if ((Installed-Version) -eq $normalizedVersion) {
            Remove-Item -Force -ErrorAction SilentlyContinue $binPath
            Write-Output "removed $binPath"
        }
        Remove-Item -Recurse -Force -ErrorAction SilentlyContinue (Join-Path $installRoot $normalizedVersion)
        Remove-EmptyDir $installRoot
        Write-Output "removed runseal $normalizedVersion from $installRoot"
        return
    }

    Remove-Item -Force -ErrorAction SilentlyContinue $binPath
    Remove-Item -Recurse -Force -ErrorAction SilentlyContinue $installRoot
    Remove-EmptyDir $localBinDir
    Write-Output "removed runseal from $installRoot and $binPath"
}

switch ($command) {
    { $_ -in @('-h', '--help', 'help') } {
        @'
runseal manager

Usage:
  manage.ps1 install [--channel stable|beta] [--version vX.Y.Z] [--retain[=true|false]]
  manage.ps1 uninstall [--version vX.Y.Z]

Options:
  --public-url <url>     release metadata and artifact base URL
  --install-root <path>  versioned install root
  --bin-dir <path>       directory for the runseal executable

Environment:
  RUNSEAL_RELEASES_PUBLIC_URL  # default: https://releases.runseal.perish.uk
  RUNSEAL_CHANNEL
  RUNSEAL_VERSION
  RUNSEAL_INSTALL_ROOT
  RUNSEAL_LOCAL_BIN_DIR
  RUNSEAL_RETAIN
'@ | Write-Output
    }
    'install' { Install-Runseal }
    'uninstall' { Uninstall-Runseal }
    default { throw "unknown command: $command" }
}
