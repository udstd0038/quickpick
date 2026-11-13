function Get-QuickPickStatistics {
    param([Parameter(Mandatory=$true)][double[]]$Values)
    if ($Values.Count -eq 0) { throw 'No samples available.' }
    $ordered = @($Values | Sort-Object)
    $sum = 0.0
    foreach ($value in $ordered) {
        if ([double]::IsNaN($value) -or [double]::IsInfinity($value)) { throw 'Invalid sample.' }
        $sum += $value
    }
    return [pscustomobject]@{
        count = $ordered.Count; min = $ordered[0]; max = $ordered[-1]
        mean = $sum / $ordered.Count
        p50 = $ordered[[Math]::Max(0, [Math]::Ceiling($ordered.Count * 0.50) - 1)]
        p95 = $ordered[[Math]::Max(0, [Math]::Ceiling($ordered.Count * 0.95) - 1)]
    }
}

function Get-QuickPickProcessTree {
    param([object[]]$Processes, [int]$RootProcessId, [DateTime]$RootCreated)
    $root = @($Processes | Where-Object { $_.ProcessId -eq $RootProcessId -and $_.CreationDate -eq $RootCreated })
    if ($root.Count -ne 1) { throw 'The measured application exited or its process ID was reused.' }
    $found = @{}; $found[$RootProcessId] = $root[0]
    do {
        $changed = $false
        foreach ($process in $Processes) {
            $parent = $found[[int]$process.ParentProcessId]
            if ($parent -and -not $found.ContainsKey([int]$process.ProcessId) -and $process.CreationDate -ge $parent.CreationDate) {
                $found[[int]$process.ProcessId] = $process
                $changed = $true
            }
        }
    } while ($changed)
    return @($found.Values)
}
