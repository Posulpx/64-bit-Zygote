# Milestones

## Phase 0: Conception (Current)

- [x] Define project concept and philosophy.
- [x] Draft system specification.
- [x] Choose implementation language (Assembly + Rust).
- [ ] Create Rust workspace (cargo init, project structure).
- [ ] Set up Assembly build toolchain (NASM or similar, Makefile).
- [ ] Implement signal bus prototype (Rust).
- [ ] Write first Assembly cell (sensor) and assemble to .bin.
- [ ] Write first Assembly cell (logger) and assemble to .bin.
- [ ] Rust runtime: load .bin, mmap regions, call cell entry point.
- [ ] Run first end-to-end test: sensor → logger.

## Phase 1: Viability

- [ ] All 10 built-in cell types implemented as Assembly .bin files.
- [ ] Genome parsing from JSON (Rust).
- [ ] Tick-based simulation loop (Rust).
- [ ] Cell lifecycle management: birth, listen, process, act, death (Rust).
- [ ] Watchdog timer and crash recovery (Rust signal handlers).
- [ ] First observed phenomenon documented.

## Phase 2: Evolution

- [ ] Mutation and crossover operators.
- [ ] Fitness function framework.
- [ ] Automated evolution runs.
- [ ] Visualization of cell interaction graphs.

## Phase 3: Scaling

- [ ] 1,000+ concurrent cells.
- [ ] Distributed signal bus.
- [ ] Real-time visualization dashboard.
- [ ] Edge deployment (Cloudflare Workers or similar).

## Phase 4: Generalization

- [ ] Domain-specific genome libraries.
- [ ] Transfer learning between genomes.
- [ ] Human-in-the-loop gene editing.
- [ ] Benchmark suite for cognitive tasks.
