param(
    [Parameter(Mandatory=$true)][string]$Executable,
    [Parameter(Mandatory=$true)][int]$ProcessId,
    [Parameter(Mandatory=$true)][ValidateSet('baseline','candidate')][string]$Label,
    [ValidateSet('startup-idle','after-use-idle')][string]$Phase = 'startup-idle',
    [ValidateRange(1,3600)][int]$Seconds = 60,
    [ValidateRange(0,3600)][int]$SettleSeconds = 60,
    [string]$Output
)
$ErrorActionPreference = 'Stop'
if ($env:OS -ne 'Windows_NT') { throw '此采样脚本需要 Windows。' }
. (Join-Path $PSScriptRoot 'gpui-validation-common.ps1')
$binary = (Get-Item -LiteralPath $Executable).FullName
$rootProcess = Get-Process -Id $ProcessId
if ($rootProcess.Path -ne $binary) { throw 'ProcessId 与指定的可执行文件路径不匹配。' }
$rootInfo = Get-CimInstance Win32_Process -Filter "ProcessId=$ProcessId" -Property ProcessId,ParentProcessId,CreationDate
$rootCreated = $rootInfo.CreationDate
$cpuCount = [Environment]::ProcessorCount
$started = [DateTime]::UtcNow.ToString('o')
Write-Host "仅采样已运行的 $Label 进程树；稳定等待 $SettleSeconds 秒，然后采样 $Seconds 秒。请保持相同电源、缩放与操作条件。"
for ($remaining = $SettleSeconds; $remaining -gt 0; $remaining--) {
    if ($rootProcess.HasExited) { throw '稳定等待期间被测程序已退出。' }
    if ($remaining % 10 -eq 0) { Write-Host "稳定等待剩余 $remaining 秒" }
    Start-Sleep -Seconds 1
}
$watch = [Diagnostics.Stopwatch]::StartNew()
$samples = @(); $lastCpu = @{}; $previousTime = 0.0; $churn = 0
for ($index = 0; $index -le $Seconds; $index++) {
    $snapshot = @(Get-CimInstance Win32_Process -Property ProcessId,ParentProcessId,CreationDate)
    $tree = @(Get-QuickPickProcessTree -Processes $snapshot -RootProcessId $ProcessId -RootCreated $rootCreated)
    $private = 0L; $working = 0L; $cpu = @{}
    foreach ($node in $tree) {
        try {
            $process = Get-Process -Id $node.ProcessId -ErrorAction Stop
            if ([Math]::Abs(($process.StartTime.ToUniversalTime() - $node.CreationDate.ToUniversalTime()).TotalMilliseconds) -gt 100) { throw 'Identity changed' }
            $identity = "$($node.ProcessId):$($node.CreationDate.ToUniversalTime().Ticks)"
            $private += $process.PrivateMemorySize64
            $working += $process.WorkingSet64
            $cpu[$identity] = $process.TotalProcessorTime.TotalSeconds
        } catch { throw '进程在采样中退出或无法读取计数器；本轮采样无效，请重新执行。' }
    }
    $now = $watch.Elapsed.TotalSeconds
    if ($index -gt 0) {
        $delta = 0.0
        foreach ($identity in $cpu.Keys) {
            if ($lastCpu.ContainsKey($identity)) { $delta += [Math]::Max(0, $cpu[$identity] - $lastCpu[$identity]) }
            else { $churn++ }
        }
        foreach ($identity in $lastCpu.Keys) { if (-not $cpu.ContainsKey($identity)) { $churn++ } }
        $rate = $delta / ($now - $previousTime)
        $samples += [pscustomobject]@{
            elapsed_seconds = $now; process_count = $tree.Count
            private_bytes = $private; working_set_bytes = $working
            cpu_percent_one_core = 100 * $rate
            cpu_percent_machine = 100 * $rate / $cpuCount
        }
    }
    $lastCpu = $cpu; $previousTime = $now
    if ($index -lt $Seconds) { Start-Sleep -Milliseconds 1000 }
    if ($index -gt 0 -and $index % 10 -eq 0) { Write-Host "已采样 $index / $Seconds" }
}
$watch.Stop()
$operatingSystem = Get-CimInstance Win32_OperatingSystem -Property Version,BuildNumber
$processor = @(Get-CimInstance Win32_Processor -Property Name | ForEach-Object { $_.Name })
$graphics = @(Get-CimInstance Win32_VideoController -Property Name,DriverVersion | Select-Object Name,DriverVersion)
$report = [pscustomobject]@{
    schema = 1; label = $Label; phase = $Phase; started_utc = $started
    executable_sha256 = (Get-FileHash -LiteralPath $binary -Algorithm SHA256).Hash
    executable_bytes = (Get-Item -LiteralPath $binary).Length
    file_version = (Get-Item -LiteralPath $binary).VersionInfo.FileVersion
    os = [pscustomobject]@{ version=$operatingSystem.Version; build=$operatingSystem.BuildNumber }
    processor = $processor; graphics = $graphics; logical_processors = $cpuCount
    settle_seconds = $SettleSeconds; requested_seconds = $Seconds; actual_seconds = $watch.Elapsed.TotalSeconds
    process_churn_events = $churn
    summary = [pscustomobject]@{
        private_bytes = Get-QuickPickStatistics -Values @($samples.private_bytes)
        working_set_bytes = Get-QuickPickStatistics -Values @($samples.working_set_bytes)
        cpu_percent_one_core = Get-QuickPickStatistics -Values @($samples.cpu_percent_one_core)
        cpu_percent_machine = Get-QuickPickStatistics -Values @($samples.cpu_percent_machine)
    }
    samples = $samples
    limitations = @('Sampling omits CPU used between snapshots by short-lived processes. Process churn requires review.',
        'Private bytes and working set include observed descendants; working-set sums can count shared pages more than once. GPU memory and GPU utilization are not measured.',
        'This measures stable idle resources, not cold startup, first frame, interaction latency or leak freedom.')
}
if (-not $Output) { $Output = Join-Path (Split-Path $PSScriptRoot -Parent) "target/gpui-candidate/gpui-stage5-reports/$Label-$Phase-$([DateTime]::UtcNow.ToString('yyyyMMdd-HHmmss')).json" }
$destination = [IO.Path]::GetFullPath($Output)
New-Item -ItemType Directory -Force -Path (Split-Path $destination -Parent) | Out-Null
if (Test-Path -LiteralPath $destination) { throw '报告已存在，未覆盖。' }
$report | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath $destination -Encoding UTF8
Write-Host "资源采样完成：$destination。未记录进程参数、窗口标题、正文、截图或用户配置。"
