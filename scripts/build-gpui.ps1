param([switch]$Package, [switch]$Launch)
$ErrorActionPreference = 'Stop'
$repo = Split-Path $PSScriptRoot -Parent
$previousTarget = $env:CARGO_TARGET_DIR
$env:CARGO_TARGET_DIR = Join-Path $repo 'target/gpui-candidate'
Push-Location $repo
try {
    & powershell -ExecutionPolicy Bypass -File scripts\verify-gpui-stage0.ps1
    if ($LASTEXITCODE -ne 0) { throw 'GPUI 检查或构建失败' }
    $target = $env:CARGO_TARGET_DIR
    if ($Package) {
        $metadataText = & cargo metadata --locked --format-version 1 --filter-platform x86_64-pc-windows-msvc
        if ($LASTEXITCODE -ne 0) { throw '依赖清单读取失败' }
        $metadata = $metadataText | ConvertFrom-Json
        $nodes = @{}; $packages = @{}
        foreach ($node in $metadata.resolve.nodes) { $nodes[$node.id] = $node }
        foreach ($item in $metadata.packages) { $packages[$item.id] = $item }
        $root = $metadata.packages | Where-Object name -eq 'quickpick-gpui'
        $pending = [System.Collections.Generic.Stack[string]]::new()
        $seen = [System.Collections.Generic.HashSet[string]]::new()
        $pending.Push($root.id)
        while ($pending.Count -gt 0) {
            $id = $pending.Pop()
            if ($seen.Add($id)) { foreach ($dep in $nodes[$id].deps) { $pending.Push($dep.pkg) } }
        }
        $stage = Join-Path $target 'package'
        # 使用唯一空目录，防止旧产物或用户内容混入 ZIP。
        $stage = Join-Path $stage ([Guid]::NewGuid().ToString('N'))
        New-Item -ItemType Directory -Path $stage | Out-Null
        Copy-Item (Join-Path $target 'release/quickpick.exe') $stage
        Copy-Item LICENSE $stage
        Copy-Item apps/quickpick-gpui/third-party/sources.json (Join-Path $stage 'THIRD-PARTY-SOURCES.json')
        $notices = @(); $missing = @()
        foreach ($id in $seen) {
            $item = $packages[$id]
            if ($item.name -like 'quickpick-*') { continue }
            $directory = Split-Path $item.manifest_path -Parent
            $licenseFiles = @(Get-ChildItem -LiteralPath $directory -File | Where-Object { $_.Name -match '^(LICENSE|LICENCE|COPYING|COPYRIGHT|NOTICE|UNLICENSE)' })
            if ($item.license_file) {
                $specified = if ([IO.Path]::IsPathRooted($item.license_file)) { $item.license_file } else { Join-Path $directory $item.license_file }
                if (Test-Path -LiteralPath $specified) { $licenseFiles += Get-Item -LiteralPath $specified }
            }
            $supplement = Join-Path $repo "apps/quickpick-gpui/third-party/$($item.name)-$($item.version)"
            if (Test-Path -LiteralPath $supplement) { $licenseFiles += Get-ChildItem -LiteralPath $supplement -File }
            $licenseFiles = @($licenseFiles | Sort-Object FullName -Unique)
            if ($licenseFiles.Count -eq 0) { $missing += "$($item.name) $($item.version)" }
            $destination = Join-Path $stage "third-party-licenses/$($item.name)-$($item.version)"
            New-Item -ItemType Directory -Force -Path $destination | Out-Null
            foreach ($file in $licenseFiles) { Copy-Item -LiteralPath $file.FullName -Destination $destination }
            $notices += [pscustomobject]@{ name=$item.name; version=$item.version; license=$item.license; authors=$item.authors; repository=$item.repository; files=@($licenseFiles.Name) }
        }
        $notices | Sort-Object name,version | ConvertTo-Json -Depth 5 | Set-Content -Encoding utf8 (Join-Path $stage 'THIRD-PARTY-NOTICES.json')
        if ($missing.Count -gt 0) {
            $missing | Set-Content -Encoding utf8 (Join-Path $target 'missing-license-files.txt')
            throw "部分依赖缺少许可正文，请补齐来源声明后打包：$($missing -join ', ')"
        }
        $artifacts = Join-Path $target 'artifacts'
        New-Item -ItemType Directory -Force -Path $artifacts | Out-Null
        $version = $root.version
        $zip = Join-Path $artifacts "QuickPick_${version}_gpui_candidate_x64.zip"
        # ZIP 的 DOS 时间范围为 1980..2107；仅修正暂存副本，不修改依赖原文件。
        $zipMinimum = [DateTime]::new(1980, 1, 1, 0, 0, 0)
        $zipMaximum = [DateTime]::new(2107, 12, 31, 23, 59, 58)
        Get-ChildItem -LiteralPath $stage -Recurse -Force | ForEach-Object {
            if ($_.LastWriteTime -lt $zipMinimum) { $_.LastWriteTime = $zipMinimum }
            elseif ($_.LastWriteTime -gt $zipMaximum) { $_.LastWriteTime = $zipMaximum }
        }
        Compress-Archive -Path (Join-Path $stage '*') -DestinationPath $zip -Force
        $compiler = Get-Command makensis.exe -ErrorAction SilentlyContinue
        if (-not $compiler) {
            $nsisPath = Join-Path ${env:ProgramFiles(x86)} 'NSIS/makensis.exe'
            $compiler = Get-Command $nsisPath -ErrorAction Stop
        }
        $installer = Join-Path $artifacts "QuickPick_${version}_gpui_candidate_x64_setup.exe"
        Push-Location apps/quickpick-gpui/installer
        try {
            & $compiler.Source /INPUTCHARSET UTF8 "/DVERSION=$version" "/DSOURCE_DIR=$stage" "/DOUTPUT_FILE=$installer" installer.nsi
            if ($LASTEXITCODE -ne 0) { throw 'NSIS 构建失败' }
        } finally { Pop-Location }
        Get-FileHash -Algorithm SHA256 $zip,$installer | Select-Object Path,Hash | ConvertTo-Json | Set-Content -Encoding utf8 (Join-Path $artifacts 'SHA256.json')
        Write-Host "候选包已生成：$artifacts；安装/升级/卸载和无 WebView2 环境验收仍需在 Windows 完成。"
    }
    if ($Launch) { & (Join-Path $target 'release/quickpick.exe') --candidate }
} finally {
    $env:CARGO_TARGET_DIR = $previousTarget
    Pop-Location
}
