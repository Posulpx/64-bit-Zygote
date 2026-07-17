# Open Questions

## Architecture

1. **Single genome vs. population?** Should we run one genome per simulation, or maintain a population that competes?
2. **Synchronous vs. asynchronous ticks?** All cells tick at once vs. cells tick independently on their own schedules.
3. **How to handle signal routing?** Does the genome encode wiring, or does wiring emerge?
4. **Cell ID scheme?** UUIDs vs. hierarchical names vs. content-addressed.

## Evolution

5. **Fitness functions?** How do we score a genome's performance without a known target?
6. **Genome size control?** Should we penalize genome length to prevent bloat?
7. **Sexual vs. asexual evolution?** Should genomes recombine, or only mutate?

## Implementation

8. **Language choice?** ~~TypeScript (edge compat), Rust (performance), or Python (fast iteration)?~~
   **DECIDED:** Assembly (x86-64) for cells, Rust for runtime.
9. **Signal bus implementation?** In-memory ring buffers (Assembly side), managed by Rust. External JSON bridging later.
10. **Persistence?** Genome storage in JSON files, evolved genomes written as new files.

## Epistemology

11. **When is a behavior truly emergent?** How do we distinguish emergence from aggregation?
12. **How to measure intelligence?** What metrics correspond to "cognitive" capability?
13. **What is a minimal viable phenomenon?** The simplest interaction that produces something surprising.

## Ethics

14. **Should we ever run evolution without a kill-switch?** Unbounded optimization could produce unintended behaviors.
15. **How do we audit a genome's implicit biases?**
