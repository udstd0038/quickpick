param([switch]$Launch, [switch]$FullApplication)
$ErrorActionPreference = 'Stop'
if (-not [System.Runtime.InteropServices.RuntimeInformation]::IsOSPlatform([System.Runtime.InteropServices.OSPlatform]::Windows)) {
    throw '此脚本需要 Windows 11 x64 和 MSVC/Windows SDK。'
}
$repo = Split-Path $PSScriptRoot -Parent
$previousTarget = $env:CARGO_TARGET_DIR
$env:CARGO_TARGET_DIR = Join-Path $repo 'target/gpui-candidate'
Push-Location $repo
try {
    function Invoke-Cargo {
        param([string[]]$CargoArguments)
        & cargo @CargoArguments
        if ($LASTEXITCODE -ne 0) { throw "cargo $CargoArguments 失败，退出码 $LASTEXITCODE" }
    }
    Invoke-Cargo -CargoArguments @('fmt', '--all', '--check')
    Invoke-Cargo -CargoArguments @('clippy', '--workspace', '--all-targets', '--locked', '--', '-D', 'warnings')
    Invoke-Cargo -CargoArguments @('test', '--workspace', '--locked')
    Invoke-Cargo -CargoArguments @('build', '-p', 'quickpick-gpui', '--release', '--locked')

    $metadataText = & cargo metadata --locked --format-version 1 --filter-platform x86_64-pc-windows-msvc
    if ($LASTEXITCODE -ne 0) { throw '读取 Windows 目标依赖失败' }
    $metadata = $metadataText | ConvertFrom-Json
    $reportDir = Join-Path $metadata.target_directory 'gpui-stage0-reports'
    New-Item -ItemType Directory -Force $reportDir | Out-Null
    # metadata.packages 也包含其他目标包；必须按候选应用的已解析依赖闭包筛选。
    $nodes = @{}
    $packages = @{}
    foreach ($node in $metadata.resolve.nodes) { $nodes[$node.id] = $node }
    foreach ($package in $metadata.packages) { $packages[$package.id] = $package }
    $root = $metadata.packages | Where-Object name -eq 'quickpick-gpui'
    $pending = [System.Collections.Generic.Stack[string]]::new()
    $pending.Push($root.id)
    $seen = [System.Collections.Generic.HashSet[string]]::new()
    while ($pending.Count -gt 0) {
        $id = $pending.Pop()
        if ($seen.Add($id)) {
            foreach ($dep in $nodes[$id].deps) { $pending.Push($dep.pkg) }
        }
    }
    $dependencies = @($seen | ForEach-Object { $packages[$_] })
    $blocked = @($dependencies | Where-Object { $_.name -match '^(tauri($|-)|wry$|gpui-fast($|-)|gpui-webview$|gpui-shell$|syntect$|tree-sitter($|-))' })
    if ($blocked.Count -gt 0) { throw "候选依赖包含禁止项：$($blocked.name -join ', ')" }
    $backends = @($dependencies | Where-Object name -eq 'gpui-pre')
    if ($backends.Count -ne 1 -or $backends[0].version -ne '0.3.12') { throw 'GPUI 后端版本不唯一或与 Kit 不匹配' }
    $dependencies | Sort-Object name,version | Select-Object name,version,license,repository |
        ConvertTo-Json -Depth 4 | Set-Content -Encoding utf8 (Join-Path $reportDir 'windows-dependencies.json')
    $tree = & cargo tree -p quickpick-gpui --target x86_64-pc-windows-msvc --locked --edges normal,build
    if ($LASTEXITCODE -ne 0) { throw '生成依赖树失败' }
    $tree | Set-Content -Encoding utf8 (Join-Path $reportDir 'windows-dependency-tree.txt')
    Write-Host "自动化检查完成；依赖许可清单待人工审核：$reportDir"
    Write-Host '透明合成、IME、材质、DPI、路径/密钥、托盘/热键和性能基线仍须逐项验收。'
    if ($Launch) { if ($FullApplication) { & (Join-Path $metadata.target_directory 'release/quickpick.exe') --candidate } else { & (Join-Path $metadata.target_directory 'release/quickpick.exe') --feasibility } }
} finally {
    $env:CARGO_TARGET_DIR = $previousTarget
    Pop-Location
}
