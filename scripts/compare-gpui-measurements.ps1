param(
    [Parameter(Mandatory=$true)][string]$Baseline,
    [Parameter(Mandatory=$true)][string]$Candidate,
    [string]$Output
)
$ErrorActionPreference = 'Stop'
$old = Get-Content -LiteralPath $Baseline -Raw -Encoding UTF8 | ConvertFrom-Json
$new = Get-Content -LiteralPath $Candidate -Raw -Encoding UTF8 | ConvertFrom-Json
if ($old.schema -ne 1 -or $new.schema -ne 1 -or $old.label -ne 'baseline' -or $new.label -ne 'candidate') { throw 'Expected a baseline report and a candidate report.' }
foreach ($report in @($old,$new)) {
    if ($report.requested_seconds -lt 60 -or $report.settle_seconds -lt 60 -or @($report.samples).Count -lt 60 -or $report.process_churn_events -ne 0) { throw 'Comparison requires 60 seconds of stabilization, at least 60 samples and no observed process churn.' }
}
if ($old.phase -ne $new.phase -or $old.requested_seconds -ne $new.requested_seconds -or $old.settle_seconds -ne $new.settle_seconds) { throw 'Measurement phase or duration differs.' }
foreach ($field in @('os','processor','graphics','logical_processors')) {
    if (($old.$field | ConvertTo-Json -Depth 5 -Compress) -ne ($new.$field | ConvertTo-Json -Depth 5 -Compress)) { throw "Measurement environment differs: $field" }
}
$metrics = @()
foreach ($name in @('private_bytes','working_set_bytes','cpu_percent_one_core','cpu_percent_machine')) {
    $before = $old.summary.$name; $after = $new.summary.$name
    if ($before.count -lt 60 -or $after.count -lt 60) { throw 'Incomplete summary.' }
    foreach ($value in @($before.mean,$after.mean,$before.p50,$after.p50,$before.p95,$after.p95)) {
        if ($null -eq $value -or [double]::IsNaN([double]$value) -or [double]::IsInfinity([double]$value) -or [double]$value -lt 0) { throw 'Invalid metric.' }
    }
    $change = $null
    if ($before.mean -gt 0) { $change = 100 * ($after.mean - $before.mean) / $before.mean }
    $metrics += [pscustomobject]@{ name=$name; baseline_mean=$before.mean; candidate_mean=$after.mean
        delta_mean=($after.mean - $before.mean); change_percent=$change
        baseline_p50=$before.p50; candidate_p50=$after.p50; baseline_p95=$before.p95; candidate_p95=$after.p95 }
}
$result = [pscustomobject]@{
    schema=1; phase=$old.phase
    baseline_sha256=$old.executable_sha256; candidate_sha256=$new.executable_sha256
    baseline_version=$old.file_version; candidate_version=$new.file_version
    metrics=$metrics
    limitations=@('Environment fields match; the operator must still confirm the same machine, power mode, DPI, workload and background applications.',
        'Working sets are summed across processes and can count shared pages more than once.',
        'This comparison does not measure GPU activity, startup, interactive latency or leak freedom; no release acceptance is inferred.')
}
$json = $result | ConvertTo-Json -Depth 7
if ($Output) {
    $destination = [IO.Path]::GetFullPath($Output)
    if (Test-Path -LiteralPath $destination) { throw 'Report already exists; use a new output path.' }
    New-Item -ItemType Directory -Force -Path (Split-Path $destination -Parent) | Out-Null
    $json | Set-Content -LiteralPath $destination -Encoding UTF8
} else { $json }
