# 64-bit Zygote

**64-bit Zygote** is an experimental evolutionary systems laboratory built from two components:

* **Assembly cells** (x86-64 machine code)
* **Rust runtime** (orchestration, scheduling, signaling, safety)

Each cell is an independent executable organism with its own state, energy budget, signal affinities, reproduction rules, and lifecycle.

Cells communicate through a binary signal bus, reproduce through heredity and mutation, compete for survival under environmental pressures, and collectively produce emergent behavior that is not explicitly programmed into any individual cell.

## Why?

Most evolutionary simulations start with high-level abstractions.

64-bit Zygote starts much closer to the machine.

Every organism is literally executable code.

The project explores what kinds of behaviors emerge when simple machine-code agents interact through signals, energy constraints, mutation, reproduction, memory, and environmental stress.

The goal is not to build artificial consciousness.

The goal is to discover and document reproducible laws of adaptation, selection, information processing, and emergent organization.

## Architecture

```text
Assembly Cell
      │
      ▼
Input FIFO
      │
      ▼
Cell Logic (.asm)
      │
      ▼
Output FIFO
      │
      ▼
Signal Bus
      │
      ▼
Other Cells
```

### Runtime

The Rust runtime provides:

* Signal routing
* Energy accounting
* Reproduction
* Mutation
* Heredity
* Cell lifecycle management
* Scheduling strategies
* Environmental events
* Experiment instrumentation

### Cells

Cells are compiled assembly binaries.

Examples:

* Sensor
* Relay
* Loopback
* Logger
* Memory
* Self-destruct
* Reproducer

Cells know nothing about the larger system.

They only process signals according to their local logic.

## Research Direction

The project investigates:

* Emergence
* Evolutionary dynamics
* Fitness landscapes
* Selection pressure
* Adaptation
* Diversity maintenance
* Memory
* Prediction
* Communication
* Ecosystem stability

## Experimental Method

Every experiment is documented in LAB_NOTES.md.

Experiments are assigned IDs:

```text
E001
E002
E003
...
```

Observed principles are recorded as Laws:

```text
L005  Pool Order Creates Bias
L020  Heredity Enables Evolution
L028  Mortality Unfreezes Selection
L056  Predictive Information Has Fitness Value
L059  Fixation Creates Extinction Risk
```

The repository serves both as a software project and as a laboratory notebook.

## Current Status

64-bit Zygote has demonstrated:

* Stable signaling loops
* Competitive selection
* Heredity and mutation
* Fitness gradients
* Evolutionary hill-climbing
* Environmental adaptation
* Memory-based survival advantages
* Information-driven selection

The project is currently exploring the transition from simple adaptation toward communication and collective information processing.

## Disclaimer

64-bit Zygote is an experimental research project.

It is not intended to model biological reality with scientific accuracy, nor is it a claim of artificial consciousness.

Any similarities between observed behaviors and biological systems should be treated as hypotheses for further investigation rather than evidence of equivalence.


## Project Status

This project is in its earliest stage — a living lab notebook more than a product. See [LAB_NOTES.md](./LAB_NOTES.md) for current state, and [ROADMAP.md](./ROADMAP.md) for direction.

---

**Slogan:** *From a seed of logic, a garden of mind.*
