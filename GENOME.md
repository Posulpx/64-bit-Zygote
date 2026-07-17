# Genome — Architecture & Design

## What is the Genome?

The genome is the **blueprint and orchestrator** of the ZygoteAI system. It defines what types of cells exist, which Assembly blobs to load, how cells behave, when they spawn, and how they evolve.

The genome is a JSON file consumed by the Rust runtime at startup and during evolution.

## Gene Structure

Each gene is a JSON object with the following schema:

```json
{
  "cell_type": "accumulator",
  "version": 1,
  "asm_path": "cells/accumulator/accumulator.bin",
  "stack_size": 512,
  "state_size": 256,
  "timeout_us": 100,
  "spawn_condition": {
    "on_signal": "data_point",
    "probability": 0.5
  },
  "max_instances": 10,
  "signal_affinity": [1, 5],
  "mutation_rate": 0.01,
  "metadata": {
    "description": "Collects data points and exposes a running sum."
  }
}
```

### Field Descriptions

| Field             | Type     | Description                                        |
|-------------------|----------|----------------------------------------------------|
| `cell_type`       | string   | Unique identifier for this cell type               |
| `version`         | integer  | Gene version (incremented on mutation)             |
| `asm_path`        | string   | Path to compiled .bin file (relative to genome)    |
| `stack_size`      | integer  | Stack size in bytes for this cell type             |
| `state_size`      | integer  | Scratch memory size in bytes                       |
| `timeout_us`      | integer  | Watchdog timeout per tick in microseconds          |
| `spawn_condition` | object   | Rules for when to spawn instances                  |
| `max_instances`   | integer  | Upper bound on concurrent instances of this type   |
| `signal_affinity` | integer[] | Array of signal type IDs this cell reacts to       |
| `mutation_rate`   | float    | Probability of this gene being altered on mutation |
| `metadata`        | object   | Human-readable notes (ignored by runtime)          |

## Genome Encoding Formats

| Format   | Use Case                    |
|----------|-----------------------------|
| JSON     | Human-readable authoring    |
| Binary   | Serialization & evolution   |
| Graph    | Visual editing              |

## Genome Operations (Implemented in Rust)

### Mutation
A single gene is selected. One of its numeric fields is randomly altered. `asm_path` may be swapped to a variant blob if available.

### Crossover
Two genomes exchange contiguous gene segments at a random breakpoint.

### Selection
Each genome is scored by a fitness function. Low-scoring genes/genomes are removed or replaced.

## Assembly-Level Mutation (Future)

Higher-risk evolution where the Rust runtime directly patches the Assembly binary:
- NOP sliding (insert no-ops to shift offsets).
- Instruction substitution (e.g., `add` ↔ `sub`, `jmp` ↔ `je`).
- Constant vector manipulation (change immediate values).

## Persistence

The genome is stored as the authoritative record of system state. Cells are ephemeral and can always be regenerated from the genome + their `.bin` files.

## Future

- Multi-genome populations with tournaments.
- Gene dependency graphs (gene A requires gene B).
- Lazy evaluation: genes that activate only under specific environmental conditions.
- In-memory genome database for faster evolution cycles.
