# Roadmap

## Q3 2026 — Germination

- [x] Project scaffold (docs, spec).
- [x] Language decision: Assembly for cells, Rust for runtime.
- [ ] Rust workspace: cargo init, signal bus, mmap loader.
- [ ] Assembly toolchain: NASM, per-cell Makefile, link scripts.
- [ ] First two cells: sensor.asm → sensor.bin, logger.asm → logger.bin.
- [ ] Smoke test: Rust loads sensor.bin + logger.bin, signal flows.
- [ ] First logged phenomenon.
- [ ] CI / basic tests (cargo test + assemble check).

**Exit criteria:** A signal flows from an Assembly sensor cell through the Rust signal bus to an Assembly logger cell.

## Q4 2026 — Differentiation

- [ ] All 10 core cell types as Assembly .bin files.
- [ ] Genome JSON parser in Rust.
- [ ] Genome mutation and crossover in Rust.
- [ ] Manual and random genome generation.
- [ ] Basic visualization (cell graph / signal timeline).

**Exit criteria:** An evolved genome solves a simple task better than a random genome.

## Q1 2027 — Organization

- [ ] 100+ concurrent Assembly cells.
- [ ] Multi-tick signal chains.
- [ ] Fitness-driven evolution in Rust.
- [ ] Watchdog timer and crash recovery hardened.
- [ ] Homeostasis phenomenon achieved and documented.

**Exit criteria:** System self-stabilizes after input perturbation without explicit programming.

## Q2 2027 — Extension

- [ ] Distributed runtime (multi-process / multi-machine).
- [ ] Real-time dashboard with signal trace visualization.
- [ ] Public sandbox for experimentation.
- [ ] Paper / technical report draft.

**Exit criteria:** External contributor runs an experiment and documents a new phenomenon.

## Beyond 64

> Logged 2026-07-17. The 64-law catalog (L#001–L#064) is complete through the
> ecosystem-collapse threshold (E038). The next phase is about *throughput* and
> *meaning* — making the instrument fast enough to test its own predictions, and
> stating what the laws argue.

- **Simulation speedup (blocker for further experiments).** Runtime is I/O- and
  log-bound (~7 ticks/sec at 600 cells; 330MB event logs). **DONE (partial,
  2026-07-17):** added `--verbose` (gates the per-cell `eprintln!` debug spam,
  default off) and `--summary-log` (skips per-signal JSON writes, keeps
  snapshots/laws/births). Measured: **2.4x faster, 14x smaller files** at 1000
  ticks (86s/7MB vs 207s/105MB). Remaining:
  - `--binary-log` mode (packed binary) replacing JSON for an even bigger win +
    faster parse; update `extract_*.py`.
  - Decouple observation: headless runs, in-Rust aggregate stats, sample
    snapshots every N ticks.
  - Only if needed: parallelize the cell step loop; replace hot-path `HashMap`
    with Vec keyed by fixed cell id.
- **Retest L#064's population-collapse prediction.** Current protection window
  (`PREPARED_DURATION=6`) is far shorter than shock duration (40 ticks), so the
  warning is never load-bearing at survivable shock rates — only the *warning
  network* collapses, not the population. Either extend `PREPARED_DURATION` to
  match shock duration, or find the shock rate where braced cells live and
  un-braced cells die, to demonstrate the full L#064 chain (producer loss →
  protection loss → population collapse).
- **Articulate the "why" — `ZYGOTE_THESIS.md`.** Frame Zygote as a minimal model
  of the evolution of cooperative information systems: communication emerges,
  becomes a public good, is destroyed by free riders (L#062), rescued by
  selective sharing (L#063), and persists only above a minimum producer fraction
  (L#064). Connect to language / immune alarm signaling / AI alignment. The
  stable state is a club, not a commons.
- **Interactive Canvas visualization (deferred).** Desired: a single self-contained
  HTML file showing the cell population live — dots for cells, lines for type-50
  warning edges, a producer_fraction gauge, shock flashes, sliders for
  `--producer-death-bonus` / shock rate. Blocked on the speedup work above:
  (a) the real Rust runtime can't run in-browser (no mmap/SIGSEGV in WASM), so a
  *faithful* live port means reimplementing the cell loop + signal bus + energy +
  lineage-switch + directed-warning in JS (multi-day); (b) a *replay* viewer of
  recorded event logs is simpler but needs the binary-log format to be tractable
  (330MB JSON is unworkable in-browser). Revisit after speedup + L#064 retest.

---

## North Star — Autonomous Law Discovery

> The long-term goal: an AI agent that *autonomously* designs experiments, runs
> the runtime, reads the event logs, and discovers + writes the next 64 laws
> (L#065–L#128) without human-in-the-loop. Zygote becomes a self-driving ALife
> laboratory.
>
> Prerequisites (must land first):
> - **Speedup** (binary logs, verbose gating) — so an agent can run thousands of
>   experiments, not dozens.
> - **Structured experiment API** — the agent needs to parameterize genomes /
>   CLI flags programmatically and get back compact, parseable summaries (not
>   330MB JSON).
> - **Automated insight extraction** — a reader that turns event logs into
>   traits / edge counts / collapse signals the agent can reason over.
> - **Law diffing** — detect when a new run *contradicts* or *refines* an
>   existing law (L#001–L#064) vs discovers a genuinely new one.
>
> The agent loop: hypothesize → configure genome/flags → run → extract → compare
> to known laws → write new law or update old one. Human reviews the catalog.
>
> **Speculative law target board:** `ZYGOTE_LAWS_PROJECTED.md` lists predicted
> L#065–L#128 (5 research blocks A–E) as hypotheses + experiment designs for the
> autonomous discoverer to confirm or refute.
>
> **Why this matters (applications):** `ZYGOTE_THESIS.md` §2 frames Zygote as an
> instrument, not a product — a reference model (periodic table for cooperative
> information systems) used for cooperation theory, multi-agent AI / alignment,
> distributed-systems failure simulation, and education. The real output is the
> law catalog as a diagnostic, not the simulation.

---

## Beyond

- Domain-specific genome libraries.
- Hybrid human-AI genome design.
- Benchmark suite for emergent cognition.
- Deployment as a service.

---

*This roadmap is a living document. Milestones will shift as we learn.*
