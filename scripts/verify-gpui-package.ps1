param(
    [string]$ArtifactsDirectory = (Join-Path (Split-Path $PSScriptRoot -Parent) 'target/gpui-candidate/artifacts'),
    [string]$Executable,
    [string]$Output
)
$ErrorActionPreference = 'Stop'
Add-Type -AssemblyName System.IO.Compression
Add-Type -AssemblyName System.IO.Compression.FileSystem
$directory = (Resolve-Path -LiteralPath $ArtifactsDirectory).Path
$parsedHashes = Get-Content -LiteralPath (Join-Path $directory 'SHA256.json') -Raw -Encoding UTF8 | ConvertFrom-Json
$hashes = @($parsedHashes)
if ($hashes.Count -ne 2) { throw 'Expected exactly one ZIP and one NSIS checksum.' }
$names = @{}; $verified = @()
foreach ($item in $hashes) {
    $name = [regex]::Split([string]$item.Path, '[\\/]')[-1]
    if ($name -notmatch '^QuickPick_([0-9]+\.[0-9]+\.[0-9]+)_gpui_candidate_x64(\.zip|_setup\.exe)$' -or $names.ContainsKey($name)) { throw 'Unexpected or duplicate artifact name.' }
    $version = $Matches[1]
    if ([string]$item.Hash -notmatch '^[a-fA-F0-9]{64}$') { throw 'Invalid SHA-256 value.' }
    $file = Join-Path $directory $name
    $hash = (Get-FileHash -LiteralPath $file -Algorithm SHA256).Hash
    if ($hash -ne $item.Hash) { throw "Artifact checksum mismatch: $name" }
    $names[$name] = $version
    $verified += [pscustomobject]@{ name=$name; bytes=(Get-Item -LiteralPath $file).Length; sha256=$hash }
}
$zipName = @($names.Keys | Where-Object { $_.EndsWith('.zip') })
$setupName = @($names.Keys | Where-Object { $_.EndsWith('_setup.exe') })
if ($zipName.Count -ne 1 -or $setupName.Count -ne 1 -or $names[$zipName[0]] -ne $names[$setupName[0]]) { throw 'ZIP and NSIS versions do not match.' }
$archive = [IO.Compression.ZipFile]::OpenRead((Join-Path $directory $zipName[0]))
try {
    $entries = @{}; $files = @{}; $total = 0L
    foreach ($entry in $archive.Entries) {
        $path = $entry.FullName.Replace('\','/')
        if ($path -match '(^/|[\x00-\x1f:<>"|?*])' -or @($path.TrimEnd('/').Split('/') | Where-Object { $_ -in @('', '.', '..') -or $_ -match '[. ]$' }).Count) { throw 'Invalid ZIP member path.' }
        if ($entries.ContainsKey($path)) { throw 'Case-insensitive duplicate ZIP member.' }
        $entries[$path] = $true
        if ($path.EndsWith('/')) { continue }
        if ($path -notin @('quickpick.exe','LICENSE','THIRD-PARTY-NOTICES.json','THIRD-PARTY-SOURCES.json') -and -not $path.StartsWith('third-party-licenses/', [StringComparison]::Ordinal)) { throw 'Unexpected runtime payload in ZIP.' }
        if ($entry.Length -gt 256MB) { throw 'ZIP member exceeds validation size limit.' }
        $total += $entry.Length
        if ($total -gt 1GB) { throw 'ZIP exceeds validation size limit.' }
        $files[$path] = $entry
    }
    foreach ($required in @('quickpick.exe','LICENSE','THIRD-PARTY-NOTICES.json','THIRD-PARTY-SOURCES.json')) {
        if (-not $files.ContainsKey($required) -or $files[$required].Length -eq 0) { throw 'Missing required payload.' }
    }
    function Read-PackageJson([string]$Name) {
        if ($files[$Name].Length -gt 8MB) { throw 'JSON payload exceeds validation limit.' }
        $reader = [IO.StreamReader]::new($files[$Name].Open(), [Text.Encoding]::UTF8, $true)
        try {
            $parsed = $reader.ReadToEnd() | ConvertFrom-Json
            foreach ($item in $parsed) { Write-Output $item }
        } finally { $reader.Dispose() }
    }
    $notices = @(Read-PackageJson 'THIRD-PARTY-NOTICES.json')
    $null = Read-PackageJson 'THIRD-PARTY-SOURCES.json'
    if ($notices.Count -eq 0) { throw 'Empty third-party notice list.' }
    $packages = @{}
    foreach ($notice in $notices) {
        $package = "$($notice.name)-$($notice.version)"
        if ($package -notmatch '^[a-zA-Z0-9_.+-]+$' -or $packages.ContainsKey($package) -or -not $notice.license -or @($notice.files).Count -eq 0) { throw 'Invalid third-party notice.' }
        $packages[$package] = $true
        foreach ($license in $notice.files) {
            if ($license -notmatch '^[^\\/:]+$' -or -not $files.ContainsKey("third-party-licenses/$package/$license") -or $files["third-party-licenses/$package/$license"].Length -eq 0) { throw 'Missing third-party license material.' }
        }
    }
    $stream = $files['quickpick.exe'].Open()
    $memory = [IO.MemoryStream]::new()
    try { $stream.CopyTo($memory); $pe = $memory.ToArray() } finally { $stream.Dispose(); $memory.Dispose() }
    if ($pe.Length -lt 64 -or $pe[0] -ne 0x4d -or $pe[1] -ne 0x5a) { throw 'Invalid executable DOS header.' }
    $offset = [BitConverter]::ToInt32($pe, 0x3c)
    if ($offset -lt 64 -or $offset -gt $pe.Length - 96 -or [BitConverter]::ToUInt32($pe, $offset) -ne 0x4550 -or [BitConverter]::ToUInt16($pe, $offset + 4) -ne 0x8664 -or [BitConverter]::ToUInt16($pe, $offset + 24) -ne 0x20b -or [BitConverter]::ToUInt16($pe, $offset + 92) -ne 2) { throw 'Expected an x64 PE32+ Windows GUI executable.' }
    $sha = [Security.Cryptography.SHA256]::Create()
    try { $executableHash = [BitConverter]::ToString($sha.ComputeHash($pe)).Replace('-','') } finally { $sha.Dispose() }
    if ($Executable -and (Get-FileHash -LiteralPath $Executable -Algorithm SHA256).Hash -ne $executableHash) { throw 'ZIP executable differs from the specified release executable.' }
    $report = [pscustomobject]@{
        schema=1; status='static-checks-passed'; version=$names[$zipName[0]]; artifacts=$verified
        executable_sha256=$executableHash; executable_bytes=$pe.Length
        zip_file_count=$files.Count; zip_uncompressed_bytes=$total; third_party_packages=$packages.Count
        limitations=@('Checksums verify integrity against the supplied manifest, not publisher authenticity.',
            'NSIS is checked by hash only; its embedded payload and install, upgrade, rollback and uninstall are not executed.',
            'License-file presence does not replace license review. No desktop or clean-machine acceptance is implied.')
    }
} finally { $archive.Dispose() }
if (-not $Output) { $Output = Join-Path $directory 'package-validation.json' }
$destination = [IO.Path]::GetFullPath($Output)
New-Item -ItemType Directory -Force -Path (Split-Path $destination -Parent) | Out-Null
if (Test-Path -LiteralPath $destination) { throw 'Report already exists; use a new output path.' }
$report | ConvertTo-Json -Depth 6 | Set-Content -LiteralPath $destination -Encoding UTF8
Write-Host "Package integrity, x64 GUI header and license-file checks passed: $destination"
