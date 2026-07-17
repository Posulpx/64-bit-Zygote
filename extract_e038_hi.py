import json, glob, os

files = sorted(glob.glob(r"C:\Users\pipos\OneDrive\Desktop\CODEBAZE\ZygoteAI\zygote-runtime\events_e038_hi_b*.jsonl"))
for f in files:
    snaps = {}
    min_pop = 10**9
    with open(f, "r", encoding="utf-8", errors="replace") as fh:
        for line in fh:
            if '"type":"snapshot"' in line:
                try:
                    s = json.loads(line)
                except:
                    continue
                snaps[s["tick"]] = (s["total_cells"], s["producer_fraction"])
                if s["total_cells"] < min_pop:
                    min_pop = s["total_cells"]
    print(f"=== {os.path.basename(f)} ===  min_pop={min_pop}")
    for t in [0, 300, 600, 1000, 1500, 2000]:
        if t in snaps:
            pop, pf = snaps[t]
            print(f"  t={t:5d} pop={pop:4d} producer_frac={pf:.4f}")
