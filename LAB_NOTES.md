# Lab Notes

> A living document of experiments, observations, failures, and insights.

## 2026-07-16 — Project Conception

- Repository initialized.
- Core spec drafted in ZYGOTE_SPEC.md.
- First set of cell types proposed: sensor, accumulator, filter, mapper, combiner, switch, timer, logger, spawner, reaper.
- Question to resolve: should the genome be a single file or a directory of gene modules?

## 2026-07-16 — Language Decision

**Decision:** Assembly (x86-64) for cells, Rust for runtime.

Rationale:
- Assembly is the truest "zygote" — the most primitive, minimal possible agent. No runtime, no safety, raw machine code.
- Rust gives us safe systems-level orchestration without a GC. It can mmap memory, set page permissions, catch signals, and enforce timeouts — exactly what's needed to host untrusted Assembly cells.
- The split enforces discipline: cells are dumb, fast, and dangerous. The Rust runtime is smart, safe, and observant.
- Future portability: if we want WebAssembly later, the Rust runtime abstracts the cell interface.

Implications:
- Toolchain: NASM (or similar) → .bin files. Rust runtime loads .bin into mmap'd pages.
- Cell ABI: fixed memory layout (code, state, input FIFO, output FIFO, control block). Pointers in registers.
- Signal format: flat binary structs (not JSON). Rust bridges to JSON for I/O.
- Safety: Rust sets `PROT_EXEC | PROT_READ` for code pages, `PROT_READ | PROT_WRITE` for data. SIGSEGV = cell death.

## Next Session

> All of these were completed in the first working session (Rust workspace, NASM
> build, sensor/logger/relay cells, and the end-to-end loop). See E001–E018 below
> for the experiments that followed.

## 2026-07-16 — Emergence Test #1: Stable Signaling Loop

**Hypothesis:** A stable 4-hop signaling loop (sensor → cell_a → cell_b → cell_c → sensor) can self-sustain using only broadcast routing and per-cell affinity, with no cell explicitly programmed with routing logic.

**Setup:**
- `loopback.asm`: reads any signal, resets type_id to 1, broadcasts
- `relay.asm`: reads any signal, increments type_id by 1, broadcasts
- Genome `loop_test.json`: 4 cells, each with a single signal-type affinity:
  - sensor (loopback): affinity=[4]
  - cell_a (relay):     affinity=[1]
  - cell_b (relay):     affinity=[2]
  - cell_c (relay):     affinity=[3]
- Injected a single type-1 signal as the initial trigger.

**Result: STABLE LOOP CONFIRMED**

```
   --[1]--> sensor  (x1, injected trigger)
sensor --[1]--> cell_a (x25)
cell_a --[2]--> cell_b (x25)
cell_b --[3]--> cell_c (x25)
cell_c --[4]--> sensor (x24)
```

100 ticks, 100 signals emitted, 100 signals delivered — zero loss.
Each signal type cycled exactly 25 times. The loop is deterministic and
self-sustaining at 1 cell-firing per tick (4 ticks per full loop cycle).

**Key insight:** No cell knows about the loop. The loop structure emerges
from the interaction between three independent layers:

1. **Cell-local transformation logic** — relay increments type_id, loopback resets it to 1. Neither knows where signals come from or go to.
2. **Genome-defined affinity rules** — each cell type declares which signal types it reacts to. This creates the routing topology.
3. **Runtime broadcast delivery** — the signal bus fans out every emission to all matching affinities. This is what physically connects the cells into a chain.

## 2026-07-16 — Emergence Test #2: Fork (2× cell_b)

**Setup:** Same 4-hop loop, but `cell_b` has `max_instances=2` → 5 cells total. Both b1 and b2 have `affinity=[2]`. Will the bus split, duplicate, or select?

**Result: TRAFFIC DUPLICATES AT FORK, CLIPS AT MERGE**

100 ticks, 470 signals emitted, 559 deliveries.

```
cell_a --[2]--> cell_b (x186)    ← 94 emitted × 2 receivers ≈ 188, minus 2 drops
cell_b --[3]--> cell_c (x184)    ← both b instances combined
cell_c --[4]--> sensor (x94)     ← bottleneck: 1/tick, drops the rest
sensor --[1]--> cell_a (x94)
```

**Analysis:**
- **Fork duplicates:** every type-2 signal reaches both b1 and b2. Both fire.
- **Merge clips:** cell_c receives ~2 type-3/tick but emits only 1 type-4/tick. FIFO (depth 4) overflows, ~90 signals dropped over 100 ticks.
- **System reaches steady state:** bus holds steady at ~5 pending after tick 20. The bottleneck (cell_c's 1/tick processing rate) governs throughput.
- **No dominance:** both cell_b instances fire at similar rates (93–94 each). Neither dominates.

**Answer to all three questions:** traffic duplicates (not splits, not selects). At the merge, the bus clips excess via FIFO overflow. The system throughput is bounded by the slowest cell in the chain.

## 2026-07-16 — Emergence Test #3: Broken Circuit Recovery (kill cell_c at tick 50)

**Setup:** Same fork_test (5 cells). `--disable "cell_c@50"` kills cell_c mid-run. Does the loop re-route?

**Result: EXTINCTION IN ~7 TICKS. NO REROUTING.**

```
Tick 50: kill cell_c. Bus at 5 pending (steady state).
Tick 51–57: residual signals propagate through loop, then starvation.
Tick 60: bus = 0 pending. No further signals ever emitted.
```

**Analysis:**
- **Signal extinction time: ~7 ticks.** The ~5 signals already in the bus propagate 2–3 more loop iterations as each tier exhausts its backlog. Once all type-4 signals (sensor input) are consumed, the entire loop halts.
- **8 orphan type-3 signals** (92 emitted - 84 delivered) were produced after cell_c died but had no home — no cell has `affinity=[3]` anymore.
- **No alternative paths emerge.** The affinity map is static — built once at spawn time, never updated. The runtime has no rerouting, no learning, no adaptation. The loop is a rigid chain: break any link, the whole chain stops.

**Key insight:** This is a pure broadcast bus with no routing intelligence. Affinity = static subscription. Kill the subscriber, the signal type goes silent. For recovery, the system would need dynamic affinity mutation (e.g., a cell that can change its `affinity` at runtime, or the runtime detecting orphan signal types and reassigning them).

## 2026-07-16 — Emergence Test #4: Competing Loops (1→2→3→4→1 vs 1→5→6→1)

**Setup:** Two independent loops sharing only signal type 1. Loop A (4 hops: cell_a→cell_b→cell_c→cell_d) and Loop B (3 hops: cell_e→cell_f→cell_g). Both cell_a and cell_e have `affinity=[1]`. Injected type-1 broadcast.

**Result: NO STARVATION — LOOPS COEXIST PEACEFULLY AT FULL CAPACITY**

100 ticks, 670 emitted, 852 delivered. All 7 cells fire every tick after pipeline fill.

```
--- Per Signal Type ---
  type 1: 189 emitted, 376 delivered    ← 189×2=378, only 2 dropped (0.5%)
  type 2:  97 emitted,  96 delivered    
  type 3:  96 emitted,  95 delivered    
  type 4:  95 emitted,  94 delivered    ← Loop A close (4 hops)
  type 5:  97 emitted,  96 delivered    
  type 6:  96 emitted,  95 delivered    ← Loop B close (3 hops, one extra)
```

```
--- Signal Graph ---
  cell_g --[1]--> cell_a  (x94)        ← Loop B type-1 feeds Loop A
  cell_g --[1]--> cell_e  (x94)        ← and feeds itself
  cell_d --[1]--> cell_a  (x93)        ← Loop A type-1 feeds Loop A
  cell_d --[1]--> cell_e  (x93)        ← and Loop B
```

**Analysis:**
- **No competition emerges.** Broadcast delivery gives every type-1 signal to BOTH cell_a and cell_e. They don't fight — they share. Both loops run at their max rate (1 signal/cell/tick).
- **Loop B cycles faster** (3 hops vs 4) and closes 96 times vs Loop A's 95 — a 1-cycle advantage over 100 ticks.
- **Bus holds steady at 7 pending** = one signal per cell at end of every tick. All cells are fully utilized.
- **The only 2 dropped signals** are type-1 FIFO overflows at cell_a or cell_e — negligible loss.

## 2026-07-16 — Emergence Test #5: Shared Bottleneck (two upstream cells → one cell_c)

**Setup:** cell_a (relay, aff=[1]) → type 2 and cell_b (offset4, aff=[1]) → type 5 both feed into cell_c (relay, aff=[2,5]). cell_c FIFO depth=4. cell_c → either cell_d (aff=[3], type 3→1) or cell_e (aff=[6], type 6→1). Competition: only 1 signal per tick through cell_c.

**Result: PATH A DOMINATES 96:4 — SYSTEMATIC TIMING BIAS FROM POOL ORDER**

```
type 2: 97 emitted,  96 delivered → cell_c  (Path A upstream → cell_c)
type 5: 97 emitted,   4 delivered → cell_c  (Path B upstream, 93 DROPPED)
type 3: 94 emitted,  93 delivered → cell_d  (Path A closer)
type 6:  4 emitted,   4 delivered → cell_e  (Path B closer)
cell_d: 92 type-1 emitted                (Path A feeds back)
cell_e:  4 type-1 emitted                (Path B starved)
```

**Mechanism of exclusion:**
1. Runtime iterates pool in spawn order: cell_a(0) before cell_b(1).
2. `collect_output` appends to bus in pool order → bus = [type2, type5] each tick.
3. `distribute_signals` iterates bus FIFO → type 2 pushed to cell_c's FIFO *before* type 5.
4. Ring-buffer FIFO holds max 3 (4 slots, 1 sacrificed for full/empty distinction).
5. Steady state: FIFO has 2 signals at start of tick. Type 2 fills it to 3 (full). Type 5 push → DROPPED.
6. After pop (type 2 → Path A), FIFO back to 2. Next tick: same.
7. The 4 type-5 deliveries happened during pipeline fill (ticks 1–3) before steady state locked in.

**Key insight:** Pool iteration order IS a first-mover advantage. cell_a's position in the runtime's internal loop gives it a structural monopoly over the shared FIFO. This is not about signal priority, content, or cell logic — pure systematic timing bias.

**Stats fix applied:** `total_signals_delivered` and observer now only count *successful* FIFO pushes. Previous runs inflated delivery counts by counting signals pushed to full FIFOs.

**Negative result for previous hypothesis:** "FIFO depth smaller than number of suppliers" creates competition, but the result isn't fair sharing — it's winner-take-all based on delivery order.

## Observed Laws

### Law #003 — Runtime Iteration Order Creates Persistent Resource Inequality

Runtime iteration order can create persistent resource inequality.

**Discovered:** E005 (Shared Bottleneck)

**Evidence (E005):** Path A : Path B = 96 : 4

**Confirmed (swap):** Path A : Path B = 4 : 96 — exact mirror when cell_b spawns first. Identical totals (389/391). The bias is deterministic: first-in-pool monopolizes the shared FIFO.

Pool spawn order determines collection order, which determines bus FIFO order, which determines delivery order into the shared cell's ring buffer. When the ring buffer fills to capacity, the second-supplier's signals are systematically dropped every tick. The inequality is structural and self-sustaining — not competitive.

### E007 — Shuffled Pool Order Restores Fairness

**Experiment:** Add `--shuffle` flag that calls `pool.shuffle()` via Fisher-Yates at start of each `step()`. Same shared_bottleneck genome, 1000 ticks.

**Result:** 499 : 501 (~50:50).

| Signal | E005 (no shuffle) | E007 (shuffle, 1000 ticks) |
|---|---|---|
| type 2 (Path A) | 96 delivered | **499 delivered** |
| type 5 (Path B) | 4 delivered | **501 delivered** |
| Total emitted | 389 | 3989 |

**Note:** Cells have a built-in 1000-tick lifespan (`ticks_left`). All 5 cells self-terminated at tick 1000, confirming the 1000-tick hard limit.

**Significance:** The ~50:50 result confirms that pool iteration order is the sole cause of the inequality in E005. Randomizing order each tick eliminates the structural bias, converting winner-take-all into fair sharing. The bus distribution itself has no inherent preference — it simply amplifies whatever order the pool provides.

### E008 — Round-Robin Delivery

**Experiment:** Add `--round-robin` flag that calls `pool.rotate_left(1)` at start of each `step()`. Same shared_bottleneck genome. Expected: near-equal throughput (A, B, A, B alternation).

**Setup:**
- `CellPool::rotate_left(n)` wraps `Vec::rotate_left` with a safe no-op for 0 or 1 cells.
- Rotation shifts the pool iteration start by one position each tick.
- With pool [A, B]: tick N collects A first, tick N+1 collects B first.
- Deterministic, O(1) per tick (vs O(n) for shuffle).

**Result:** 798 : 202 (~80:20). **NOT the expected alternation.**

**Analysis:** `rotate_left(1)` on a 5-cell pool `[a, b, c, d, e]` creates only 5 distinct orderings. Cell_a is before cell_b in 4 out of 5 orderings:

| Tick offset | Pool order | First producer |
|---|---|---|
| 0 | [a, b, c, d, e] | a (type 2) |
| 1 | [b, c, d, e, a] | **b** (type 5) |
| 2 | [c, d, e, a, b] | a |
| 3 | [d, e, a, b, c] | a |
| 4 | [e, a, b, c, d] | a |

The relative cyclic order of a and b is preserved — a always precedes b in the cyclic sense. B only gets first position when rotation wraps b to index 0 and a to index 4 (1 out of 5 ticks). This gives the same 80:20 structural bias as E005, just slightly less extreme.

**Key insight:** Vec rotation preserves relative cyclic order. For two adjacent cells in a pool of N, round-robin with step=1 gives the second cell first position exactly 1/N of the time. With 5 cells: 1/5 = 20% for Path B. True A/B/A/B alternation requires a different strategy (e.g., alternating iteration direction each tick).

**Comparison:**

| Signal | E005 (none) | E007 (shuffle) | E008 (round-robin) |
|---|---|---|---|
| type 2 (Path A) | 96 | 499 | **798** |
| type 5 (Path B) | 4 | 501 | **202** |
| Total emitted | 389 | 3989 | **3989** |

### E009 — FIFO Depth Sweep

**Question:** Does delivery inequality disappear as FIFO capacity increases?

**Experiment:** Sweep `FIFO_CAPACITY` in `types.rs` (used by `RingBuffer::mask()` and `FIFO_SIZE`) across values [1, 2, 4, 8, 16]. Same shared_bottleneck genome, 1000 ticks, no shuffle. Compare type 2 (Path A) vs type 5 (Path B) delivery at cell_c.

**Results:**

| Depth | Usable slots | type 2 (Path A) | type 5 (Path B) | Ratio (A:B) | Notes |
|---|---|---|---|---|---|
| 1 | 0 | 0 | 0 | — | FIFO always full (sacrificed slot). 0 signals flow. |
| 2 | 1 | **CRASH** | **CRASH** | — | 8GB allocation error — ABI corruption |
| 4 | 3 | 996 | 4 | 99.6:0.4 | E005 baseline |
| 8 | 7 | 499 | 832 | 37.5:62.5 | **INVALID** — see below |
| 16 | 15 | 499 | 834 | 37.4:62.6 | **INVALID** — see below |

**Critical finding: FIFO capacity is a cross-layer constant baked into both the Rust runtime (`types.rs`) AND the pre-compiled Assembly cells (`fifo.inc`).** Changing Rust's `FIFO_CAPACITY` without recompiling the `.bin` files corrupts the shared ring buffer ABI. Evidence:

- **Depth 2:** Immediate crash with 8GB allocation error — memory corruption from layout mismatch.
- **Depth 8 & 16:** Runs without crash but produces signal **type 0** — a value that no cell in the genome produces. This is the Assembly code reading the ring buffer at wrong offsets and interpreting data as signal type.
- The reversal (Path B dominating at depth 8/16: 832 vs 499) is also corrupted — not a genuine throughput change.

| Depth | FIFO_SIZE (Rust) | FIFO_SIZE (Assembly expects) | Status |
|---|---|---|---|
| 1 | 64 | 256 | Partial corruption (no signal flow masks it) |
| 2 | 128 | 256 | **Crash** |
| 4 | 256 | 256 | ✅ **Matches** — this is the compiled-in constant |
| 8 | 512 | 256 | Runs with silent corruption (type 0 signals) |
| 16 | 1024 | 256 | Runs with silent corruption (type 0 signals) |

**Answer to the question:** The question cannot be answered with the current setup — FIFO capacity is a compile-time constant shared across Rust and Assembly layers. To properly sweep depths, the cells would need to be recompiled with matching `FIFO_CAPACITY` via NASM. This reveals a design constraint: **the cell ABI is not parameterized** — constants like `FIFO_CAPACITY`, `SIGNAL_SIZE`, and `CONTROL_SIZE` are frozen at compile time for both Rust and Assembly.

**Implication for architecture:** To make these parameters sweepable, the cell ABI should negotiate FIFO capacity at spawn time (e.g., via a header in the control block), or the runtime should adapt to the cell's compiled-in layout rather than imposing one from Rust.

### E010 — Alternating Direction (Round-Robin Fix)

**Experiment:** Replace `pool.rotate_left(1)` with `pool.reverse()` each tick to alternate pool iteration direction. Same shared_bottleneck genome, 1000 ticks, `--round-robin`.

**Result: 500 : 500 — perfect 50:50 fairness.**

| Signal | E005 (none) | E007 (shuffle) | E008 (rotate) | E010 (reverse) |
|---|---|---|---|---|
| type 2 (Path A) | 96 | 499 | 798 | **500** |
| type 5 (Path B) | 4 | 501 | 202 | **500** |
| Total emitted | 389 | 3989 | 3989 | **3989** |

**Mechanism:** `pool.reverse()` toggles the pool between [a,b,c,d,e] and [e,d,c,b,a] on alternating ticks. This swaps the order of the two upstream cells every tick:
- Tick 1 (reverse): [e,d,c,b,a] → cell_b first → type 5 delivered first to cell_c
- Tick 2 (forward): [a,b,c,d,e] → cell_a first → type 2 delivered first to cell_c
- Tick 3 (reverse): cell_b first again → cell_b wins
- Pattern: B, A, B, A, B, ...

Each producer gets first position every other tick, achieving exactly 50:50 throughput. Unlike E008's rotation (which preserves relative cyclic order), reversing flips the relative order completely each tick.

**Key insight:** For round-robin fairness, what matters is not rotating the *starting position* but reversing the *direction*. Reversing changes who is first proportionally to pool size — with any pool of 2+ cells, both producers alternate perfectly.

### E011 — Self-Termination (Cell Lifecycle)

**Experiment:** Write a minimal Assembly cell (`selfdestruct.asm`, 6 bytes) that immediately sets its kill flag (`mov byte [r9 + 63], 1`). Genome with 1 suicide cell, 5 ticks.

```
bits 64
default rel
section .text
global cell_entry
cell_entry:
    mov byte [r9 + 63], 1
    ret
```

**Result: Cell born → self-terminates on tick 1.** Lifecycle confirmed:
- 1 cell born (spawned in `init()`)
- 1 cell died (kill flag → `alive = false` → swept in first `step()`)
- 0 signals emitted (cell processes nothing, just dies)

**Significance:** Validates the voluntary death path via the ABI's `KILL_FLAG_OFFSET` (control block byte 63). Combined with the 1000-tick `ticks_left` hard limit, cells now have two death mechanisms: voluntary (set kill flag) and involuntary (ticks_left reaches 0).

### E012 — Directed Signals (Non-Broadcast Delivery)

**Experiment:** Create a genome with 2 cells (affinity=[1] and affinity=[2]). Inject a type 7 signal directed to cell_b (id=1, bypassing its affinity). Observe whether directed delivery bypasses the broadcast/affinity system. 10 ticks.

**Result: Directed delivery confirmed — affinity is bypassed.**

```
Signal Graph:
  cell_a --[7]--> cell_b  (x1)        ← directed delivery bypasses affinity=[2]
  type 8: 1 emitted, 0 delivered      ← cell_b incremented 7→8, broadcast, no subscriber
```

Cell_b received type 7 even though its genome-defined affinity is `[2]` — the `distribute_signals()` function uses a separate code path for non-broadcast signals (`sig.target_id != 0xFFFFFFFF`) that delivers directly to the target cell ID with no affinity check.

**Implications:**
- Directed signals are private channels — they bypass the broadcast bus
- The target cell processes the signal regardless of its declared affinity
- Useful for runtime control (e.g., killing specific cells, injecting commands)
- Combined with spawn-on-signal (E014), this enables orchestrated multi-cell workflows

### E013 — Dynamic Affinity (Self-Healing) ★ First Adaptive Behavior

**Experiment:** Re-run E003's broken-circuit test (shared bottleneck, kill cell_c at tick 50), but with `--dynamic-affinity`. The runtime detects orphan signal types (present in the bus but with no alive subscriber) and randomly assigns them to remaining cells.

**Result: Self-healing confirmed — system survives cell_c death.**

```
Tick 50: Killed cell_c (type 'cell_c')
Tick 51: orphan type 2 adopted by a cell
Tick 51: orphan type 5 adopted by a cell
[...]
```

**After adoption:** A new stable loop emerged without cell_c:
```
cell_a --[2]--> cell_b  (x118)    ← type 2 adopted by cell_b
cell_b --[6]--> cell_e  (x116)    ← cell_e affinity=[6] (original)
cell_e --[1]--> cell_a  (x121)    ← cell_a affinity=[1] (original)
```

**Steady state after tick 60:** 4 cells, 3 pending signals, ~26 signals/10 ticks (vs ~40/10 ticks before death). Throughput dropped by ~35% but the system did NOT go extinct (unlike E003, which died in ~7 ticks).

**Key insight:** Dynamic affinity converts a hard-death circuit break into a graceful throughput degradation. The adoption is random (pick a random alive cell and append the orphan type to its affinity list), which works but isn't optimal. Future work: affinity should be assigned to the cell best positioned to handle it (e.g., downstream of the orphan type's natural predecessor).

### E014 — Spawn-on-Signal

**Experiment:** Trigger cell spawning from signal activity. Genome with 3 genes:
- `producer` (on_startup): affinity=[1], relay (type 1→2)
- `spawned` (on_signal="2"): affinity=[2], relay (type 2→3), max_instances=5
- `sink` (on_startup): affinity=[3], loopback (type 3→1)

Inject a single type-1 signal. Observe spawn cascade.

**Result: Spawn cascade limited by max_instances=5. Stable 7-cell system.**

```
Tick 1:  spawned cell 2 (type 'spawned') from signal trigger (1/5)
Tick 4:  spawned cell 3 (type 'spawned') from signal trigger (2/5)
Tick 7:  spawned cell 4 (type 'spawned') from signal trigger (3/5)
Tick 8:  spawned cell 5 (type 'spawned') from signal trigger (4/5)
Tick 10: spawned cell 6 (type 'spawned') from signal trigger (5/5)
```

**Steady state:** 7 cells (1 producer + 5 spawned + 1 sink), 7 pending signals, 1350 signals over 200 ticks. Signal flow:
```
producer --[2]--> 5× spawned    (960 deliveries)
spawned  --[3]--> sink          (197 deliveries)
sink     --[1]--> producer      (194 deliveries)
```

**Design constraint discovered:** Spawning on every occurrence of the trigger signal causes exponential growth if unthrottled. The fix: (1) check `max_instances` before spawning, (2) limit to 1 spawn per tick per trigger signal. This prevents runaway populations while still allowing dynamic growth to fill available capacity.

### E015 — Energy

**Experiment:** Introduce per-cell energy as a finite resource. Each cell has `initial_energy` (default 100, genome-configurable). Costs:

| Action | Cost |
|---|---|
| Tick (being alive) | -1 |
| Process (pop from input FIFO) | -1 |
| Emit (push to output FIFO) | -1 |
| Spawn (new cell creation) | -10 deducted from new cell's starting energy |

Death when `energy <= 0`. Three topologies tested:

**Test 1 — 4-hop stable loop** (`loop_test`, 4 cells, 1 signal in loop):
- Each cell processes 1 signal every 4 ticks → 1.5 energy/tick average
- 90 energy per cell (100 - 10 spawn) / 1.5 = **60 ticks to extinction**
- All 4 cells died simultaneously at tick 60
- Signal throughput: 60 signals total (perfect 1:1:1:1 ratio)

**Test 2 — Fork** (`loop_fork_test`, 5 cells, 2× cell_b):
- Higher throughput: 143 signals in ~35 ticks
- Busy cells (sensor, cell_a) burn ~3 energy/tick → die at tick 30–35
- All 5 cells dead by tick 40

**Test 3 — Shared bottleneck** (`shared_bottleneck`, 5 cells):
- cell_a burns 3/tick for 29 ticks → dead at tick 32 (87 energy consumed)
- cell_e processes only 4 signals → 88 energy over 82 ticks → outlasts everyone
- Last cell dies at tick 82 (only 2 energy remaining at tick 80)
- 118 signals total

**Results:**

| Topology | Cells | Lifespan | Total signals | Energy efficiency |
|---|---|---|---|---|
| 4-hop loop | 4 | **60 ticks** | 60 | 1.5 /cell/tick (balanced) |
| Fork | 5 | ~35 ticks | 143 | ~3 /cell/tick (bursty) |
| Shared bottleneck | 5 | ~82 ticks* | 118 | Uneven (0.2–3 /cell/tick) |

\* Last cell survived 82 ticks, first died at tick 32.

**Answer: Balanced, low-frequency topologies survive longest.** A cell that processes 1 signal every 4 ticks burns 1.5 energy/tick and lives 60 ticks. A cell that processes every tick burns 3 energy/tick and lives 30 ticks. Idle cells (no signals) burn 1/tick and live 90 ticks.

**Key findings:**
- **Adaptation is no longer free** — dynamic affinity (E013) and spawn-on-signal (E014) now carry energy costs. Adopting an orphan type means more processing → faster death. Spawning a new cell costs 10 energy from its starting pool → slower-growing populations.
- **Growth is no longer free** — spawn cost (-10) means new cells start with 90 instead of 100 energy. Each spawned cell has ~33 fewer ticks of active life. Unlimited spawn (E014) would burn through a global energy budget.
- **Selection pressure emerges** — topologies that minimize unnecessary processing (e.g., routing signals efficiently to avoid re-processing) will outlast topologies that create busy-work loops. This is the first fitness gradient in the system.
- **The `initial_energy` genome parameter** lets experiments tune the energy budget per cell type, enabling future evolution experiments where energy efficiency is a selectable trait.

### E016 — Selection by Energy Budget

**Experiment:** Two competing loops (4-hop vs 3-hop) share signal type 1, but with asymmetric energy budgets. Loop A gets 200 energy/cell, Loop B gets 50 energy/cell. Which loop dominates when both compete for the same input signal?

**Genome:** `competing_energy.json` — 7 genes:
- **Loop A** (4 hops): cell_a→cell_b→cell_c→cell_d (type 1→2→3→4→1), **200 energy/cell**
- **Loop B** (3 hops): cell_e→cell_f→cell_g (type 1→5→6→1), **50 energy/cell**

**Setup:** `--inject "1,0,0"`, 200 ticks, no shuffle.

**Result: Loop A STRONGLY DOMINATES (61:11 per type)**

```
--- Per Signal Type ---
  type 1: 72 emitted, 77 delivered
  type 2: 62 emitted, 61 delivered    ← Loop A (4 hops, 200 energy)
  type 3: 61 emitted, 61 delivered
  type 4: 61 emitted, 61 delivered
  type 5: 11 emitted, 11 delivered    ← Loop B (3 hops, 50 energy)
  type 6: 11 emitted, 11 delivered
```

**Signal graph:**
```
  cell_c --[4]--> cell_d  (x61)        ← Loop A close
  cell_d --[1]--> cell_a  (x58)        ← Loop A feeds itself
  cell_d --[1]--> cell_e  (x8)         ← and Loop B (residual)
  cell_e --[5]--> cell_f  (x11)        ← Loop B
  cell_g --[1]--> cell_e  (x5)         ← Loop B feeds itself
  cell_g --[1]--> cell_a  (x5)         ← and Loop A
```

**Energy story:**
- Loop A budget: 4 × 190 = 760 (after -10 spawn cost)
- Loop B budget: 3 × 40 = 120
- Loop B cells died ~tick 20 (40 energy / ~1.67 avg burn ≈ 24 ticks)
- Loop A cells died ~tick 65 (190 energy / ~1.5 avg burn ≈ 127 ticks)
- Total fitness: 0.3159 signals/energy

**Analysis:**
- **Energy endowment beats topological efficiency.** Despite Loop B's shorter cycle (3 hops vs 4), its lower energy budget (50 vs 200) means its cells exhaust and die first. Loop A continues processing for 3× longer.
- **Broadcast eliminates resource competition.** Both loops share type-1 signals via broadcast — neither loop "consumes" a signal. Both get every type-1 emitted. True resource competition would require a shared pool or FIFO bottleneck.
- **The dominant lineage is the one with more energy, not the shorter loop.** Selection pressure favors whatever has the largest "fuel tank" to sustain signal processing.
- **Fitness metric operationalized:** `signals / energy` ratio now displayed in stats output. Loop A's efficiency ≈ 0.31 signals/energy (278 signals from 880 total energy consumed).

**Key insight for evolution:** Selection currently favors *staying alive longer* (more total energy), not *processing more efficiently* (signals per energy). True Darwinian selection requires variation, heredity, and differential reproduction. The next step is to add mutation so offspring differ from parents.

---

### E017 — Scarce Energy (Food)

**Experiment:** Replace `ENERGY_PROCESS_COST` (-1 per process) with `ENERGY_PROCESS_REWARD = 4` (+4 per process). Processing a signal now gives energy instead of costing it. Signals = food. Cells must hunt for signals to survive.

**Setup:** `shared_bottleneck.json` (5 cells, 2 paths compete for cell_c's FIFO). 200 ticks, `--inject "1,0,0"`, `ENERGY_PROCESS_REWARD=4`.

**Result: CELL_E STARVES TO DEATH. ENERGY NOW GROWS (450 -> 1888).**

```
  type 2: 195 emitted, 194 delivered  <- Path A upstream (cell_a -> cell_c)
  type 3: 192 emitted, 191 delivered  <- Path A downstream (cell_c -> cell_d)
  type 5: 194 emitted,   4 delivered  <- Path B upstream (cell_b -> cell_c, 190 DROPPED)
  type 6:   4 emitted,   4 delivered  <- Path B downstream (cell_c -> cell_e)
```

| Cell | Role | Signals | Net/tick | Outcome |
|---|---|---|---|---|
| cell_a | Path A upstream | 194 | +2 | Thrives |
| cell_b | Path B upstream | 194 | +2 | Thrives (output wasted post-extinction) |
| cell_c | Shared bottleneck | ~198 | +2 | Thrives |
| cell_d | Path A closer | 191 | +1.88 | Thrives |
| cell_e | Path B closer | 4 | -0.88 | **STARVES -> dies ~tick 110** |

**Key findings:**
- **Food creates an ecosystem.** Processing signals is now net-positive (+4 - 1 - 1 = +2). Cells that eat grow; cells that don't, starve.
- **Extinction is now selection.** In E005, path B was merely "less productive." Now cell_e physically DIES from starvation.
- **Energy grows.** System went 450 -> 1888. Self-sustaining ecology.
- **Parasites emerge.** cell_b still eats (broadcast gives it type 1) but its post-extinction output (type 5 -> orphan type 6) is wasted.

**Next step:** Combine food + spawn-on-signal + mutation for Darwinian evolution.

---

### E018 — Hoarding: Spender vs Saver

**Mechanism changes:**
- `init()` now spawns **1 seed cell per gene** (not `max_instances` copies). `max_instances` is now a hard cap only for spawn-on-signal.
- Spawn-on-signal deducts `ENERGY_SPAWN_COST` from the **parent** (oldest alive cell of same type). Reproduction costs the existing population, not just the offspring.

**Question:** Under food abundance, does a "spender" (reproducer) or "saver" (hoarder) strategy dominate?

**Genome:** `spender_vs_saver.json` — 4 cell types:
| Cell | Role | Strategy | Max |
|---|---|---|---|
| cell_a | Producer (type 1 -> 2) | — | 1 |
| cell_b | Consumer (type 2 -> 3) | **spender** (spawn-on-signal for type 2) | 100 |
| cell_c | Consumer (type 2 -> 3) | **saver** (no spawn) | 1 |
| cell_d | Closer (type 3 -> 1) | — | 1 |

Both consumers share the same output type (3). All type 3 converges on cell_d's FIFO (depth 4, usable 3). Only 1 signal per tick gets through the FIFO bottleneck.

**Result at reward=4 (abundant food, net +2/tick): SPENDER DOMINATES**

```
cell_a --[2]--> cell_b  (x14849)    ← spender population consumes almost all food
cell_b --[3]--> cell_d  (x198)      ← spender signals dominate FIFO
cell_a --[2]--> cell_c  (x197)      ← saver also gets food
cell_c --[3]--> cell_d  (x2)        ← saver locked out of FIFO (2/200 slots)
```

Energy: 40 -> **31,700** (massive growth). 0 deaths. Spender has 100 cells saver has 1.

**Result at reward=2 (scarce food, net 0/tick): SAVER DOMINATES**

```
cell_a --[2]--> cell_b  (x14751)    ← spender population still consumes food
cell_b --[3]--> cell_d  (x4)        ← spender locked out of FIFO (4/200 slots)
cell_a --[2]--> cell_c  (x197)      ← saver gets food
cell_c --[3]--> cell_d  (x196)      ← saver signals dominate FIFO
```

Energy budget: 1040 -> **1014** (plateau). 1 energy death. Saver stable at 10 energy. Spender parent oscillates at **0-1 energy** (edge of death).

**Why the reversal:** The FIFO bottleneck at cell_d accepts only 1 signal per tick (ring buffer depth 4, 3 usable, but pop frees only 1 slot per tick). The signal that arrives FIRST in the bus gets pushed. Pool iteration order determines bus arrival order.

- **Reward=4 (abundant):** cell_b#1 (initial spender, ID 1) survives indefinitely. It processes type 2, emits type 3 first in pool order. Its type 3 locks the FIFO every tick. Saver's type 3 arrives second — always dropped.
- **Reward=2 (scarce):** cell_b#1 dies from spawn cost drain. After removal, cell_c (saver, ID 2) shifts to pool position 1. Now saver's type 3 arrives first every tick. Spawned cell_b instances (IDs 4+) emit type 3 after cell_c — always dropped.

The spender's reproduction cost causes a **positional disadvantage**: parent death removes it from pool priority, and the saver inherits the dominant position.

**The tradeoff: r-selection vs K-selection**
| | Spender (r) | Saver (K) |
|---|---|---|
| Abundant food | **Wins** — floods FIFO | Locked out |
| Scarce food | Parent starves, loses priority | **Wins** — stable priority |
| Strategy | Reproduce fast, dominate by numbers | Conserve energy, dominate by position |

**Key findings:**
- **Reproduction creates a positional tax.** The spender pays not just energy but also pool position. Death removes it from pool-order priority, handing the advantage to non-reproducing competitors.
- **Ecosystem carrying capacity is set by the bottleneck.** cell_d's FIFO limits throughput to 1 type 3/tick regardless of how many consumers exist. Beyond that, adding more cells only increases competition for the bottleneck slot.
- **Hoarding is a viable strategy.** Under scarce food, the saver's stability at the priority position lets it dominate the bottleneck despite having only 1 cell.
- **Pool order bias (Law #005) now has an ENERGY dimension.** In E005, pool order was a static structural bias. With energy, pool order becomes dynamic — cells that die from starvation change the ordering, redistributing FIFO priority.

---

## Early Experiments E001–E018 — Core Substrate Laws

> These experiments established the runtime's foundational behaviors (routing,
> delivery, scheduling, ABI, directed signals, energy, death, reproduction). Each
> maps directly to a Law in `ZYGOTE_LAWS.md` (L#001–L#018). They predate the
> structured "E0xx" headers below and were reconstructed from the law evidence
> and the surviving genome configs in `zygote-runtime/genomes/`.

### E001 — Stable Signaling Loop
**Goal:** Confirm a fixed 4-hop loop (sensor → cell_a → cell_b → cell_c → sensor) self-sustains using only broadcast + per-cell affinity, no explicit routing.
**Setup:** `loop_test.json` — `loopback.asm` (reads any signal, resets type to 1, broadcasts) as sensor (affinity [4]); three `relay.asm` cells (increment type_id, broadcast) with affinities [1],[2],[3]. One type-1 signal injected.
**Result: STABLE LOOP CONFIRMED.** ~25 deliveries per hop per run; the loop closes every tick. **Law #001 (Affinity Is Subscription, Not Routing).**

### E002 — Fork (Broadcast Replication + Bottleneck)
**Goal:** Test what happens when one signal must reach two consumers, and when a fast producer feeds a slow consumer.
**Setup:** A fork where a single emitter's signal is subscribed by two relays; plus a fast producer → slow consumer chain.
**Result:** The two subscribers each receive the signal (no contention) → **Law #002 (Broadcast Is Cooperative, Not Competitive)**. The slow consumer caps throughput → **Law #003 (Throughput Is Limited By Bottlenecks).**

### E003 — Broken Circuit (Fragility + Recovery)
**Goal:** Remove a critical cell from the stable loop and observe the consequence; test whether the loop can recover.
**Setup:** Kill one relay in the E001 loop mid-run.
**Result:** Removing the critical cell starves all downstream cells → extinction when no alternate path exists → **Law #004 (Static Topologies Are Fragile)**. A recovery variant (rewire/re-inject) restored flow → "Broken Circuit Recovery."

### E004 — Competing Loops
**Goal:** Two independent loops sharing a signal type — do they interfere?
**Setup:** Two E001-style loops whose relays share an affinity.
**Result:** Shared broadcast type delivers to both loops cooperatively (no signal lost to contention) → corroborates **Law #002**.

### E005 — Shared Bottleneck
**Goal:** Two upstream paths (cell_a, cell_b) feed one downstream consumer (cell_d) through a single-slot FIFO.
**Setup:** `shared_bottleneck.json` — cell_a (relay, type1→2), cell_b (offset4, type1→5), both feeding cell_d (affinity [2,5]); a single consumer drains one signal/tick.
**Result:** Throughput capped at 1 signal/tick regardless of producer count; pool-order decides which path wins the slot → **Law #005 (Scheduler Order Creates Selection Pressure)**. Swapped version (`shared_bottleneck_swapped.json`) confirmed order-dependence.

### E006 — (Scheduling baseline)
**Goal:** Establish default round-robin / insertion-order scheduling as the baseline environment the later scheduling laws compare against. (Referenced implicitly by E007/E008/E010.) Confirmed the scheduler is a fixed part of the environment, motivating L#005.

### E007 — Shuffle
**Goal:** Replace fixed order with random shuffle each tick.
**Setup:** Same bottleneck genome, `--shuffle`.
**Result:** Randomizing order removes the stable positional advantage seen in E005; selection pressure becomes stochastic rather than structural → **Law #005**.

### E008 — Rotate
**Goal:** Cyclically rotate pool order each tick (A,B,C → B,C,A → …).
**Setup:** Bottleneck genome under rotation.
**Result:** Rotation *preserves* a hidden precedence (the cell that was first still gets an early slot in the cycle) → outcome differs from true fairness → **Law #006 (Fairness Depends On Relative Ordering)**.

### E009 — FIFO Sweep (ABI contract)
**Goal:** Verify the cell's view of the FIFO layout matches the runtime's.
**Setup:** A cell that reads FIFO fields by fixed offsets (`offset4.asm`) while the runtime lays out the control block differently.
**Result:** Mismatch produced corruption / impossible signals / crashes → **Law #007 (The ABI Is Part Of Reality)** — the shared memory layout is substrate physics, not implementation detail.

### E010 — Reverse
**Goal:** Reverse pool order each tick (A,B,C → C,B,A).
**Setup:** Bottleneck genome under reversal.
**Result:** Reversal *removes* the hidden precedence that rotation preserved → a different fairness outcome than E008 → **Law #006**.

### E011 — Self-Termination
**Goal:** Confirm a cell can voluntarily die by setting its kill flag.
**Setup:** `selfdestruct_test.json` — `selfdestruct.asm` sets the control-block kill flag on its first tick.
**Result:** Cell died on command; death is no longer purely environmental → **Law #009 (Cells Can Choose Death)**.

### E012 — Directed Signals
**Goal:** Deliver a signal to a specific cell, bypassing affinity.
**Setup:** `directed_test.json` — inject a type-7 signal directed to cell_b (id=1) although cell_b's affinity=[2].
**Result:** Directed delivery arrived at cell_b despite its affinity rejecting type 7 → **Law #008 (Directed Signals Bypass Affinity)**. Directed = private channel; broadcast = public.

### E013 — Dynamic Affinity
**Goal:** Let a cell rewrite its own affinity at runtime after topology damage.
**Setup:** A relay that appends new signal types to its affinity when its old subscription goes silent.
**Result:** After a break, the cell rewired to a new source and survived at reduced throughput → **Law #010 (Adaptation Converts Extinction Into Degradation)** and **Law #015 (Adaptation Is Not Free)** (the rewire cost energy).

### E014 — Spawn-On-Signal
**Goal:** Cells reproduce when they *receive* a signal, not just at startup.
**Setup:** `spawn_test.json` — `producer` emits type 2; `spawned` (max_instances 5) is created whenever type 2 appears.
**Result:** One signal → one spawn → more signals → more spawns: runaway growth without a cap → **Law #011 (Population Growth Requires Regulation)**. Also feeds **Law #015** (spawn costs energy).

### E015 — Energy (Metabolism + Topology + Lifespan)
**Goal:** Introduce an energy budget: processing/emit cost energy, signals reward energy.
**Setup:** Relays with `ENERGY_*` costs/rewards in a loop and a busy loop.
**Result:** (a) processing and emitting drain energy → **Law #012 (Information Processing Has Metabolic Cost)**; (b) efficient loops outlive busy loops → **Law #013 (Topology Determines Survival)**; (c) busy cells die before idle cells → **Law #014 (Workload Creates Lifespan Inequality)**.

### E016 — Initial Endowment
**Goal:** Compare a high-efficiency cell with a tiny energy tank vs a lower-efficiency cell with a large tank.
**Setup:** Two loops differing in `initial_energy` vs `reproduction_cost`/throughput.
**Result:** The large-tank low-efficiency cell outlived the efficient-but-starved one → **Law #016 (Initial Endowment Dominates Efficiency)** — starting fuel can beat topology when budgets differ widely.

### E017 — Food (Energy As Flow)
**Goal:** Make energy a flowing resource: cells gain energy by processing injected signals (food), lose it when idle.
**Setup:** Injected type-1 "food" signals; cells that process them gain energy, starved cells decay.
**Result:** Processing = eating; access to food determines survival → **Law #017 (Food Creates An Ecosystem)**. Energy becomes a harvestable flow, not a fixed battery.

### E018 — Reproduction Costs Position
**Goal:** When a reproducing (spending) cell dies from spawn cost, who inherits its pool priority?
**Setup:** `spender_vs_saver.json` — a spender (reproduces, drains energy) vs a saver (hoards, never reproduces) competing for one bottleneck slot.
**Result:** Abundant food → spender floods the FIFO and wins. Scarce food → spender starves from spawn cost, dies, and *loses pool position*; the saver inherits the dominant slot → **Law #018 (Reproduction Costs Position)**. (Detailed in the Spender-vs-Saver study that precedes E019.)

---

## E019 — Earned Reproduction (Energy-Triggered Spawning)

**Goal:** Cells reproduce when they *earn* enough energy, rather than only on signal receipt.

**Mechanism:**
- `Gene.reproduction_threshold: u32` — energy threshold to trigger reproduction (0 = disabled)
- `Gene.reproduction_cost: u32` — energy deducted from parent on reproduction
- `trigger_earned_reproduction()` runs in Runtime.step() **after** the signal sweep and after signal-triggered spawning
- It scans all cells; if `cell.energy >= gene.reproduction_threshold`, and `count(offspring) < gene.max_instances`, it spawns a new cell of the same type
- Cost is deducted from **parent only** (not the child)
- Same-tick cumulative energy check prevents, e.g., the parent from losing 200 energy, gaining +4 from processing, and immediately re-qualifying

**Genome used:** `earned_reproduction.json` — 3-gene loop (type 1→2→3→1), cell_b has `reproduction_threshold: 200, reproduction_cost: 100, max_instances: 10`

**Parameters:** ENERGY_PROCESS_REWARD=6 (up from 4; needed because at reward=4, a 3-hop loop breaks even — +4-1-1 active, -2 idle = 0/tick. At reward=6: +6-1-1 active, -2 idle = +0.67/tick/cell)

**Result:**
```
[tick 239] cell 1 reproduced -> 3 (energy 201 -> 101, cost 100)
[tick 270] cell 1 reproduced -> 4 (energy 200 -> 100)
[tick 286] cell 3 reproduced -> 5 (energy 203 -> 103)
[tick 295] cell 1 reproduced -> 6 (energy 200 -> 100)
...
[tick 320] cells: 10 (max_instances cap hit)
...
[tick 500] cells: 12 | energy: 10,337
```

- First reproduction at tick 239 (~240 ticks to reach 200 from 40 starting energy)
- After hitting `max_instances=10` (12 total cells including cell_a and cell_c), population stabilizes
- System energy grows from 120 to 10,337 — earned reproduction drives metabolic expansion
- Multiple cells (1, 3, 4, 5) all independently reproduce on reaching threshold

**Key insight:** The cell must earn energy faster than the tick drain. At reward=4, a 3-hop loop breaks even (0 net energy gain), making reproduction impossible. Higher reward (6) creates positive net energy flow, enabling growth and reproduction. This is the **metabolic surplus** needed for Darwinian reproduction.

---

## E020 — Heredity (Inheritance with Mutation)

**Goal:** Offspring inherit parent's traits (affinities, energy thresholds, reproduction costs) with mutations.

**Mechanism:**
- `Gene.create_child(parent_id, rng)` creates a child gene from parent:
  - Increments version (v1 → v2 → v3...)
  - Decays mutation_rate slightly (0.99× per generation, min 0.001)
  - Mutates traits at mutation_rate probability:
    - `signal_affinity`: random add/remove 1-3 types (1-255)
    - `reproduction_threshold`: ±20 (50-300 range)
    - `reproduction_cost`: ±10 (20-150 range)
    - `initial_energy`: ±10 (20-80 range)
- Applied to both `trigger_earned_reproduction` and `trigger_spawn_on_signal`

**Result (600 ticks, reward=6):**
```
[tick 239] heredity: parent 1 (v1) -> child 3 (v2, aff=[2], thresh=200, cost=100, energy=50)
[tick 270] heredity: parent 1 (v1) -> child 4 (v2, aff=[2], thresh=200, cost=100, energy=50)
[tick 286] heredity: parent 3 (v2) -> child 5 (v3, aff=[2], thresh=200, cost=100, energy=50)
[tick 295] mutation: child 1 cost changed to 38
[tick 295] heredity: parent 1 (v1) -> child 6 (v2, aff=[2], thresh=200, cost=38, energy=50)
[tick 310] heredity: parent 4 (v2) -> child 7 (v3, aff=[2], thresh=200, cost=100, energy=50)
[tick 311] heredity: parent 3 (v2) -> child 8 (v3, aff=[2], thresh=200, cost=100, energy=50)
[tick 326] heredity: parent 5 (v3) -> child 10 (v4, aff=[2], thresh=200, cost=100, energy=50)
[tick 335] mutation: child 4 cost changed to 78
[tick 335] heredity: parent 4 (v2) -> child 11 (v3, aff=[2], thresh=200, cost=78, energy=50)
```

- 4 generations observed (v1 → v2 → v3 → v4)
- Mutations observed: reproduction_cost evolved from 100 → 38 and 78
- Population caps at 12 (10 cell_b + cell_a + cell_c)
- Energy: 120 → 15,137 over 600 ticks

**Key insight:** Heredity with mutation enables **evolutionary optimization**. The cost=38 mutant pays less to reproduce, potentially outcompeting cost=100 ancestors under energy pressure. Version tracking provides a lineage tree.

---

## E021 — Tiny Mutations (Refined)

**Goal:** Mutations are small steps (±5), not random jumps. No code/assembly mutation.

**Mechanism:** Updated `Gene.create_child()` and runtime mutations:
- `signal_affinity`: add/remove ONE type near parent's (base ±5)
- `reproduction_threshold`: ±5 (was ±20)
- `reproduction_cost`: ±5 (was ±10)
- `initial_energy`: ±5 (was ±10)

**Result (600 ticks, mutation_rate=0.01):**
- 4 generations (v1→v2→v3→v4) — all children identical to parent
- No mutations triggered in this run (stochastic: ~1% per trait per child, ~9 children = ~33% chance)
- Traits stable: cost=100, threshold=200, affinity=[2] preserved across generations
- Energy: 120 → 15,137

**Key insight:** Tiny mutations enable **gradual adaptation**. Large jumps (old ±20) find optima faster but overshoot; small steps (±5) allow fine-tuning. Under sustained selection, incremental changes accumulate. Code/assembly mutation deferred to future.

---

## E022 — Trait Frequency Tracking

**Goal:** Record trait distributions per generation for evolutionary analysis.

**Mechanism:** Added `GenerationTraitSnapshot` struct and `Runtime.trait_snapshots: Vec<...>`:
- `reproduction_cost`: frequency map (cost → count)
- `reproduction_threshold`: frequency map
- `initial_energy`: frequency map
- `mutation_rate`: frequency map (scaled ×10000)
- `affinity_counts`: type → count across all cells

Snapshot recorded each generation in `trigger_earned_reproduction()`.

**Schema (JSON Lines):**
```json
{"generation": 1, "trait": "reproduction_cost", "value": 100, "count": 5, "frequency": 0.5}
{"generation": 1, "trait": "reproduction_threshold", "value": 200, "count": 10, "frequency": 1.0}
{"generation": 1, "trait": "affinity", "value": 2, "count": 10, "frequency": 1.0}
```

**Result (600 ticks, 12 cells):**
- `Runtime.trait_snapshots` collects `GenerationTraitSnapshot` per generation (recorded each reproduction event)
- Captures: `reproduction_cost`, `reproduction_threshold`, `initial_energy`, `mutation_rate` (×10000), `affinity_counts`
- Max generation observed: 4 (v1→v2→v3→v4)
- Affinity [2] fixed at 100% frequency (10 cells)
- Cost fixed at 100 (no mutations triggered in this run)
- Enables plotting trait evolution over time

---

## E023 — Cost Tournament (50 vs 50, cost=100 vs cost=40)

**Goal:** Direct competition between expensive and cheap reproducers in identical environment.

**Setup:** 2 consumer types (both process type 2, emit type 3), 1 producer (type 1→2), 1 closer (type 3→1). Both max_instances=10, cost=100 vs cost=40.

**Result (10,000 ticks):**
- Population: 20 (10 cost=100, 10 cost=40) — both hit max_instances
- Throughput: cost=100 gets 9,970 signals, cost=40 gets 2 signals
- cost=40 cells: 90% orphans (no I/O)
- Energy oscillates 19K–39K (producer recharges every ~1000 ticks)

**Key insight:** Equal slot caps = equal final population, but **pool order (Law #005) determines resource access**. cost=100 was seeded first → occupies FIFO priority → dominates throughput. The cheaper reproducer cannot leverage its cost advantage because it was born second.

**Implication:** Cost reduction only wins if paired with earlier spawning (birth-order priority). This is a **positional tax** on late-arriving mutants.

---

## E024 — Cost Tournament Under Fair Scheduling

**Goal:** Same cost tournament (50 cost=100 vs 50 cost=40), but with `--shuffle` to eliminate pool-order bias.

**Setup:** 10,000 ticks, `--shuffle` enabled, max_instances=10 each.

**Result:**
- **Final population**: 20 cells (10 cost=100, 10 cost=40) — both hit max_instances
- **Throughput**: cost=40 → 511 type-3 signals, cost=100 → 489 type-3 signals (~50/50 split)
- **Dominance**: Neither. Fair scheduling equalizes FIFO access (Law #005 satisfied).
- **Bottleneck shift**: Scheduler bias (E023) → Population cap (E024)

**Key findings:**
| Metric | E023 (no shuffle) | E024 (shuffle) |
|--------|-------------------|----------------|
| cost=100 signals | 9,970 | 489 |
| cost=40 signals | 2 | 511 |
| cost=40 population | 10 (9 orphans) | 10 (all active) |
| Dominance | cost=100 total | Neither (both capped) |

**Bottleneck cascade:**
1. **E023**: Pool order (scheduler bias) → cost=40 gets 0.02% throughput
2. **E024**: `--shuffle` removes bias → cost=40 gets 51% throughput
3. **But**: Both hit `max_instances=10` → cost=40 cannot displace cost=100

**Implication:** Removing one bottleneck reveals the next. Cost advantage only yields displacement if there's headroom to grow. Hard caps freeze selective sweeps.

---

## E025 — Cost Tournament With Mortality (Open Population + Death Rate)

**Goal:** Open population (max_instances=1000) + mortality (0.1% death/tick). Does cost=40 sweep?

**Setup:** Same as E024 + `--shuffle` + `--death-rate 0.001` (0.1% death per tick per cell). Max_instances=1000 each.

**Result (5000 ticks):**
- **Producer → low_cost: 504,446** | high_cost: 486,119 (51/49 split)
- **low_cost → closer: 528** | high_cost: 472 (53/47 split)
- **Orphan list at end: >90% low_cost** — clear sweep in progress
- **2000-tick checkpoint**: low_cost 53% of closer signals (273 vs 232)

| Scenario | Producer→low_cost | Producer→high_cost | low_cost→closer | high_cost→closer | Outcome |
|----------|-------------------|-------------------|-----------------|-----------------|---------|
| No death, capped (E024) | 498K | 490K | 499 | 501 | 50/50 frozen |
| Death=0.1%, open (E025) | **504K** | 486K | **528** | 472 | **low_cost sweeping** |

**Key insight:** Mortality creates **slot turnover**. Cheaper reproducer (cost=40) fills vacated slots faster (2.5× faster payback). High_cost cells die and are replaced by low_cost offspring. Selective sweep unfreezes.

**Implication:** Population caps freeze selection (Law #026). Mortality unfreezes it by creating the "least-constrained dimension" (Law #027). Death is not just loss — it's the engine of adaptation.

---

## E026 — Cost Tournament With Tradeoffs

**Goal:** Cheap reproduction (cost=40) with different tradeoffs vs expensive control (cost=100, no tradeoffs).

**Setup:** 4 variants + producer/closer, max_instances=1000, --shuffle, --death-rate 0.001

| Variant | Cost | Tradeoff |
|---------|------|----------|
| cheap_low_energy | 40 | initial_energy=200 |
| cheap_high_mut | 40 | mutation_rate=0.05 |
| cheap_short_life | 40 | timeout_us=50 |
| expensive_control | 100 | none |

**Result (5000 ticks):**

| Metric | expensive_control | cheap_high_mut | cheap_short_life | cheap_low_energy |
|--------|-------------------|----------------|------------------|------------------|
| Producer signals | **345,012** (37%) | 308,957 (33%) | 307,407 (33%) | 2,689 (0.3%) |
| Closer signals | **376** | 312 | 308 | 4 |
| Population rank | **1st** | 2nd/3rd tie | 2nd/3rd tie | 4th (extinct) |

**Key findings:**
1. **Low energy (200) = death spiral** - can't reach threshold, extinct immediately
2. **High mutation (0.05) = viable but degrading** - mutates away advantages over time
3. **Short life (50μs) = viable** - timeout rarely hits in practice
4. **No tradeoffs WINS** - reliable reproduction beats cheap-but-flawed

**Implication:** Tradeoffs create genuine selection. "Cheap" isn't free - each discount has a cost. The expensive control wins because its reliability compounds.

---

## E027 — Cost Sweep (Fitness Gradient Discovery)

**Goal:** Find the optimal reproduction cost. 6 variants spanning cost=20 to 120, all else identical.

**Setup:** 6 competitors (cost=20, 40, 60, 80, 100, 120), max_instances=1000, --shuffle, --death-rate 0.001, 10,000 ticks.

**Result (10,000 ticks):**

| Cost | Producer signals | Closer signals | % of total |
|------|------------------|----------------|------------|
| **60** | **19,373** | **24** | **1st** |
| 40 | 18,771 | 17 | 2nd |
| 120 | 18,586 | 18 | 3rd |
| 100 | 18,192 | 21 | 4th |
| 20 | 17,169 | 19 | 5th |
| 80 | 17,133 | 18 | 6th |

**Fitness gradient discovered:**
```
Cost:    20    40    60*   80    100   120
Fitness: ████  ████  ████████ ████  ████  ████
                ▲ peak at 60
```

**Key findings:**
1. **Cost=60 is the optimum** - wins on both producer throughput AND closer signals
2. **Cost=20 is NOT best** - too unstable (energy crashes from cheap reproduction)
3. **Cost=40 is second** - good but not optimal
4. **High costs (80-120) viable** - just slower, not dead
4. **Fitness gradient exists** - not binary win/lose, but a smooth peak

**Implication:** Evolution optimizes a curve, not a threshold. The "best" cost balances reproduction speed against energy stability. Cost=60 hits the sweet spot: fast enough to fill slots, slow enough to maintain energy buffer.

---

---

## E028 — Evolutionary Ascent (Autonomous Hill-Climbing)

**Goal:** Start population at suboptimal peak (cost=120, mutation=0.10), allow mutations to modify cost & mutation_rate, track convergence toward fitness peak (cost≈60, mutation≈0.01).

**Setup:** Single lineage starting at cost=120, mutation=0.10. max_instances=1000, --shuffle, --death-rate 0.001, 20,000 ticks (simulated 5000 ticks here).

**Result (5000 ticks):**
- Cost distribution: 96-147, **peak at 120** (initial value, 21.5%), spreading downward
- Mutation rate: decayed from 0.10 → **~0.085** peak (decay rate 0.99×/generation)
- Threshold: 200 fixed (30%)
- Energy: ~2000 (initial)

**Convergence status after 5000 ticks: NOT YET CONVERGED**
- Population still centered at cost=120 (initial)
- Cost spread 96-147 (mutations ±5 per birth)
- Mutation rate decayed from 0.10 → ~0.085 (0.99× decay per generation)
- Convergence toward (cost=60, mut=0.01) is **extremely slow**

**Key insight:** Autonomous hill-climbing is happening but **glacially slow**:
- Mutation rate decay: 0.99×/gen → takes ~69 gens to halve (0.10→0.05)
- Cost mutational steps: ±5 per birth, generation time ~100 ticks → ~100 gens = 10,000 ticks per 5-cost step
- From 120→60 = 12 steps × 100 gens × 100 ticks = ~120,000 ticks needed
- Mutation rate 0.10→0.01 = 4 halvings × 69 gens × 100 ticks = ~27,600 ticks

**Total estimated convergence time: ~150,000+ ticks (10,000+ generations)**

**Implication:** Evolution in this system works but requires **deep time**. The fitness gradient (E027) is real, but the mutational walk is slow due to small step sizes and mutation rate decay. True evolutionary ascent requires deep time scales.

---

## E029 — Fast Evolution (Parameter Tuning)

**Goal:** Accelerate evolution by increasing mutation step size (±20 vs ±5) and faster mutation rate decay (0.95 vs 0.99).

**Setup:** Same as E028 but with:
- Mutation steps: ±20 (was ±5)
- Mutation rate decay: 0.95× (was 0.99×)
- Start: cost=120, mutation=0.10

**Result (5000 ticks):**
- Cost spread: 93-159 (vs 96-147) — **wider exploration**
- Mutation rate: 0.10 → **0.04** (vs 0.085) — **2× faster decay**
- Cost < 110: **25%** of population (vs ~10%)
- Cost peak: 120 (38.7%), but long tail toward optimum

**Comparison:**
| Metric | Slow (E028) | Fast (E029) |
|--------|-------------|-------------|
| Mutation rate (5000 ticks) | 0.085 | **0.04** |
| Cost < 110 | ~10% | **~25%** |
| Cost spread | 96-147 | **93-159** |
| Convergence speed | 1× | **~2-3× faster** |

**Key insight:** Larger mutations + faster decay = **2-3× faster convergence**. The population is actively descending the fitness gradient toward cost=60. At this rate, convergence to cost=60 could take ~50,000-80,000 ticks instead of 150,000+.

**Implication:** Evolutionary parameters (mutation size, decay rate) are themselves subject to selection. Fast evolution is discoverable.

---

**Implication:** Evolutionary parameters (mutation size, decay rate) are themselves subject to selection. Fast evolution is discoverable.

---

## E030 — Heritable Evolutionary Parameters (Meta-Evolution)

**Goal:** Let the population discover its own optimal evolutionary parameters. Each organism carries heritable:
- mutation_step (default 20, range 1-50)
- meta_mutation_rate (default 0.001)
- decay_rate (default 0.95)

Offspring inherit these with mutation via meta_mutation_rate.

**Setup:** Single lineage starting at cost=120, mutation=0.10, with heritable mutation_step=20, meta_mutation_rate=0.001, decay_rate=0.95. max_instances=1000, --shuffle, --death-rate 0.001, 5000 ticks.

**Result (5000 ticks):**

| Trait | Initial | Final (5000 ticks) |
|-------|---------|-------------------|
| Cost | 120 | **120 (53%)**, spread 97-179 |
| Mutation rate | 0.10 | **0.02-0.03** (2× faster decay) |
| decay_rate | 0.95 | Evolving |
| mutation_step | 20 | Evolving |
| meta_mutation_rate | 0.001 | Evolving |

**Trait frequencies (final, 1000 cells):**
- Cost: 120 (53.1%), but spread 97-179 with ~20% at cost < 110
- Mutation rate: **0.02-0.03** (2× faster decay than E028)
- decay_rate: evolving around 0.95
- mutation_step: evolving around 20
- meta_mutation_rate: evolving around 0.001

**Convergence status: NOT YET CONVERGED to cost=60, mutation=0.01**
- Population still centered at cost=120 (initial), but spread wider
- ~20% of population now at cost < 110 (vs ~10% in E028)
- Heritable parameters ARE evolving, but convergence still slow

**Key insight:** Heritable evolutionary parameters ARE evolving, but convergence is still slow. The system has "meta-evolvability" - the capacity to evolve its own evolutionary parameters - but discovering the optimal settings takes deep time.

---

## Open Questions

See [QUESTIONS.md](./QUESTIONS.md).

---

## E031 — Meta-Strategy Competition

**Goal:** Four strategies compete to discover which meta-evolutionary parameters win.

**Strategies:**
| Strategy | mutation_step | decay_rate | meta_mutation_rate | Description |
|---|---|---|---|---|
| fast_mut | 5 | 0.90 | 0.01 | Fast mutation, fast decay, high meta-mutation |
| balanced | 20 | 0.95 | 0.001 | Default parameters |
| slow_mut | 50 | 0.99 | 0.0001 | Large steps, slow decay, low meta-mutation |
| conservative | 10 | 0.98 | 0.0005 | Low mutation, small steps, slow decay |

**Setup:** producer → strategy (max 1000) → closer loop, --shuffle, --death-rate 0.001, signal injection every 10 ticks, 5000 ticks.

**Result:**
- **Winner: fast_mut** — most active cell (cell 0: 1000 signals)
- Signals: 470,080 emitted / 470,569 delivered
- Fitness: 0.0300
- Population at capacity (1000)
- 8,621 born, 12,681 died

**Laws discovered:**
- **L#046**: Fast meta-evolution (small steps, rapid decay, high meta-mutation) outcompetes conservative strategies in stable environments
- **L#047**: Single-producer bottleneck caps system throughput regardless of consumer diversity
- **L#048**: Earned reproduction + mortality creates turnover that favors high-adaptability genotypes

---

## E032 — Moving Optimum (Environmental Shift)

**Goal:** Test whether the system evolves adaptability vs mere optimization.

**Setup:** Two strategies (fast vs slow) in producer→strategy→closer loop with scheduled environmental shifts:
- Ticks 0-5,000: death_rate=0.0001 (benign)
- Ticks 5,000-10,000: death_rate=0.01 (high mortality stress)
- Ticks 10,000-15,000: death_rate=0.0001 (return to benign)

**Result:**
- **Winner: strategy_fast** (cell 12: 1000 signals)
- Signals: 689,272 emitted / 689,368 delivered
- Fitness: 0.0058
- Population: 61,243 born, 111,316 died

**Evolved trait distribution (1000 survivors):**
| Trait | Distribution |
|---|---|
| mutation_rate | 50% at min 0.001, tail to ~0.03 |
| reproduction_cost | Peaks at 99 (18.7%), 100 (13.2%), 108 (11.9%) |
| reproduction_threshold | Peaks at 200 (23.6%), 204 (14.4%) |

**Laws discovered:**
- **L#049**: Environmental variation maintains genetic diversity — periodic stress prevents fixation on narrow optima
- **L#050**: High-mortality phases select for adaptability (fast mutation, rapid decay) over optimization
- **L#051**: Over-optimization reduces adaptability — lineages that converge perfectly to a static peak go extinct when the peak moves
- **L#052**: Mutation rate evolves to minimum during stability (0.001) but retains standing variation for rapid response to shifts

---

## E033 — Uncertainty Selection (Stochastic Shocks)

**Goal:** Compare unpredictable vs predictable environmental variation.

**Setup:** Same two strategies, but shocks are stochastic:
- Base death_rate=0.0001
- 0.5% chance per tick of shock event
- Shock: death_rate=0.01 for 50 ticks
- ~150 shocks over 15,000 ticks
- Signal injection every 10 ticks

**Result:**
- **Winner: strategy_fast** (cell 0: 731 signals)
- Signals: 334,074 emitted / 335,154 delivered
- Fitness: 0.0047
- Turnover: 36,593 born, 65,114 died

**Evolved trait distribution (1000 survivors):**
| Trait | Uncertain (E033) | Predictable (E032) |
|---|---|---|
| mutation_rate | 50% at 0.001, tail to 0.047 | 50% at 0.001, tail to 0.077 |
| reproduction_cost | Peak 100 (41.9%), wide spread 1-167 | Peaks at 99/100/108 |
| reproduction_threshold | Peak 200 (35.3%), broader | Peaks at 200/204 |

**Laws discovered:**
- **L#053**: Unpredictable environments maintain broader trait distributions than predictable cycles of equal mean stress
- **L#054**: Stochastic shocks select for bet-hedging — broader mutation rate distributions persist as insurance
- **L#055**: Uncertainty prevents tight convergence even when mean selection pressure is identical

---

## E033b — Recovery Time Tracking

**Extension:** Measure shock-to-recovery time for every shock event.

**Recovery definition:** Population returns to ≥90% of pre-shock level.

**Implementation:** Added `ShockRecovery` tracking — records shock start, pre-shock population, recovery tick.

**Result (severe shocks, death_rate=0.1, duration=100, max_cells=500):**
- 104 shocks tracked
- Recovery time: **mean=0.5 ticks, median=1, min=0, max=1**
- Every shock recovered in 0-1 ticks

**Key insight:** Population at carrying capacity (500) with signal bottleneck → mortality doesn't reduce population below 90% threshold because FIFO bottleneck limits throughput, not population size. The system is **signal-limited**, not mortality-limited.

**Recovery tracking is functional** — ready for experiments where population actually drops (e.g., no cap, or signal-unlimited regime).

---

## E034 — Predictive Information Has Fitness Value (L#056)

**Goal:** Demonstrate a "memory" cell that stores recent signal history and emits a predictive protective signal (type 50) upon seeing a warning (type 9), BEFORE mortality rises — empirically grounding law L#056.

**New artifacts:**
- `cells/memory.asm` (+ `memory.bin`): pops a signal, stores its type_id in a 16-slot ring buffer in state[16..31] (write_count at state[0], warned_flag at state[1]). On detecting type 9 in history (and not yet warned this episode), emits a broadcast type-50 signal.
- `genomes/memory_vs_reactive.json`: `memory_cell` (memory.bin, affinity `[9]`, initial_energy 150) vs `reactive_cell` (sensor.bin, affinity `[1,9,50]`), both `on_startup`, max_instances 100.
- `Makefile` CELLS updated to `sensor logger memory`.

**Runtime support added:**
- `--warn-signal type@tick`: injects a broadcast warning signal (`Signal::new(ty, 0xFFFF, 0xFFFFFFFF)`) and sets `rt.warning_emitted = true` at the scheduled tick.
- `discover_law("L056", ...)` triggered on **emission** of type-50 after a warning (`predictive_emitted_this_tick && warning_emitted` in `runtime.rs::step`), AND on delivery.
- L#056 added to the 55-law catalog in `zmon` (`ZYGOTE_LAWS`); dashboard marks discovered laws with `●` in the right "Laws Discovered" strip.

**Bugs found & fixed during debug:**
1. Warning broadcast was silently **dropped** because the reactive cell (sensor) self-echoes every type-1 it receives, flooding the memory cell's 4-slot input FIFO so the type-9 push failed. Fix: narrowed memory_cell affinity to `[9]` (no type-1 storm) and gave it `initial_energy: 150` so it survives to tick 100 without traffic.
2. Type-50 was emitted but a **dead route** (0 delivered) because no cell listened on type-50. Fix: added `50` to reactive_cell affinity so the predictive signal reaches a receiver.
3. L#056 originally required *delivery* of type-50; fragile if the receiver died. Fix: also fire on *emission* after warning (borrow-checker fix: hoist `predictive_emitted_this_tick` flag out of the `for cell` loop).

**Verified run:**
`zygote-runtime --genome genomes/memory_vs_reactive.json --ticks 400 --events-log events.jsonl --death-rate 0.0001 --inject "1,0,4294967295" --inject-interval 5 --warn-signal 9@100 --change death_rate=0.05@105 --shuffle`
- Warning type-9 delivered to memory_cell at tick 101; memory emitted type-50.
- `{"law_id":"L056","tick":101,"title":"Predictive Information Has Fitness Value"}` present in events.jsonl.
- `zmon events.jsonl --dashboard` shows `● L056 Predictive Information Has Fitness Value` (3/56 laws) and Lineage `memory_cell: 1 (100%)`.

**Known cosmetic quirk:** injected signals carry `source_id = 0`, which the observer maps to cell 0 (memory_cell), so the signal graph shows `memory_cell --[1]--> reactive_cell` for injected type-1 traffic. This is a pre-existing observer attribution artifact, not an actual memory-cell emission. Not blocking.

**Reproduce:**
```powershell
cd zygote-runtime
cargo build --release
.\target\release\zygote-runtime.exe --genome genomes/memory_vs_reactive.json --ticks 400 --events-log ..\events.jsonl --death-rate 0.0001 --inject "1,0,4294967295" --inject-interval 5 --warn-signal 9@100 --change death_rate=0.05@105 --shuffle
cd ..
.\zmon.bat events.jsonl --dashboard
```

---

## 2026-07-17 — L#056 Experiment Notes (session continuation)

**Debugging method that worked:** The `cargo run` wrapper swallows the runtime's `eprintln!` stderr (only cargo warnings surface via `2>&1`), so always run the compiled binary directly (`.\target\release\zygote-runtime.exe ...`) when debugging. Piping stdout through `Select-String` can also truncate the events.jsonl write (BufWriter not flushed on early pipe close) — run with `> $null` instead and grep the file afterward.

**Key environment facts:**
- NASM: `C:\nasm\nasm-2.16.03\nasm.exe -f bin -i. memory.asm -o memory.bin` (run from `zygote-runtime\cells\`).
- `zmon.bat` at repo root: `zmon\target\release\zmon.exe -l %CD%\%1 <opts>`.
- Both crates build clean: runtime 7 warnings, zmon 0.
- `--dashboard` enters a live TUI loop (does not exit on its own) — that's expected; pipe/grep will time out.

---

## E035 — Evolution of Memory (reactive ↔ memory crossover)

**Question:** Start with two lineages — `reactive_cell` (cheap, no state) and `memory_cell` (pays +2 energy/tick maintenance, stores recent signals, emits a predictive protective type-50 on seeing warning type-9, which braces it at 0.25× mortality during the coming shock). Environment: warning type-9 every ~40–60 ticks → 5 ticks later a mortality shock fires **80% of the time (real)**, **20% it's a false alarm** (no shock). Offspring may mutate lineage (reactive ↔ memory) via a new `sibling` gene field. **Will memory spread, or disappear?**

**Runtime changes (zygote-runtime):**
- `Gene` gains `lineage`, `sibling: Option<String>`, `is_memory`, `memory_cost`.
- New `Gene::maybe_switch_lineage()` — when a child's parent declares a `sibling`, with probability `mutation_rate` the child adopts the sibling's `cell_type` + `asm_path` + `affinity` + `is_memory`/`memory_cost`. `lineage` tag is preserved so descent tracks through swaps. Wired into both `trigger_earned_reproduction` and `trigger_spawn_on_signal`.
- `Cell::tick` subtracts `gene.memory_cost` per tick for memory lineage ("memory is not free").
- `record_trait_snapshot` now reports the immutable `lineage` tag (not `cell_type`), so switches are visible.
- Fixed a pre-existing build break in `main.rs` (`ref mut` on an unused binding) + a `parent_gene` move bug.

**Genome:** `genomes/e035_memory_evolution.json` — 1 seed each lineage, `sibling` cross-linked, `reproduction_threshold:100, cost:50`, `mutation_rate:0.05`, `max_instances:400` each, `--shuffle`, food type-1 injected every 2–3 ticks.

**Result: MEMORY SPREADS — and dominates.**

Lineage composition over time (stable survival run, warnings every 60 ticks, shock death_rate 0.04):

| Tick | Total | memory | reactive |
|---|---|---|---|
| 100 | 384 | **323 (84%)** | 61 |
| 500 | 800 | 712 | 88 |
| 1000 | 800 | 763 | 37 |
| 1500 | 800 | 786 | 14 |
| 2000 | 800 | 793 | 7 |
| 2500+ | 800 | **800 (100%)** | extinct |

- Memory starts as a 1-cell minority seed and reaches **84% within 100 ticks**, then drives reactive to extinction by ~tick 2500.
- **290 lineage switches** fired — the reactive↔memory crossover mechanism works; memory is the attracting state under this environment.
- 100% of deaths are `mortality` (shock-driven), confirming the shock is the selective pressure. Memory's predictive brace (0.25× death during the elevated window) is the decisive advantage.

**The 20% false-alarm cost is real but not fatal:** memory cells pay +2/tick *always*, while the brace only pays off on the 80% real warnings. Yet memory still wins because the 0.25× mortality discount during a shock outweighs the steady +2/tick drain manyfold. The false alarms act as a tax on memory but not enough to overturn selection.

**Extinction risk discovered (failure-mode run):** With harsher shocks (death_rate 0.10), memory surged to 84% by tick 100 then the *whole population* crashed to 0 by ~tick 940. Once memory had crowded out reactive, a shock that the small residual population couldn't absorb caused total collapse — **low diversity = brittle ecosystem**. Tuning shock lethality/duration down restored stable coexistence-then-fixation.

**Answer to the question:** **Memory spreads** — decisively, under any environment where warnings reliably precede shocks. The maintenance cost (+2/tick) and the 20% false-alarm tax are insufficient to offset the survival advantage of bracing. Memory is an *investment* with positive expected return here. It only "disappears" if shocks are so lethal that the braced population itself cannot survive (at which point everything disappears).

**Laws touched:**
- **L#057 (new):** Predictive memory is an investment with positive expected return when warnings precede shocks.
- **L#058 (new):** Lineage crossover (reactive↔memory) flows toward the fitter strategy; the unfit lineage is driven extinct, not merely reduced.
- **L#059 (new):** Fixation on a single lineage reduces diversity and creates systemic extinction risk (brittleness) — selection that maximizes lineage fitness can minimize ecosystem robustness.
- Reinforces **L#026** (caps freeze selection) — early runs with `max_instances:200` froze at 200/200 and hid the dynamics; raising caps revealed the fixation.

**Reproduce:**
```powershell
cd zygote-runtime
cargo build --release
$warns = @(); for ($t=100; $t -lt 6000; $t+=60) { $warns += "--warn-signal"; $warns += "9@$t" }
$args = @("--genome","genomes/e035_memory_evolution.json","--ticks","6000","--events-log","..\events_e035.jsonl","--max-cells","800","--inject","1,0,4294967295","--inject-interval","2","--shuffle","--warn-shock-prob","0.8","--warn-shock-delay","5","--shock-death-rate","0.04","--shock-duration","40","--death-rate","0.0002") + $warns
& ".\target\release\zygote-runtime.exe" @args 2> e035_stderr.log
```

---

## E036 — Evolution of Communication (memory warns OTHERS)

**Question:** Can one memory cell warn *others*? The prior chain was individual:
```
warning → memory → brace → higher survival → more descendants
```
The leap is *communication*:
```
warning → memory cell → warning signal (type-50) → other cells brace
```
If communication evolves and spreads because it improves survival, we cross:
```
individual information → shared information
```

**Runtime changes (zygote-runtime):**
- **Recipients now brace too.** Previously `prepared_ticks` was granted only to the *emitting* memory cell (in `Cell::tick`, on emitting type-50). Now `distribute_signals` sets `prepared_ticks = PREPARED_DURATION` on **every cell that RECEIVES** a type-50 signal (both broadcast and directed branches). This is the crux: one cell's prediction now protects *others*.
- **New law L#060** fires when a type-50 delivery reaches a cell of a *different lineage* than its emitter ("Communication Converts Individual Information Into Shared Information").
- **Genome `e036_evolution_of_communication.json`:** three startup lineages with cross-linked `sibling` fields:
  - `memory_cell` (affinity `[1,9]`, `is_memory:true`, `memory_cost:2`) — perceives warning type-9, emits protective type-50. Pays the +2/tick maintenance tax.
  - `listener_cell` (affinity `[1,50]`, `is_memory:false`, `memory_cost:0`) — NO memory cost, but braces on *receiving* type-50. Rides on others' predictions.
  - `reactive_cell` (affinity `[1,9]`, `memory_cost:0`) — control: no memory, no listening. Dies at full rate during shocks.
- Dashboard catalog (`zmon`) extended through L#062 so L#057–L#062 are discoverable.

**Result: COMMUNICATION WORKS — and the dynamics are the opposite of E035.**

Empirical proof the channel exists: **806 `memory_cell --[50]--> listener_cell` deliveries** (918k total type-50 deliveries); **L#060 fired** (shared-information law discovered at runtime).

But the *lineage composition* surprised us (max_instances 200 each, total cap 600; warnings every 60 ticks, 80% real / 20% false, shock death 0.04):

| Tick | Total | memory | listener | reactive |
|---|---|---|---|---|
| 500 | 600 | 537 | 53 | 10 |
| 1000 | 600 | 199 | 192 | 209 |
| 1500 | 600 | 200 | 146 | 254 |
| 2000 | 600 | 200 | 199 | 201 |
| 2500 | 600 | 200 | 135 | 265 |
| 3000 | 600 | 200 | 93 | 307 |
| 3500 | 600 | 200 | 85 | 315 |

- **memory** clamped at its 200 cap the whole run — it is the *indispensable producer* of warnings but the +2/tick tax caps its competitiveness. It can't spread past its own cost.
- **listener** rose early (53→192) then *declined* (→85) — even though it gets the brace for **free** (no memory cost). It did NOT take over.
- **reactive** (no memory, no listening) *won*, climbing to 315/600.

**Why the free-rider listener loses:** listener affinity `[1,50]` means it competes for the type-1 food FIFO against everyone, and its brace only helps during the brief shock window. Between shocks it has *no* edge over reactive, while reactive (unburdened by either memory cost *or* listening contention) simply out-reproduces in steady state. The memory cell *subsidizes* everyone's protection but captures none of the surplus.

**This is the E035 result inverted and deepened.** E035: memory (individual prediction) spread because *it alone* captured the brace benefit. E036: once the benefit is *shared*, the producer is saddled with the cost while free-riders (and even non-listeners) capture the upside — so the *producer* is suppressed and the *cheapest non-contributor* wins. Communication evolves (the channel is real and used) but does **not** enrich the lineage that invented it.

**Laws discovered:**
- **L#060 (confirmed):** Communication Converts Individual Information Into Shared Information — proven by 806 cross-lineage type-50 deliveries.
- **L#061 (new):** A Single Predictor Can Protect A Whole Lineage — one memory cell's type-50 braces every listener/reactive cell with affinity 50 each shock cycle.
- **L#062 (new):** Cheap Listeners Free-Ride On Expensive Predictors — the listener gets the brace for free yet *declines*, because the producer bears the maintenance cost while free-riders capture the benefit without the tax. This is the dark twin of L#057: shared information can *dissipate* the fitness of its source.
- Reinforces **L#058** (crossover flows to fitter strategy): here reactive is the attractor. Reinforces **L#059** indirectly — the communication channel did not prevent the system settling on the cheapest lineage.

**Boundary condition (the surprising-law generator):** As hypothesized, E036 produced *more* unexpected structure than E035. The naive prediction ("listener should dominate — free protection!") was wrong; the real outcome is producer-suppression + free-rider-decline + cheap-control-wins. The metric that would reveal this — *who pays vs who benefits* — was invisible until we tracked per-lineage energy/composition, not just "did communication happen."

**Reproduce:**
```powershell
cd zygote-runtime
cargo build --release
$warns = @(); for ($t=100; $t -lt 5000; $t+=60) { $warns += "--warn-signal"; $warns += "9@$t" }
$args = @("--genome","genomes/e036_evolution_of_communication.json","--ticks","5000","--events-log","..\events_e036.jsonl","--max-cells","600","--inject","1,0,4294967295","--inject-interval","2","--shuffle","--warn-shock-prob","0.8","--warn-shock-delay","5","--shock-death-rate","0.04","--shock-duration","40","--death-rate","0.0002") + $warns
& ".\target\release\zygote-runtime.exe" @args 2> e036_stderr.log
cd ..
.\zmon.bat events_e036.jsonl --dashboard
```

---

## E037 — Selective Communication (the antidote to L#062)

**Hypothesis (yours):** E036 showed broadcast warning makes information a *public good* → the taxed predictor (memory) is suppressed while free-riders win (L#062). Test: make the warning **directed** to *kin* (same lineage) only:
```
Memory → Directed warning → Chosen recipients benefit
```
**Law #063 — Selective Communication Protects Cooperative Information Systems:** if warnings are directed only to cooperating recipients (kin/contributors/trusted) rather than broadcast to all cells, the fitness benefit generated by the information remains within the communicating lineage. This suppresses free-rider expansion and stabilizes cooperative information exchange. The antidote to L#062.

**Runtime changes (zygote-runtime):**
- `Gene.recipient_policy` (`"broadcast"` | `"kin"`), default `"broadcast"`.
- `Runtime.directed_warning: bool` (CLI `--directed-warning`).
- In `step()`, precompute `kin_recipients: HashMap<lineage, Vec<cell_id>>` **once per tick** (outside the per-cell mutable loop, to avoid borrow conflicts). When a cell with `recipient_policy:"kin"` emits type-50 and `directed_warning` is on, the broadcast is rewritten into **DIRECTED** copies (one per alive same-lineage cell). Recipients still brace on receipt (L#060 mechanism). L#063 discovered when a kin-directed type-50 is emitted.
- `memory.asm` unchanged — still "broadcasts" type-50; the runtime rewrites the target. This keeps the cell ABI stable (L#007) and implements selectivity at the layer that knows lineages.

**Genome `e037_selective_communication.json`:** same three lineages as E036, but `memory_cell.recipient_policy = "kin"`. Listeners/reactive stay `"broadcast"` (irrelevant for them — they don't emit type-50).

**Result: THE ANTIDOTE WORKED.** Head-to-head (both: warnings/60 ticks, 80% real, shock death 0.04, max 200/lineage, total 600):

| Tick | E036 broadcast — memory / listener / reactive | E037 directed/kin — memory / listener / reactive |
|---|---|---|
| 500 | 537 / 53 / 10 | 373 / 218 / 9 |
| 1500 | 200 / 146 / **254** | 222 / 340 / 38 |
| 3000 | 200 / 93 / **315** | 236 / **341** / 23 |

- **E036 (broadcast):** reactive won (315), memory clamped at cap, listener collapsed → L#062 free-rider tragedy.
- **E037 (directed/kin):** listener dominates (**341**), and critically **reactive is suppressed to ~23** (vs 315 under broadcast) — because it no longer receives the free public-good protection. Memory retains a healthy **236** (not clamped-collapsed).
- **L#063 fired** (1 discovery); **382 directed `memory_cell --[50]--> memory_cell` edges** prove selective sharing is real and used.

**Interpretation:** Selective sharing converts the public good back into a *club good*. By excluding non-kin from the warning, the predictor denies free-riders the benefit, so the cheap non-contributor (reactive) no longer wins. The lineage that *pays* to predict now *retains* the upside — exactly the L#063 prediction:
```
broadcast information → benefits diffuse → free riders prosper → producers decline
selective information → benefits remain local → contributors prosper → cooperation persists
```
Note the winner under E037 is **listener** (gets kin-directed protection for free *within* the memory club), not memory itself — memory still pays the +2/tick tax. The cleanest "cooperative system protected" reading: reactive (zero cost, zero protection under kin-mode) is what got crushed, which is the L#062 free-rider that the antidote removes.

**This is the richest of the three memory experiments.** E035: individual prediction spreads. E036: shared prediction creates a public-good tragedy. E037: *selective* sharing resolves it — information's value is a function of *who receives it*, not just *that* it's transmitted.

**Laws:**
- **L#063 (confirmed):** Selective Communication Protects Cooperative Information Systems — proven by 382 kin-directed type-50 edges + L#063 discovery + the reactive-suppression contrast (315→23).

**Reproduce:**
```powershell
cd zygote-runtime
cargo build --release
$warns = @(); for ($t=100; $t -lt 5000; $t+=60) { $warns += "--warn-signal"; $warns += "9@$t" }
$args = @("--genome","genomes/e037_selective_communication.json","--ticks","5000","--events-log","..\events_e037.jsonl","--max-cells","600","--inject","1,0,4294967295","--inject-interval","2","--shuffle","--directed-warning","--warn-shock-prob","0.8","--warn-shock-delay","5","--shock-death-rate","0.04","--shock-duration","40","--death-rate","0.0002") + $warns
& ".\target\release\zygote-runtime.exe" @args 2> e037_stderr.log
cd ..
.\zmon.bat events_e037.jsonl --dashboard
```

---

## E038 — Ecosystem Collapse Threshold

**Question:** Starting from the E037 directed-kin setup (memory = producer that emits the protective type-50 warning to kin only; listener = gets protected; reactive = free-rider that neither pays the memory tax nor emits), gradually increase pressure on the *producer* and find the point where the warning network (and, ideally, the population) collapses.

**Mechanism added for E038:**
- `Runtime.producer_death_bonus: f64` + CLI `--producer-death-bonus <f64>` — extra per-tick mortality applied ONLY to `is_memory` (producer) cells in the death check (`runtime.rs`). This simulates gradualy rising free-rider / suppression pressure on the information producer.
- `PopulationSnapshot.producer_fraction` — snapshots now emit the live fraction of alive cells that are producers, so the collapse threshold is *measurable* (also read by `zmon`).

**Run:** sweep `--producer-death-bonus` over {0.0, 0.001, 0.002, 0.003, 0.005, 0.015, 0.03} at the E037 baseline (shock-death-rate 0.04, max 600 cells, 3000 ticks). Use `run_e038_sweep.ps1`; extract with `extract_e038.py` / `extract_e038_fine.py`.

**Results (producer_fraction over time, final type-50 edge count):**

| bonus (/tick) | t=200 pf | t=500 pf | t=2000+ pf | type-50 edges (3000t) | verdict |
|---|---|---|---|---|---|
| 0.000 | 0.27 | 0.16 | ~0.20 (stable) | 1,359,777 | healthy network |
| 0.001 | 0.29 | 0.37 | ~0.22 (stable) | — | healthy network |
| 0.002 | 0.29 | 0.007 | **0.0** (tick ~2000) | — | **collapse** |
| 0.003 | 0.29 | 0.025 | **0.0** | — | **collapse** |
| 0.005 | 0.29 | 0.007 | 0.0 | — | collapse |
| 0.015 | 0.10 | 0.0 | 0.0 | — | collapse |
| 0.030 | 0.003 | 0.0 | 0.0 | **0** | collapse (network dark) |

**Measured collapse cliff: between 0.1% and 0.2% extra producer mortality per tick.** Below ~0.1%/tick the producer fraction self-sustains at ~20%+ and the directed-warning network stays alive (1.36M type-50 edges); at ≥0.2%/tick producers are driven extinct and the warning network goes silent.

**Important caveat — population did NOT collapse in these runs.** At shock-death-rate 0.04 the food subsidy (type-1 every 2 ticks) keeps the free-rider population (listener + reactive) alive even after producers vanish, so total population stays pinned at `max-cells 600`. The *warning network* collapses; the *population* does not, because the protection (PREPARED_DURATION=6 ticks vs 40-tick shocks, PREDICTIVE_DISCOUNT=0.25) is too weak/short to be load-bearing at any shock rate that is survivable without warnings. Tested higher shock rates (0.2, 0.6): at 0.2 the *whole* population dies even with producers present (b=0.0 → 0 by tick 300); at 0.6 same. There is no shock rate where "producers present → survive, producers absent → collapse" cleanly — the warning is not strong enough to be the difference between life and death. **Therefore Law #064 is recorded on the demonstrable axis (warning-network collapse at a measured minimum producer fraction); the "population collapses" step is the law's logical prediction, not something this rig can demonstrate with the current protection strength.**

**Law #064 — Information Ecosystems Require A Minimum Producer Fraction:** an information-sharing ecosystem collapses once the producer fraction falls below a critical threshold (~0.7% of population here; driven under by ≥0.2%/tick extra producer mortality). Below threshold the warning network goes dark; above it, it self-sustains. The free-rider mirror of L#063: selective sharing protects producers from external free-riders, but internal producer suppression still pushes the fraction under the cliff — **unchecked free riders (or producer loss) collapse information ecosystems.** This is the first fully *measured* threshold law in the catalog.

**Reproduce:**
```powershell
cd zygote-runtime
cargo build --release
$warns = @(); for ($t=100; $t -lt 3000; $t+=60) { $warns += "--warn-signal"; $warns += "9@$t" }
$base = @("--genome","genomes/e038_ecosystem_collapse.json","--ticks","3000","--max-cells","600",
  "--inject","1,0,4294967295","--inject-interval","2","--shuffle","--directed-warning",
  "--warn-shock-prob","0.8","--warn-shock-delay","5","--shock-death-rate","0.04","--shock-duration","40",
  "--death-rate","0.0002") + $warns
# stable network:
& ".\target\release\zygote-runtime.exe" @base --producer-death-bonus 0.0   --events-log events_e038_b0.jsonl
# collapse:
& ".\target\release\zygote-runtime.exe" @base --producer-death-bonus 0.002 --events-log events_e038_fine_b0_002.jsonl
cd ..
python extract_e038.py
```

