$ErrorActionPreference = 'Stop'
$scripts = Split-Path $PSScriptRoot -Parent
. (Join-Path $scripts 'gpui-validation-common.ps1')
Add-Type -AssemblyName System.IO.Compression
Add-Type -AssemblyName System.IO.Compression.FileSystem
$checks = 0
function Require($Condition, [string]$Message) { if (-not $Condition) { throw $Message } }
foreach ($name in @('gpui-validation-common.ps1','measure-gpui.ps1','verify-gpui-package.ps1','compare-gpui-measurements.ps1')) {
    $tokens=$null; $errors=$null
    $null = [Management.Automation.Language.Parser]::ParseFile((Join-Path $scripts $name), [ref]$tokens, [ref]$errors)
    Require ($errors.Count -eq 0) "Syntax error: $name"
}
$checks++
$stats = Get-QuickPickStatistics -Values @(1, 2, 3, 4, 5, 6, 7, 8, 9, 10)
Require ($stats.p50 -eq 5 -and $stats.p95 -eq 10 -and $stats.mean -eq 5.5 -and $stats.count -eq 10) 'Incorrect percentile or mean'
$checks++
$created = [DateTime]::new(2026,11,11)
$processes = @(
    [pscustomobject]@{ ProcessId=10; ParentProcessId=1; CreationDate=$created },
    [pscustomobject]@{ ProcessId=11; ParentProcessId=10; CreationDate=$created.AddSeconds(1) },
    [pscustomobject]@{ ProcessId=12; ParentProcessId=11; CreationDate=$created.AddSeconds(2) },
    [pscustomobject]@{ ProcessId=13; ParentProcessId=10; CreationDate=$created.AddSeconds(-1) },
    [pscustomobject]@{ ProcessId=99; ParentProcessId=1; CreationDate=$created }
)
$tree = @(Get-QuickPickProcessTree -Processes $processes -RootProcessId 10 -RootCreated $created)
Require (($tree.ProcessId | Sort-Object) -join ',' -eq '10,11,12') 'Incorrect process tree or stale-parent filtering'
$checks++
$rejected = $false
try { Get-QuickPickProcessTree -Processes $processes -RootProcessId 10 -RootCreated $created.AddSeconds(-1) | Out-Null } catch { $rejected = $true }
Require $rejected 'Reused root PID was accepted'
$checks++

$temp = Join-Path ([IO.Path]::GetTempPath()) ('quickpick-package-fixtures-' + [Guid]::NewGuid().ToString('N'))
New-Item -ItemType Directory -Path $temp | Out-Null
function New-Fixture([string]$Case) {
    $directory = Join-Path $temp $Case
    New-Item -ItemType Directory -Path $directory | Out-Null
    $pe = [byte[]]::new(256)
    $pe[0]=0x4d; $pe[1]=0x5a
    [BitConverter]::GetBytes([int]64).CopyTo($pe,0x3c)
    [BitConverter]::GetBytes([uint32]0x4550).CopyTo($pe,64)
    $machine = if ($Case -eq 'wrong-machine') { 0x14c } else { 0x8664 }
    [BitConverter]::GetBytes([uint16]$machine).CopyTo($pe,68)
    [BitConverter]::GetBytes([uint16]0x20b).CopyTo($pe,88)
    [BitConverter]::GetBytes([uint16]2).CopyTo($pe,156)
    $zipName = 'QuickPick_0.1.4_gpui_candidate_x64.zip'
    $setupName = if ($Case -eq 'version-mismatch') { 'QuickPick_0.1.5_gpui_candidate_x64_setup.exe' } else { 'QuickPick_0.1.4_gpui_candidate_x64_setup.exe' }
    $zip = [IO.Compression.ZipFile]::Open((Join-Path $directory $zipName), [IO.Compression.ZipArchiveMode]::Create)
    try {
        $entry = $zip.CreateEntry('quickpick.exe'); $stream=$entry.Open()
        try { $stream.Write($pe,0,$pe.Length) } finally { $stream.Dispose() }
        $payload = @{
            'LICENSE'='synthetic project license'; 'THIRD-PARTY-SOURCES.json'='{}'
            'THIRD-PARTY-NOTICES.json'='[{"name":"fixture","version":"1.0.0","license":"MIT","files":["LICENSE"]}]'
            'third-party-licenses/fixture-1.0.0/LICENSE'='synthetic dependency license'
        }
        if ($Case -eq 'multiple-notices') {
            $payload['THIRD-PARTY-NOTICES.json']='[{"name":"fixture","version":"1.0.0","license":"MIT","files":["LICENSE"]},{"name":"second","version":"2.0.0","license":"MIT","files":["LICENSE"]}]'
            $payload['third-party-licenses/second-2.0.0/LICENSE']='second synthetic dependency license'
        }
        if ($Case -eq 'missing-license') { $payload.Remove('third-party-licenses/fixture-1.0.0/LICENSE') }
        if ($Case -eq 'traversal') { $payload['../outside']='synthetic' }
        if ($Case -eq 'unexpected-runtime') { $payload['node.exe']='synthetic' }
        if ($Case -eq 'case-duplicate') { $payload['QUICKPICK.EXE']='synthetic' }
        foreach ($name in $payload.Keys) {
            $writer = [IO.StreamWriter]::new($zip.CreateEntry($name).Open(), [Text.UTF8Encoding]::new($false))
            try { $writer.Write($payload[$name]) } finally { $writer.Dispose() }
        }
    } finally { $zip.Dispose() }
    [IO.File]::WriteAllText((Join-Path $directory $setupName), 'synthetic NSIS placeholder; never execute')
    @(Get-FileHash (Join-Path $directory $zipName),(Join-Path $directory $setupName)) | Select-Object Path,Hash | ConvertTo-Json | Set-Content (Join-Path $directory 'SHA256.json') -Encoding UTF8
    if ($Case -eq 'checksum') { [IO.File]::AppendAllText((Join-Path $directory $setupName), 'changed') }
    return $directory
}
try {
    $directory = New-Fixture 'valid'
    & (Join-Path $scripts 'verify-gpui-package.ps1') -ArtifactsDirectory $directory | Out-Null
    $report = Get-Content (Join-Path $directory 'package-validation.json') -Raw | ConvertFrom-Json
    Require ($report.status -eq 'static-checks-passed' -and $report.third_party_packages -eq 1 -and $report.executable_bytes -eq 256 -and $report.version -eq '0.1.4') 'Valid package fixture failed'
    $checks++
    $zip = [IO.Compression.ZipFile]::OpenRead((Join-Path $directory 'QuickPick_0.1.4_gpui_candidate_x64.zip'))
    $exe = Join-Path $directory 'release.exe'
    try { [IO.Compression.ZipFileExtensions]::ExtractToFile($zip.GetEntry('quickpick.exe'), $exe) } finally { $zip.Dispose() }
    & (Join-Path $scripts 'verify-gpui-package.ps1') -ArtifactsDirectory $directory -Executable $exe -Output (Join-Path $directory 'matching-exe.json') | Out-Null
    $checks++
    [IO.File]::AppendAllText($exe, 'different release')
    $rejected=$false
    try { & (Join-Path $scripts 'verify-gpui-package.ps1') -ArtifactsDirectory $directory -Executable $exe -Output (Join-Path $directory 'mismatching-exe.json') | Out-Null } catch { $rejected=$_.Exception.Message -match 'differs' }
    Require $rejected 'Mismatching standalone release executable was accepted'
    $checks++
    foreach ($case in @('checksum','traversal','unexpected-runtime','missing-license','wrong-machine','case-duplicate','version-mismatch')) {
        $directory = New-Fixture $case
        $rejected = $false
        try { & (Join-Path $scripts 'verify-gpui-package.ps1') -ArtifactsDirectory $directory | Out-Null } catch { $rejected = $true }
        Require $rejected "Invalid package accepted: $case"
        Require (-not (Test-Path (Join-Path $directory 'package-validation.json'))) "Invalid package produced a success report: $case"
        $checks++
    }
    $directory = New-Fixture 'multiple-notices'
    & (Join-Path $scripts 'verify-gpui-package.ps1') -ArtifactsDirectory $directory | Out-Null
    $report = Get-Content (Join-Path $directory 'package-validation.json') -Raw | ConvertFrom-Json
    Require ($report.third_party_packages -eq 2) 'JSON notice arrays were not flattened correctly'
    $checks++
    $summary = @{}
    foreach ($name in @('private_bytes','working_set_bytes','cpu_percent_one_core','cpu_percent_machine')) {
        $summary[$name] = @{ count=60; mean=0; p50=0; p95=0 }
    }
    $baseline = @{ schema=1; label='baseline'; phase='startup-idle'; requested_seconds=60; settle_seconds=60
        samples=@(1..60); process_churn_events=0; os=@{version='synthetic'}; processor=@('synthetic'); graphics=@('synthetic'); logical_processors=4
        summary=$summary; file_version='0.1.3'; executable_sha256=('0'*64) }
    $candidate = $baseline.Clone(); $candidate.label='candidate'; $candidate.file_version='0.1.4'
    $baselinePath = Join-Path $temp 'baseline.json'; $candidatePath = Join-Path $temp 'candidate.json'
    $baseline | ConvertTo-Json -Depth 6 | Set-Content $baselinePath
    $candidate | ConvertTo-Json -Depth 6 | Set-Content $candidatePath
    $comparison = & (Join-Path $scripts 'compare-gpui-measurements.ps1') -Baseline $baselinePath -Candidate $candidatePath | ConvertFrom-Json
    Require ($comparison.metrics.Count -eq 4 -and $null -eq $comparison.metrics[0].change_percent) 'Zero baseline produced a false ratio'
    $checks++
    foreach ($change in @('short','churn','phase','hardware')) {
        $invalid = $candidate.Clone()
        switch ($change) {
            'short' { $invalid.samples=@(1..3) }
            'churn' { $invalid.process_churn_events=1 }
            'phase' { $invalid.phase='after-use-idle' }
            'hardware' { $invalid.logical_processors=8 }
        }
        $invalid | ConvertTo-Json -Depth 6 | Set-Content $candidatePath
        $rejected=$false
        try { & (Join-Path $scripts 'compare-gpui-measurements.ps1') -Baseline $baselinePath -Candidate $candidatePath | Out-Null } catch { $rejected=$true }
        Require $rejected "Incomparable measurement accepted: $change"
        $checks++
    }
} finally { Remove-Item -LiteralPath $temp -Recurse -Force }
Write-Host "$checks validation-tool fixture checks passed. No real Windows application was run or installed."
