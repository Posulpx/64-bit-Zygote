import json, glob, os

files = sorted(glob.glob(r"C:\Users\pipos\OneDrive\Desktop\CODEBAZE\ZygoteAI\zygote-runtime\events_e038_fine_b*.jsonl"))
for f in files:
    snaps = {}
    with open(f, "r", encoding="utf-8", errors="replace") as fh:
        for line in fh:
            if '"type":"snapshot"' in line:
                try:
                    s = json.loads(line)
                except:
                    continue
                snaps[s["tick"]] = (s["total_cells"], s["producer_fraction"])
    print(f"=== {os.path.basename(f)} ===")
    for t in [0, 500, 1000, 2000, 3000]:
        if t in snaps:
            pop, pf = snaps[t]
            print(f"  t={t:5d} pop={pop:4d} producer_frac={pf:.4f}")
