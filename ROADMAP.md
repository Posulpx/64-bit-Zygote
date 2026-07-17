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
  log-bound (~7 ticks/sec at 600 cells; 330MB event logs). Planned work:
  - Gate `eprintln!` reproduction/heredity/mortality spam behind `--verbose`
    (default off). Expected 2–5x.
  - Add `--binary-log` mode (packed binary / protobuf) replacing JSON-lines per
    signal; ~10x smaller, ~20x faster parse. Update `extract_*.py` readers.
  - Decouple observation from sim: headless runs, in-Rust aggregate stats,
    sample snapshots every N ticks, emit summary once.
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

## Beyond

- Domain-specific genome libraries.
- Hybrid human-AI genome design.
- Benchmark suite for emergent cognition.
- Deployment as a service.

---

*This roadmap is a living document. Milestones will shift as we learn.*
