#!/usr/bin/env pwsh
# E038 — Ecosystem Collapse Threshold sweep.
#
# We start from the E037 directed-kin setup (memory = producer that emits the
# protective type-50 warning to kin only; listener = gets protected; reactive =
# free-rider that neither pays the +2/tick memory tax nor emits warnings).
#
# To find Law #064's measurable threshold we apply gradualy rising extra
# mortality ONLY to producers (is_memory) via --producer-death-bonus. As the
# producer fraction drops, the warning network should thin, protection should
# collapse, and finally total population should crash.
#
# For each bonus level we record: final producer_fraction, final population,
# and whether the warning network (type-50 edges) collapsed.

$ErrorActionPreference = "Stop"
$root = "C:\Users\pipos\OneDrive\Desktop\CODEBAZE\ZygoteAI"
$rt   = Join-Path $root "zygote-runtime"
$bin  = Join-Path $rt "target\release\zygote-runtime.exe"
$genome = Join-Path $rt "genomes\e038_ecosystem_collapse.json"

# E037-style environment: warning type-9 every 60 ticks, shock 5 ticks later
# (real 80% / false 20%), food type-1 injected every 2 ticks, shuffled,
# max 600 cells. Same baseline that produced ~600 stable cells in E037.
$warns = @()
for ($t = 100; $t -lt 5000; $t += 60) { $warns += "--warn-signal"; $warns += "9@$t" }

$common = @(
    "--genome", $genome,
    "--ticks", "5000",
    "--max-cells", "600",
    "--shuffle",
    "--death-rate", "0.0002",
    "--shock-death-rate", "0.04",
    "--shock-duration", "40",
    "--warn-shock-prob", "0.8",
    "--warn-shock-delay", "5",
    "--inject", "1,0,4294967295",
    "--inject-interval", "2",
    "--directed-warning"
) + $warns

# Sweep the extra producer mortality. The producer tax on its own is +2/tick;
# these bonuses stack lethal pressure on top so the fraction erodes over time.
$bonuses = @(0.0, 0.001, 0.003, 0.005, 0.008, 0.01, 0.015, 0.02, 0.03, 0.05)

$results = @()
foreach ($b in $bonuses) {
    $out = Join-Path $rt ("events_e038_b{0}.jsonl" -f ($b -replace '\.', '_'))
    Write-Host "=== E038 sweep: producer_death_bonus = $b ==="
    & $bin @common --producer-death-bonus $b --events $out | Out-Null
    # Pull final snapshot + collapse signal from the events file.
    $last = Get-Content $out | Where-Object { $_ -like '*"type":"snapshot"*' } | Select-Object -Last 1
    $snap = $last | ConvertFrom-Json
    $warnings = (Get-Content $out | Where-Object { $_ -like '*"type":"signal"*' -and $_ -like '*"signal_type":50*' }).Count
    $results += [pscustomobject]@{
        bonus              = $b
        final_pop          = $snap.total_cells
        final_producer_frac = [math]::Round($snap.producer_fraction, 4)
        type50_edges       = $warnings
    }
    Write-Host ("  final_pop={0} producer_frac={1} type50_edges={2}" -f $snap.total_cells, $snap.producer_fraction, $warnings)
}

Write-Host ""
Write-Host "=== E038 SWEEP SUMMARY ==="
$results | Format-Table -AutoSize
