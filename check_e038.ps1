$root = "C:\Users\pipos\OneDrive\Desktop\CODEBAZE\ZygoteAI\zygote-runtime"
foreach ($b in @("0_0","0_005","0_015","0_03")) {
    $f = Join-Path $root "events_e038_b$b.jsonl"
    if (-not (Test-Path $f)) { "$b : MISSING"; continue }
    # last snapshot line only (fast)
    $lastSnap = & cmd /c "for /f `"delims=`"` %i in ('findstr /r `"`"`type`"`:`"`snapshot`"``"` events_e038_b$b.jsonl') do @set `"x=%i`"" & echo %x%" 2>$null
    $s = $lastSnap | ConvertFrom-Json
    "$b : pop=$($s.total_cells) producer_frac=$($s.producer_fraction) tick=$($s.tick)"
}
