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

## Beyond

- Domain-specific genome libraries.
- Hybrid human-AI genome design.
- Benchmark suite for emergent cognition.
- Deployment as a service.

---

*This roadmap is a living document. Milestones will shift as we learn.*
