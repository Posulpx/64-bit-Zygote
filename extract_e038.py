import json, glob, os

files = sorted(glob.glob(r"C:\Users\pipos\OneDrive\Desktop\CODEBAZE\ZygoteAI\zygote-runtime\events_e038_b*.jsonl"))
for f in files:
    last_snap = None
    type50 = 0
    total_lines = 0
    with open(f, "r", encoding="utf-8", errors="replace") as fh:
        for line in fh:
            total_lines += 1
            if '"type":"snapshot"' in line:
                last_snap = line
            elif '"signal_type":50' in line or '\\"signal_type\\":50' in line:
                type50 += 1
    if last_snap:
        s = json.loads(last_snap)
        print(f"{os.path.basename(f):30s} lines={total_lines} final_pop={s.get('total_cells')} producer_frac={s.get('producer_fraction')} tick={s.get('tick')} type50_edges={type50}")
    else:
        print(f"{os.path.basename(f):30s} lines={total_lines} NO SNAPSHOT")
