# Constraints

## Hard Constraints (Must Always Hold)

1. **Determinism** — given the same genome, same input sequence, and same random seed, the system produces identical output.
2. **Bounded memory** — total cell count never exceeds a configurable maximum.
3. **Memory isolation** — no cell can read or write memory outside its mmap'd region. Enforced by OS page protection (Rust sets page permissions).
4. **No direct cell-to-cell access** — cells communicate only through the Rust-managed signal bus.
5. **Genome authority** — the genome is the sole source of persistent truth. All cell state is regenerable from the genome.
6. **No privileged instructions in cells** — cells run in user space. SIGILL on `syscall`, `int`, or privileged `cr*` access.
7. **Watchdog termination** — any cell that exceeds its time budget or causes a fatal signal (SIGSEGV, SIGFPE, SIGILL, SIGBUS) is killed by the Rust runtime.

## Soft Constraints (Should Hold)

1. **Tick deadline** — each tick completes within a configurable time budget.
2. **Signal delivery** — intra-tick signal delivery is eventually consistent.
3. **Graceful degradation** — when resource limits are hit, low-priority cells are pruned first.
4. **Observability** — every signal is logged by default (can be sampled or filtered in production).
5. **Crash resilience** — a single cell crash never takes down the entire runtime.

## Anti-Patterns (Must Avoid)

- Cells holding global state or accessing memory outside their mmap.
- Genome containing hardcoded cell pool indices (cells are ephemeral).
- Synchronous blocking calls within a cell's tick (watchdog will fire).
- Signal broadcast storms (no ttl / hop-count mechanism).
- Assembly cells executing `int 0x80` or `syscall` (privilege escalation).

## Safety Mechanisms (Rust Runtime)

| Mechanism            | Implementation                                |
|----------------------|-----------------------------------------------|
| Memory isolation     | `mmap` with `PROT_READ \| PROT_WRITE` per region |
| Execute-only code    | `mprotect` code page to `PROT_READ \| PROT_EXEC` |
| Watchdog timer       | `std::time::Instant` per tick, per-cell timeout |
| Crash signal handler | `sigaction` catching SIGSEGV, SIGFPE, SIGILL, SIGBUS |
| Stack isolation      | Separate stack per cell (set via `rsp` on entry) |
| FIFO overflow guard  | Rust checks head/tail before writing signals   |

## Performance Bounds (Initial Targets)

| Metric            | Target          |
|-------------------|-----------------|
| Max cells/tick    | 10,000          |
| Max signals/tick  | 100,000         |
| Tick latency      | < 10ms          |
| Watchdog timeout  | 100μs per cell  |
| Genome size       | < 1MB           |
| Cell state limit  | 256B per cell   |
| Cell FIFO depth   | 4 signals       |
