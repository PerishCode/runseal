$ErrorActionPreference = 'Stop'

$command = if ($args.Length -gt 0) { $args[0] } else { 'install' }
$remaining = if ($args.Length -gt 1) { $args[1..($args.Length - 1)] } else { @() }

$channel = if ($env:RUNSEAL_CHANNEL) { $env:RUNSEAL_CHANNEL } else { 'stable' }
$version = if ($env:RUNSEAL_VERSION) { $env:RUNSEAL_VERSION } else { '' }
$publicUrl = if ($env:RUNSEAL_RELEASES_PUBLIC_URL) { $env:RUNSEAL_RELEASES_PUBLIC_URL } else { 'https://releases.runseal.perish.uk' }
$installRoot = if ($env:RUNSEAL_INSTALL_ROOT) { $env:RUNSEAL_INSTALL_ROOT } else { Join-Path $env:LOCALAPPDATA 'runseal' }
$localBinDir = if ($env:RUNSEAL_LOCAL_BIN_DIR) { $env:RUNSEAL_LOCAL_BIN_DIR } else { Join-Path $env:USERPROFILE '.local\bin' }

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
        '^(-h|--help|help)$' {
            @'
runseal installer

Usage:
  install.ps1
  install.ps1 install [--channel stable|beta] [--version vX.Y.Z] [--public-url <url>]
  install.ps1 upgrade [--channel stable|beta] [--version vX.Y.Z] [--public-url <url>]
  install.ps1 uninstall

Environment:
  RUNSEAL_RELEASES_PUBLIC_URL  # default: https://releases.runseal.perish.uk
  RUNSEAL_CHANNEL
  RUNSEAL_VERSION
  RUNSEAL_INSTALL_ROOT
  RUNSEAL_LOCAL_BIN_DIR
'@ | Write-Output
            exit 0
        }
        default { throw "unknown argument: $arg" }
    }
}

function Require-PublicUrl {
    return $publicUrl.TrimEnd('/')
}

function Install-Runseal {
    $resolvedPublicUrl = Require-PublicUrl
    $resolvedVersion = $version
    if ([string]::IsNullOrWhiteSpace($resolvedVersion)) {
        $metadataUrl = "$resolvedPublicUrl/$channel/latest/metadata.json"
        $metadata = Invoke-RestMethod -Uri $metadataUrl
        $resolvedVersion = $metadata.releaseVersion
        if ([string]::IsNullOrWhiteSpace($resolvedVersion)) {
            throw 'failed to resolve latest runseal version'
        }
    }

    $archive = 'runseal-x86_64-pc-windows-msvc.zip'
    $tmpdir = Join-Path ([System.IO.Path]::GetTempPath()) ("runseal-" + [System.Guid]::NewGuid().ToString('N'))
    New-Item -ItemType Directory -Path $tmpdir | Out-Null
    try {
        $archivePath = Join-Path $tmpdir $archive
        Invoke-WebRequest -Uri "$resolvedPublicUrl/$channel/versions/$resolvedVersion/$archive" -OutFile $archivePath
        $versionRoot = Join-Path $installRoot $resolvedVersion
        New-Item -ItemType Directory -Force -Path $versionRoot | Out-Null
        Expand-Archive -LiteralPath $archivePath -DestinationPath $versionRoot -Force
        New-Item -ItemType Directory -Force -Path $localBinDir | Out-Null
        Copy-Item -Force (Join-Path $versionRoot 'runseal.exe') (Join-Path $localBinDir 'runseal.exe')
        & (Join-Path $localBinDir 'runseal.exe') --version
        Write-Output "installed runseal to $(Join-Path $localBinDir 'runseal.exe')"
    }
    finally {
        Remove-Item -Recurse -Force -ErrorAction SilentlyContinue $tmpdir
    }
}

function Uninstall-Runseal {
    $binPath = Join-Path $localBinDir 'runseal.exe'
    if (![string]::IsNullOrWhiteSpace($version)) {
        $normalizedVersion = "v$($version.TrimStart('v'))"
        if ([System.IO.File]::Exists($binPath)) {
            try {
                $output = & $binPath --version
                if ($output -match 'v?([0-9]+\.[0-9]+\.[0-9]+(?:[-.][A-Za-z0-9]+)*)') {
                    $installedVersion = "v$($Matches[1].TrimStart('v'))"
                    if ($installedVersion -eq $normalizedVersion) {
                        Remove-Item -Force -ErrorAction SilentlyContinue $binPath
                        Write-Output "removed $binPath"
                    }
                }
            }
            catch {}
        }
        Remove-Item -Recurse -Force -ErrorAction SilentlyContinue (Join-Path $installRoot $version)
        if ($version -ne $normalizedVersion) {
            Remove-Item -Recurse -Force -ErrorAction SilentlyContinue (Join-Path $installRoot $normalizedVersion)
        }
        try { Remove-Item -Force -ErrorAction Stop $installRoot } catch [System.IO.IOException] {}
        Write-Output "removed runseal $version from $installRoot"
        return
    }

    Remove-Item -Force -ErrorAction SilentlyContinue $binPath
    Remove-Item -Recurse -Force -ErrorAction SilentlyContinue $installRoot
    try { Remove-Item -Force -ErrorAction Stop $localBinDir } catch [System.IO.IOException] {}
    Write-Output "removed runseal from $installRoot and $binPath"
}

switch ($command) {
    'install' { Install-Runseal }
    'upgrade' { Install-Runseal }
    'uninstall' { Uninstall-Runseal }
    default { throw "unknown command: $command" }
}
