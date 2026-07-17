# Cell — Agent Architecture

## Overview

A **cell** is the fundamental computational unit in ZygoteAI. It is a block of x86-64 Assembly code loaded into a sandboxed memory region by the Rust runtime. Cells are lightweight, ephemeral, and single-purpose.

## Memory Layout

Each cell gets a fixed 4-region mmap allocated by Rust on spawn:

```
Offset    Region         Perm    Size          Contents
──────────────────────────────────────────────────────────
0x0000    Code           R-X     asm_file_size  Compiled Assembly blob
0x1000    State          R-W     configurable   Scratch memory (default 256B)
0x2000    Input FIFO     R-W     256 bytes      Ring buffer of incoming signals
0x3000    Output FIFO    R-W     256 bytes      Ring buffer of outgoing signals
0x4000    Control Block  R-W     64 bytes       tick_count, status, kill_flag
```

Pointers to these regions are passed to the cell in registers on entry:
- `rdi` → State base
- `rsi` → Input FIFO base
- `rdx` → Output FIFO base
- `rcx` → Control Block base

## Calling Convention

The Rust runtime calls each cell's entry point once per tick:

```c
// Rust calls via extern "C" function pointer
void cell_tick(uint8_t *state, uint8_t *input_fifo,
               uint8_t *output_fifo, uint8_t *control);
```

The cell:
1. Reads signals from Input FIFO (circular buffer, see protocol below).
2. Computes using State as persistent scratch.
3. Writes output signals to Output FIFO.
4. Returns. Rust reads Output FIFO and routes signals.

## FIFO Protocol

Signals are fixed-size binary structs:

```c
struct Signal {
    uint8_t  type_id;       // 0-255
    uint32_t source_id;     // set by Rust on delivery
    uint32_t target_id;     // 0xFFFFFFFF = broadcast
    uint8_t  payload[56];   // 56 bytes = 64 total per signal
};
// Total: 1 + 4 + 4 + 56 = 65 bytes, padded to 64? 
// Actual: packed to 64 bytes for FIFO alignment.
```

Each FIFO is a ring buffer with 4 slots (256 bytes ÷ 64 bytes/signal). Head and tail pointers are stored in the Control Block at known offsets:
- `control[0..2]` — input head index (u16)
- `control[2..4]` — input tail index (u16)
- `control[4..6]` — output head index (u16)
- `control[6..8]` — output tail index (u16)

## Cell Contract (ABI)

Cells must follow these rules (enforced by the Rust watchdog):
1. Do not touch memory outside the four mapped regions.
2. Do not execute privileged instructions (will cause SIGILL).
3. Return within `timeout_us` microseconds.
4. Do not write past the FIFO capacity (Rust may truncate).
5. Set `kill_flag` (control[63]) to 1 to request death.

## Built-in Cell Types

| Type           | Purpose                              |
|----------------|--------------------------------------|
| `sensor`       | Inline assembly that bridges external input → signals |
| `accumulator`  | Aggregates signals over time in state memory |
| `filter`       | Passes signals matching a state-held predicate |
| `mapper`       | Transforms payload bytes in one signal to another |
| `combiner`     | Reads two input slots, writes one merged output |
| `switch`       | Routes signals to different target IDs based on payload |
| `timer`        | Decrements a counter in state, emits signal on zero |
| `logger`       | Copies signals to a reserved output region for Rust tracing |
| `spawner`      | Writes a spawn-request signal for the genome engine |
| `reaper`       | Emits kill signals for cells matching criteria |

Each type ships as a pre-assembled `.bin` file linked from the genome.

## State Rules

- State is a flat byte array of configurable size (set in genome).
- Cells may interpret state bytes however they wish (counters, flags, pointers).
- State persists across ticks within a cell's lifetime.
- State is initialized to zero on spawn.

## Resource Limits

- **CPU** — watchdog timer per tick (configurable per gene, default 100μs).
- **Memory** — fixed mmap size per cell, set at spawn.
- **Lifespan** — bounded number of ticks (can be extended via genome signal).
- **Output** — max 4 signals per tick (FIFO depth).

## Death

A cell dies when:
- Its lifespan reaches zero (checked by Rust after each tick).
- It sets the kill flag in its Control Block.
- The Rust watchdog fires (timeout or crash signal).
- The genome prunes its type.
- The runtime evicts it due to resource pressure.
