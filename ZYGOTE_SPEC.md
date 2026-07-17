# ZygoteAI — System Specification

## 1. Overview

ZygoteAI is a multi-agent cognitive architecture in which a **genome** governs the behavior, lifecycle, and interaction of many **cell-agents** (cells).

**Language stack:**
- **Cells** — written in x86-64 Assembly. Raw machine code with no runtime, no standard library, no safety net. This is the *zygote*: the most primitive possible computational unit.
- **Runtime** — written in Rust. The *foster parent*: hosts, sandboxes, monitors, and orchestrates cells. Manages the genome, signal bus, lifecycle, and evolution.
- **Genome** — JSON. Human-readable blueprint consumed by the Rust runtime.

The system is designed to explore emergent problem-solving from first principles.

## 2. Core Concepts

### 2.1 Genome
The genome is a mutable blueprint — a set of genes that describe how cells are created, how they communicate, and how they die. The genome can be hand-authored, randomly generated, or evolved through selective pressure. The Rust runtime is the sole reader and executor of the genome.

### 2.2 Cell
A cell is a lightweight, ephemeral block of Assembly code loaded into a sandboxed memory region by the Rust runtime. Each cell has:
- A **type** (determined by the genome).
- A **state** (a fixed-size scratch memory page).
- A **stimulus-response cycle** (receive signal buffer → compute → write signal buffer).
- A **lifespan** (can die or be killed).

Cells communicate via flat, fixed-size signals passed through the Rust-managed signal bus.

### 2.3 Phenomena
Observable system-level behaviors that emerge from cell interactions. A phenomenon is not explicitly programmed — it is measured and named post-hoc.

### 2.4 Foster Parent (Rust Runtime)
The Rust binary that:
- Parses the genome JSON at startup.
- Loads compiled Assembly blobs into mmap'd memory regions.
- Runs the tick loop.
- Routes signals between cells.
- Enforces resource limits (CPU, memory, lifespan).
- Detects crashes (SIGSEGV, division by zero, infinite loops via watchdog timer).
- Logs all signal traffic.
- Drives genome evolution (mutation, crossover, selection).

## 3. System Architecture

```
┌──────────────────────────────────────────┐
│             Rust Runtime                  │
│  ┌────────────────────────────────────┐  │
│  │         Genome Engine              │  │
│  │  (parse, evolve, persist JSON)     │  │
│  └────────────┬───────────────────────┘  │
│               │ spawn/kill               │
│  ┌────────────▼───────────────────────┐  │
│  │         Cell Pool                  │  │
│  │  ┌──────┐ ┌──────┐ ┌──────┐       │  │
│  │  │asm#1 │ │asm#2 │ │asm#3 │  ...  │  │
│  │  └──┬───┘ └──┬───┘ └──┬───┘       │  │
│  │     │        │        │            │  │
│  └─────┼────────┼────────┼────────────┘  │
│        │        │        │               │
│  ┌─────▼────────▼────────▼────────────┐  │
│  │           Signal Bus               │  │
│  │  (Rust-managed message routing)    │  │
│  └────────────────────────────────────┘  │
│  ┌────────────────────────────────────┐  │
│  │    Monitor & Log                   │  │
│  │  (crash detection, tick timing,    │  │
│  │   signal tracing, watchdog)        │  │
│  └────────────────────────────────────┘  │
└──────────────────────────────────────────┘
```

## 4. Cell Memory Model

Each cell gets a fixed memory layout set up by the Rust runtime:

```
┌─────────────────────┐
│  Code (R-X)         │  ← Assembly instructions, read-only
├─────────────────────┤
│  State (RW)         │  ← Persistent scratch memory (e.g. 256 bytes)
├─────────────────────┤
│  Input FIFO (RW)    │  ← Incoming signals for this tick
├─────────────────────┤
│  Output FIFO (RW)   │  ← Outgoing signals for next tick
├─────────────────────┤
│  Control (RW)       │  ← Tick counter, status flags, kill flag
└─────────────────────┘
```

The Rust runtime mmap's these regions, writes initial state, then transfers control to the cell's entry point.

## 5. Cell Lifecycle (Rust-Driven)

1. **Birth** — Rust reads a gene from the genome, mmap's the Assembly blob, writes initial state, registers the cell in the pool.
2. **Listen** — Rust copies pending signals into the cell's Input FIFO.
3. **Process** — Rust calls the cell's entry point (via `extern "C"` or JIT). The cell reads Input FIFO, computes, writes to Output FIFO. A watchdog timer limits execution.
4. **Act** — Rust reads the Output FIFO, routes signals to the bus, may update genome state.
5. **Death** — Rust unmaps the memory, removes the cell from the pool. Triggered by lifespan expiry, kill signal, watchdog timeout, or crash.

## 6. Signal Protocol

Signals are fixed-size binary structs (not JSON — JSON parsing at the Assembly level is untenable). The Rust runtime translates external JSON signals to/from the binary format.

```c
struct Signal {
    uint8_t  type_id;       // 0-255, mapped to genome signal registry
    uint32_t source_id;     // cell pool index
    uint32_t target_id;     // cell pool index or 0xFFFFFFFF for broadcast
    uint8_t  payload[64];   // flat byte payload
};
```

The Rust runtime handles serialization and deserialization between this binary format and external JSON.

## 7. Genome Encoding

The genome is a JSON file. Each gene specifies:
- `cell_type`: unique identifier.
- `asm_path`: path to the compiled Assembly blob (.bin).
- `stack_size`: stack size in bytes for this cell.
- `state_size`: scratch memory size in bytes.
- `spawn_condition`: when to create this cell (on signal, on timer, on startup).
- `max_instances`: upper bound on concurrent instances.
- `signal_affinity`: list of signal type IDs this cell reacts to.
- `mutation_rate`: probability of offspring variation.
- `timeout_us`: max microseconds per tick before watchdog fires.

## 8. Evolution Mechanism

The Rust runtime drives evolution:
- **Mutation** — alter a gene field (e.g., tweak `timeout_us`, swap `asm_path` to a variant).
- **Crossover** — split two genomes at a random gene boundary and recombine.
- **Selection** — score each genome against a fitness function, retain top N.

Assembly-level mutation (modifying the binary) is a future capability.

## 9. Safety & Sandboxing

Cells are untrusted machine code. The Rust runtime enforces:
- **Memory isolation** — each cell gets its own mmap'd region. No access to other cells or the runtime's memory.
- **Execution timeout** — watchdog timer (SIGALRM or thread-based) kills runaway cells.
- **Signal handler** — catch SIGSEGV, SIGILL, SIGFPE, SIGBUS. On fault: log the crash, kill the cell, continue.
- **Bounds checking** — Input/Output FIFOs are ring buffers with Rust-managed head/tail pointers. The Assembly cell sees only its slice.
- **Rate limiting** — cap signals emitted per tick per cell.

## 10. Constraints

- All cells are deterministic given the same input + same initial state.
- The genome is the only source of persistent truth.
- No cell can address memory outside its mmap'd region.
- The system must run in bounded memory (pruning enforced by Rust).

## 11. Future Considerations

- JIT compilation for dynamically-generated Assembly.
- Genetic programming at the Assembly instruction level.
- Time-travel debugging via signal log replay.
- Visual genome editor.
- Hybrid human-AI gene authoring.
- WebAssembly backend (portable Assembly target).
