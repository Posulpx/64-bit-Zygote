import json
last = None
for line in open(r"C:\Users\pipos\OneDrive\Desktop\CODEBAZE\ZygoteAI\zygote-runtime\bench_summary.jsonl", encoding="utf-8", errors="replace"):
    if '"type":"snapshot"' in line:
        try:
            last = json.loads(line)
        except:
            pass
print("last snapshot tick=%s pop=%s pf=%s" % (last.get("tick"), last.get("total_cells"), last.get("producer_fraction")))
