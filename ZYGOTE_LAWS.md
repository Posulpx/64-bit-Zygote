# Zygote Laws

## Law #001 — Affinity Is Subscription, Not Routing

A cell's affinity declares which signal types it will accept.

It does not define where signals go.

Routing emerges from the interaction between:
- signal emission
- affinity subscriptions
- runtime delivery

**Evidence:** E001 (Stable Loop), E003 (Broken Circuit)

**Consequence:**
```
Signal Type
    ↓
Subscribers
    ↓
Delivery
```

Cells do not know their neighbors.

Topology emerges from subscriptions.

## Law #002 — Broadcast Is Cooperative, Not Competitive

Broadcast delivery duplicates signals to all matching subscribers.

Subscribers do not compete for a signal.

**Evidence:** E002 (Fork), E004 (Competing Loops)

**Consequence:**
```
1 signal
    ↓
2 subscribers
    ↓
2 deliveries
```

Shared signal types create cooperation rather than contention.

## Law #003 — Throughput Is Limited By Bottlenecks

System throughput is governed by the slowest processing stage.

**Evidence:** E002 (Fork)

**Consequence:**
```
Fast Producer
      ↓
Fast Producer
      ↓
Slow Consumer
```

The consumer determines maximum sustainable throughput.

Excess signals accumulate or are dropped.

## Law #004 — Static Topologies Are Fragile

Removing a critical cell causes extinction if no alternative path exists.

**Evidence:** E003 (Broken Circuit Recovery)

**Consequence:**
```
Break Chain
     ↓
Signal Starvation
     ↓
Extinction
```

Without adaptation, topology damage is fatal.

## Law #005 — Scheduler Order Creates Selection Pressure

Iteration order influences which signals gain access to limited resources.

**Evidence:** E005 (Shared Bottleneck), E007 (Shuffle), E008 (Rotate), E010 (Reverse)

**Consequence:**
```
Producer A first
Producer B second
       ↓
A dominates
```

The scheduler is part of the environment.

## Law #006 — Fairness Depends On Relative Ordering

Changing the starting position is not enough.

Changing relative ordering is what matters.

**Evidence:** E008 (Rotate), E010 (Reverse)

**Consequence:**
```
Rotate
    ≠
Alternate
```

Cyclic rotation preserves hidden precedence.

Reversal removes it.

## Law #007 — The ABI Is Part Of Reality

The Rust runtime and Assembly cells share a physical contract.

Violating the contract alters behavior.

**Evidence:** E009 (FIFO Sweep)

**Consequence:**
```
Runtime Layout
      ≠
Cell Layout
```

Results in:
- corruption
- impossible signals
- crashes
- undefined behavior

The ABI is not implementation detail.

It is substrate physics.

## Law #008 — Directed Signals Bypass Affinity

Targeted signals are delivered directly to a cell regardless of affinity.

**Evidence:** E012 (Directed Signals)

**Consequence:**
```
Broadcast
    = public

Directed
    = private
```

Directed channels create point-to-point communication.

## Law #009 — Cells Can Choose Death

A cell may voluntarily terminate itself through the control block.

**Evidence:** E011 (Self-Termination)

**Consequence:**
```
Cell
 ↓
Kill Flag
 ↓
Death
```

Death is no longer purely environmental.

## Law #010 — Adaptation Converts Extinction Into Degradation

Dynamic affinity can preserve function after topology damage.

**Evidence:** E013 (Dynamic Affinity)

**Consequence:**
```
Damage
  ↓
Rewire
  ↓
Reduced Throughput
  ↓
Survival
```

Adaptation does not restore perfection.

It restores viability.

## Law #011 — Population Growth Requires Regulation

Unbounded spawning produces runaway growth.

**Evidence:** E014 (Spawn-On-Signal)

**Consequence:**
```
Signal
  ↓
Spawn
  ↓
More Signals
  ↓
More Spawn
```

Without constraints, positive feedback dominates.

## Law #012 — Information Processing Has Metabolic Cost

Signal handling consumes energy.

**Evidence:** E015 (Energy)

**Consequence:**
```
Process
 ↓
Energy Loss

Emit
 ↓
Energy Loss
```

Information is no longer free.

## Law #013 — Topology Determines Survival

Different network structures consume energy at different rates.

**Evidence:** E015 (Energy)

**Consequence:**
```
Efficient Loop
     ↓
Long Life

Busy Loop
     ↓
Short Life
```

Structure becomes fitness.

## Law #014 — Workload Creates Lifespan Inequality

Cells performing more work die sooner.

**Evidence:** E015 (Energy)

**Consequence:**
```
Busy Cell
    ↓
Early Death

Idle Cell
    ↓
Late Death
```

Individual cells occupy different ecological niches.

## Law #015 — Adaptation Is Not Free

Repair, growth, and activity all consume resources.

**Evidence:** E013 + E014 + E015

**Consequence:**
```
Repair
 ↓
Cost

Growth
 ↓
Cost

Processing
 ↓
Cost
```

Every adaptive behavior must justify its energy expenditure.

## Law #016 — Initial Endowment Dominates Efficiency

When energy budgets differ significantly, lifespan is determined primarily by starting energy rather than topology.

**Evidence:** E016

**Consequence:**
```
High Efficiency
+
Tiny Fuel Tank

can lose to

Lower Efficiency
+
Large Fuel Tank
```

## Law #017 — Food Creates An Ecosystem

Processing signals gives energy.

Cells that eat survive.

Cells that starve die.

**Evidence:** E017

**Consequence:**
```
Signal
  ↓
Process
  ↓
+Energy

No Signal
  ↓
-Energy
  ↓
Death
```

Energy is no longer a fixed battery.

It is a flow that must be actively harvested.

Access to food determines survival.

## Law #018 — Reproduction Costs Position

When a reproducing cell dies from spawn cost, its pool position is lost.

The non-reproducing competitor inherits the priority slot.

**Evidence:** E018

**Consequence:**
```
Abundant Food
      ↓
Spender Thrives
      ↓
Saver Locked Out

Scarce Food
      ↓
Spender Starves
      ↓
Pool Position Lost
      ↓
Saver Dominates
```

Reproduction is not just an energy decision.

It is a positional decision.

The cost of reproduction includes the loss of ecological niche.

## Law #019 — Reproduction Requires Metabolic Surplus

Earned reproduction depends on positive net energy gain.

Cells must earn energy faster than the tick drain.

**Evidence:** E019

**Consequence:**
```
Break-Even (Reward = Hops + 1)
         ↓
No Growth
         ↓
No Reproduction

Surplus (Reward > Hops + 1)
         ↓
Growth
         ↓
Reproduction
         ↓
Population Expansion
```

The loop length sets the energy barrier.

A cell in a 3-hop loop needs Reward > 4 to grow.

A cell in a 2-hop loop needs Reward > 3.

Structure still constrains evolution.

## Law #020 — Heritable Variation Creates Lineages

Offspring inherit parent traits with mutation.

Selection acts on variation across generations.

**Evidence:** E020

**Consequence:**
```
Parent
↓
Inheritance
↓
Mutation
↓
Offspring
```

Distinct lineages emerge and can diverge over generations.

---

## Law #021 — Tiny Mutations Enable Gradual Adaptation

Mutations are small steps (±5), not random jumps.

No code mutation. No assembly rewriting. Not yet.

**Evidence:** E021

**Consequence:**
```
Trait
  ↓ ±5
Trait ±5
  ↓ ±5
Trait ±10
  ↓ ±5
Trait ±15
```

Large jumps (±50) find peaks fast but overshoot.
Small steps (±5) climb peaks precisely.

Under sustained selection, increments accumulate:
```
cost 100 → 95 → 90 → 85 → 80 → 75 → 70
```

Gradualism beats saltation when the landscape is smooth.

Assembly mutation = future work.

## Law #022 — Trait Frequencies Record Evolutionary History

Track reproduction_cost, energy_capacity, affinities, mutation_rate per generation.

Record: generation, trait, value, count, frequency.

**Evidence:** E022

**Consequence:**
```
Gen 1: cost=100 (100%)
Gen 2: cost=100 (80%), cost=95 (20%)
Gen 3: cost=95 (60%), cost=90 (40%)
Gen 4: cost=90 (90%), cost=85 (10%)
```

Frequency shifts reveal selection.
A declining cost frequency = adaptation.
A rising mutation_rate frequency = evolvability selection.

Without tracking, evolution is invisible.
With tracking, evolution is measurable.

## Law #023 — Pool Order Masks Cost Advantage

A lower reproduction cost only wins if it secures earlier pool position.

**Evidence:** E023

**Consequence:**
```
cost=40 (cheaper)
  ↓ born second
  ↓ pool position 2
  ↓ FIFO access: 2/10000 signals

cost=100 (expensive)
  ↓ born first
  ↓ pool position 1
  ↓ FIFO access: 9998/10000 signals
```

Pool order (Law #005) > metabolic efficiency.
Birth sequence = ecological priority.

To evolve cheaper reproduction, a mutant must also evolve earlier spawning.

---

## Law #023 — Access Precedes Optimization

You must reach the resource before efficiency matters.

**Evidence:** E023

**Consequence:**
```
Optimization (cost=40)
  ↓ requires
Access (pool position 1)
  ↓ enables
Throughput (signals/sec)
  ↓ enables
Selection
```

No FIFO slot → zero throughput → invisible to selection.
Cost reduction on zero throughput = 0 fitness.

Access is the gate. Optimization is the path.
First get in line. Then run faster.

## Law #024 — Hard Population Caps Prevent Selective Sweeps

Population limits can suppress selection even when fitness differences exist.

**Evidence:** E024

**Consequence:**
```
Superior lineage
  ↓
Reaches cap
  ↓
Growth stops
  ↓
Inferior lineage persists
```

The bottleneck is the phenotype.
Everything else is latent potential.

## Law #025 — Removing One Bottleneck Reveals The Next

E023: Scheduler bias was the bottleneck → fair scheduling equalized throughput.
E024: Population cap became the new bottleneck → both types saturate, neither dominates.

**Evidence:** E023 → E024

**Consequence:**
```
Bottleneck 1 (scheduler) → removed → Bottleneck 2 (max_instances) → exposed
    ↓                              ↓
Cost=40 gets throughput      Cost=40 hits cap
    ↓                              ↓
Equal signals                No displacement
```

Evolution is a game of whack-a-bottleneck.
Fix the visible constraint → the hidden constraint becomes visible.
The current bottleneck *is* the selective pressure.

## Law #026 — Hard Caps Prevent Displacement

When both competitors hit `max_instances`, neither can displace the other.

Cost=40 gets 511 signals, Cost=100 gets 489 signals — but both capped at 10 cells.

**Evidence:** E024

**Consequence:**
```
Throughput advantage (511 vs 489)
         ↓
Population cap (10 each)
         ↓
Coexistence, not displacement
```

A trait cannot sweep if the population ceiling is already hit.
To displace, you must first raise the ceiling.
Carrying capacity > selective advantage.

## Law #027 — Selection Acts Through The Least-Constrained Dimension

Evolution can only operate where variation has room to express itself.

If a constraint saturates:

- scheduler
- population
- energy
- space

selection shifts elsewhere.

**Evidence:** E023 → E024

**Consequence:**
```
E023: Scheduler saturates (bias locks FIFO)
         ↓ --shuffle-->  Scheduler freed
E024: Population cap saturates (max_instances=10)
         ↓  (no more room)  Selection frozen
```

Constraint 1 bound → selection moves to Constraint 2.
Constraint 2 bound → selection moves to Constraint 3.
When all bound → stasis.

Selection is water. It flows through the cracks.
The least-constrained dimension is the phenotype.

## Law #028 — Mortality Creates Evolutionary Opportunity

Fixed population caps freeze selective sweeps.

Mortality generates continuous slot turnover → selection can act on reproductive rate.

**Evidence:** E025

**Consequence:**
```
Population capped (E024) → sweep frozen at 50/50
         ↓ add 0.1% death/tick
Mortality → slots open continuously
         ↓
Cost=40 fills slots 2.5× faster
         ↓
Cost=40 sweeps → 90%+ population
```

Without death, the superior variant waits forever for a slot that never opens.
With death, every tick is a lottery ticket. Cheaper reproduction = more tickets.
Death is not the enemy of life. Death is the engine of selection.

## Law #029 — Reproductive Rate Dominates When Slots Turn Over

When mortality creates continuous slot turnover, the cheaper reproducer wins.

**Evidence:** E025

**Consequence:**
```
Slot opens (death)
       ↓
Both types compete for slot
       ↓
Cost=40 ready in ~50 ticks (threshold=200, net=4/tick)
Cost=100 ready in ~75 ticks
       ↓
Cost=40 claims slot first → 2.5× faster
       ↓
After 1000 deaths: Cost=40 owns ~95% of population
```

Reproductive rate matters only when slots turn over.
No turnover → rate irrelevant (frozen at cap).
Turnover on → rate = selection coefficient.

The cheapest reproducer doesn't just "win" — it monopolizes the queue.

## Law #029 — Reproductive Rate Determines Lineage Expansion

**Evidence:** E025

**Consequence:**
```
Higher reproduction rate
       ↓
More offspring
       ↓
More replacement opportunities
       ↓
Greater population share
```

Under equal access to resources, faster reproduction drives selective sweeps.

## Law #030 — Every Discount Has A Cost

Cheap reproduction trades off against other fitness components.

**Evidence:** E026

**Consequence:**
```
cost=40 + energy=200    →  EXTINCT (can't reach threshold)
cost=40 + mut=0.05      →  VIABLE but degrades
cost=40 + timeout=50    →  VIABLE
cost=100 + no tradeoffs →  WINS (reliability compounds)
```

The discount is paid in a different currency.
Reliability > raw rate when variance exists.

A trait is not its best case.
A trait is its worst case under load.

## Law #030 — Fitness Is Multidimensional

Advantage minus Tradeoff equals Net Fitness.

A beneficial trait can become disadvantageous when coupled with sufficient penalties.

**Evidence:** E026

**Consequence:**
```
cost=40 (advantage)
  + energy=200 (tradeoff)
  = EXTINCT (net fitness < 0)

cost=40 (advantage)
  + mut=0.05 (tradeoff)
  = VIABLE (net fitness > 0)

cost=100 (no advantage)
  + no tradeoff (no penalty)
  = WINS (net fitness > all)
```

Selection optimizes the sum, not the parts.
A trait in isolation has no fitness.
Fitness is contextual.

## Law #031 — Reproductive Efficiency Has Viability Thresholds

Some traits have minimum viability requirements below which selection cannot operate.

**Evidence:** E026

**Consequence:**
```
Cheap reproduction (cost=40)
       +
Insufficient energy (200)
       =
EXTINCTION

Threshold not met → no reproduction → no selection.
```

Viability is binary. Efficiency is continuous.
You must survive to optimize.
A dead lineage has fitness = 0.

## Law #032 — Fitness Is A Gradient, Not A Threshold

Evolution optimizes a continuous landscape, not binary winners.

**Evidence:** E027

**Consequence:**
```
Cost:    20    40    60*   80    100   120
Fitness: ████  ████  ████████ ████  ████  ████
               ▲ peak at 60
```

The optimum is interior, not extreme.
Cost=20 fails (too unstable).
Cost=120 fails (too slow).
Cost=60 balances speed and stability.

Selection is hill-climbing on a smooth landscape.
The fittest is not the fastest nor the cheapest.
The fittest is the best-balanced.

## Law #034 — Fitness Gradients Enable Hill-Climbing

Smooth fitness gradients allow incremental optimization.

Binary win/lose → evolution stalls.
Smooth gradient → evolution walks uphill.

**Evidence:** E027

**Consequence:**
```
cost=20 → cost=40 → cost=60 (peak) → cost=80 → cost=100 → cost=120
   ↓           ↑           ↑           ↓           ↓           ↓
 too fast   better    PEAK      slower     too slow   too slow
```

Each step has a neighbor with higher fitness.
Selection has a direction at every point.
No valleys to trap. No cliffs to fall.

Evolution is a random walk with a bias.
The bias is the gradient.
The gradient is the signal.

## Law #033 — Interior Optima Emerge From Tradeoff Balance

The fitness peak occurs where opposing forces balance.

**Evidence:** E027

**Consequence:**
```
Too cheap (20)  → energy instability → crashes
Too expensive (120) → slow reproduction → misses slots
Just right (60) → stable energy + fast enough → peak
```

The optimum is where marginal benefit = marginal cost.
Not at the boundary. In the middle.
Evolution finds the compromise.

## Law #035 — Robustness Trades Against Efficiency

Cheap strategies perform well under ideal conditions.

Expensive strategies perform better under perturbation.

**Evidence:** 2D fitness surface (cost × mutation)

**Consequence:**
```
Low mutation (ideal):
    cost=20 → 62.8k signals  ← WINS (max efficiency)
    cost=60 → 66.9k signals  ← near peak
    cost=120 → 68.9k signals ← competitive

High mutation (perturbation):
    cost=20  → 18.3k signals  ← COLLAPSE
    cost=60  → 57.3k signals  ← ROBUST
    cost=120 → 50.7k signals  ← STABLE

Efficiency  = performance at ideal conditions
Robustness  = performance under stress
```

Efficiency and robustness are anti-correlated.
The fittest under calm ≠ fittest under storm.

Selection in variable environments favors robustness.
Selection in stable environments favors efficiency.
The environment chooses the currency.

## Law #036 — Fitness Landscapes Are Surfaces, Not Curves

Multi-trait selection creates fitness surfaces with ridges, valleys, and saddle points.

**Evidence:** 2D cost × mutation surface

**Consequence:**
```
1D (cost only):  smooth hill, peak at 60
2D (cost, mut):  ridge along low-mut, cliff at high-mut
                 saddle at (cost=20, mut=0.10)
```

1D projection hides structure.
2D reveals ridges, valleys, escape routes.

Evolution walks on surfaces, not paths.
Dimensionality = degrees of freedom = evolvability.

## Law #037 — Robustness Trades Against Efficiency

Peak performance and fault tolerance are often opposing pressures.

**Evidence:** 2D cost × mutation surface

**Consequence:**
```
Low mutation (calm):
    cost=20 → 62.8k  ← MAX EFFICIENCY
    cost=60 → 66.9k
    cost=120 → 68.9k

High mutation (storm):
    cost=20  → 18.3k  ← COLLAPSE
    cost=60  → 57.3k  ← ROBUST
    cost=120 → 50.7k  ← STABLE
```

Efficiency = performance at ideal conditions.
Robustness = performance under stress.
Anti-correlated. The fittest under calm ≠ fittest under storm.

Selection in variable environments favors robustness.
Selection in stable environments favors efficiency.
The environment chooses the currency.

## Law #038 — Mutation Sensitivity Defines Landscape Topology

Traits with high mutation sensitivity create cliffs.

Traits with low mutation sensitivity create plateaus.

**Evidence:** 2D cost × mutation surface

**Consequence:**
```
cost=20:  62.8k → 18.3k  (71% drop)  ← CLIFF
cost=60:  66.9k → 57.3k  (14% drop)  ← PLATEAU
cost=120: 68.9k → 50.7k  (26% drop)  ← SLOPE
```

Cliffs trap lineages. Plateaus allow exploration.
Landscape topology IS the distribution of mutation sensitivities.

## Law #039 — Evolution Prefers Reachable Peaks

A slightly lower but broad plateau is often easier to discover and remain on than a razor-thin optimum.

**Evidence:** 2D cost × mutation surface

**Consequence:**
```
cost=20:  peak=62.8k, width=0  ← razor peak, unreachable under mutation
cost=60:  peak=66.9k, broad   ← reachable plateau, sustainable
cost=120: peak=68.9k, broad   ← reachable plateau, sustainable
```

Discovery probability × retention time > peak height.

A peak you can't reach has fitness = 0.
A peak you fall off has fitness = 0.
A plateau you stay on has fitness = height × time.

## Law #040 — Evolutionary Speed Is Itself Evolvable

Mutation size, mutation rate, and decay rate are parameters subject to selection.

**Evidence:** E028 vs E029

**Consequence:**
```
Slow params (E028):  ±5 steps, 0.99 decay → 150,000 ticks to peak
Fast params (E029):  ±20 steps, 0.95 decay → ~50,000 ticks to peak

Speedup: ~3×
```

Evolution discovers not just *where* the peak is, but *how fast* to climb it.

Lineages with evolvable parameters outcompete fixed-parameter lineages.
The speed of evolution is itself a selected trait.

Evolution finds what it can keep.

## Law #042 — Evolutionary Speed Is Itself Evolvable

Mutation size, mutation rate, and decay rate are parameters subject to selection.

**Evidence:** E028 vs E029

**Consequence:**
```
Slow params (E028):  ±5 steps, 0.99 decay → 150,000 ticks to peak
Fast params (E029):  ±20 steps, 0.95 decay → ~50,000 ticks to peak

Speedup: ~3×
```

Evolution discovers not just *where* the peak is, but *how fast* to climb it.

Lineages with evolvable parameters outcompete fixed-parameter lineages.
The speed of evolution is itself a selected trait.

Evolution finds what it can keep.

## Law #041 — Convergence Requires Deep Time

Fitness peaks exist. Gradients exist. Selection exists. Yet convergence takes ~150,000 ticks.

**Evidence:** E028

**Consequence:**
```
Peak at cost=60
    ↓
Start at cost=120
    ↓
±5 steps/gen, 100 ticks/gen
    ↓
120 → 60 = 12 steps × 100 gens × 100 ticks = 120,000 ticks
```

Variation exists. Selection exists. Gradient exists.
But the walk is slow.

Deep time is not a luxury.
Deep time is a requirement.

## Law #043 — Robustness Trades Against Efficiency

Cheap strategies perform well under ideal conditions.

Expensive strategies perform better under perturbation.

**Evidence:** 2D cost × mutation surface (E028-E029)

**Consequence:**
```
Low mutation (calm):
    cost=20 → 62.8k signals  ← WINS (max efficiency)
    cost=60 → 66.9k signals  ← near peak
    cost=120 → 68.9k signals ← competitive

High mutation (storm):
    cost=20  → 18.3k signals  ← COLLAPSE
    cost=60  → 57.3k signals  ← ROBUST
    cost=120 → 50.7k signals  ← STABLE
```

Efficiency = performance at ideal conditions.
Robustness = performance under stress.
Anti-correlated. The fittest under calm ≠ fittest under storm.

Selection in variable environments favors robustness.
Selection in stable environments favors efficiency.
The environment chooses the currency.

## Law #044 — Heritable Evolutionary Parameters Are Themselves Subject to Selection

Mutation step size, mutation rate, and decay rate evolve under selection.

**Evidence:** E028 vs E029 (heritable parameters)

**Consequence:**
```
E028 (fixed params):   150,000 ticks to converge
E029 (heritable params):  ~50,000 ticks (3× faster)
```

Lineages that evolve better search strategies outcompete fixed-parameter lineages.

The speed of evolution is itself a selected trait.
Evolution searches for better search algorithms.

## Law #045 — Fast Meta-Evolution Wins in Stable Environments

Small mutation steps, rapid decay, high meta-mutation outcompete conservative strategies.

**Evidence:** E031 (meta-strategy competition)

**Consequence:**
```
fast_mut (step=5, decay=0.90, meta=0.01)  →  WINNER
balanced (step=20, decay=0.95, meta=0.001) →  2nd
slow_mut (step=50, decay=0.99, meta=0.0001) →  3rd
conservative (step=10, decay=0.98, meta=0.0005) →  4th
```

The system discovers that **fast adaptation beats optimization** when the environment is stable but mortality creates turnover.

---

## Law #046 — Single-Producer Bottleneck Caps Throughput

One producer → 1000 consumers → 1 closer: **470K signals total**

Throughput limited by producer FIFO (capacity 3), not consumer diversity.

Adding more consumer types does not increase total signal throughput.

---

## Law #047 — Mortality + Earned Reproduction = Adaptability Selection

Turnover from death + reproduction creates selection for fast adaptation, not just high fitness.

**Evidence:** E031 — fast_mut wins despite not being "most fit" in static sense.

---

## Law #048 — Environmental Variation Maintains Genetic Diversity

Periodic stress prevents fixation on narrow optima.

**Evidence:** E032 — predictable 5K/5K/5K shift maintains broader trait distributions than static environment.

---

## Law #049 — Stress Selects for Adaptability Over Optimization

High-mortality phases favor fast mutation, rapid decay, high meta-mutation.

**Evidence:** E032 — strategy_fast wins in shifting environment; strategy_slow wins in static phases.

---

## Law #050 — Over-Optimization Reduces Adaptability

Lineages that converge perfectly to a static peak go extinct when the peak moves.

**Evidence:** E032 — strategy_slow (optimized for benign) collapses during stress phase.

---

## Law #051 — Mutation Rate Evolves to Minimum During Stability, Retains Variation for Shifts

μ → 0.001 in calm, but standing variation persists for rapid response.

**Evidence:** E032 — 50% of population at μ=0.001, tail to 0.03 maintained.

---

## Law #052 — Unpredictability Maintains Broader Distributions Than Predictable Cycles

Stochastic shocks → wider trait spreads than scheduled shifts of equal mean intensity.

**Evidence:** E033 vs E032 comparison — mutation_rate tail 0.047 vs 0.077, cost spread wider under uncertainty.

---

## Law #053 — Stochastic Shocks Select for Bet-Hedging

Broader mutation rate distributions persist as insurance against unknown shock timing.

**Evidence:** E033 — 50% at μ=0.001 but long tail maintained; E032 (predictable) collapses tighter.

---

## Law #054 — Uncertainty Prevents Tight Convergence

Even when mean selection pressure is identical, unpredictable timing preserves variance.

**Evidence:** E033 vs E032 — same strategies, same mean death_rate, different temporal structure → different equilibria.

---

## Law #055 — Recovery Time Is Signal-Limited, Not Mortality-Limited (When at Carrying Capacity)

At population cap with FIFO bottleneck, shocks don't reduce population below 90% threshold.

**Evidence:** E033b — 104 shocks, recovery mean=0.5 ticks (population never drops below 90%).

The bottleneck is signal throughput, not cell survival. Recovery metrics only meaningful when population can actually decline.

---

## Law #056 — Predictive Information Has Fitness Value

A cell that stores recent signal history and emits a protective signal upon detecting a warning (before mortality rises) gains fitness value from prediction, independent of whether the predicted threat fully materializes.

**Evidence:** E034 — `memory_cell` (stores last 16 signal types in state, affinity `[9]`) receives broadcast warning type-9 at tick 101, emits predictive broadcast type-50. L#056 discovered at tick 101 (`{"law_id":"L056","tick":101}`). `zmon --dashboard` marks it `● L056 Predictive Information Has Fitness Value` (part of 3/56 discovered). Contrast lineage `reactive_cell` (sensor, no memory) only reacts to present conditions.

Mechanism: memory cell ABI — pop signal, record type_id in ring buffer (state[16..31], write_count at state[0], warned_flag at state[1]); on detecting type 9 in history and not yet warned this episode, emit broadcast type-50. Runtime trigger: emission of type-50 after `warning_emitted` (also on delivery).

---

## Law #057 — Predictive Information Increases Fitness In Partially Predictable Environments

```
warning
   ↓
memory
   ↓
brace
   ↓
higher survival
   ↓
more descendants
```

When the environment is *partially predictable* — a warning reliably (but not always) precedes a shock — a cell that retains signal history and uses it to brace *before* mortality rises out-survives and out-reproduces one that only reacts to present conditions. The causal chain is: **perceive warning → store/recognize it (memory) → preemptively prepare (brace) → pay lower mortality during the shock → leave more offspring**.

**Why "partially" matters:** Full predictability would make memory trivially dominant; zero predictability (shocks with no warning) would make memory pure cost with no payoff. The fitness gain is maximal in the *intermediate* band where warnings precede shocks often enough to repay the maintenance tax but not so perfectly that reactive strategies can't persist at all. The 20% false-alarm rate in E035 is exactly this band: memory pays +2/tick always, but the 80% real-warning rate makes the brace's 0.25× mortality discount a net positive.

**Evidence:** E034 (L#056 grounding) + E035 — memory lineage surged from a 1-cell minority to 84% within 100 ticks and fixed at 100% by ~tick 2500. 100% of deaths were shock-driven mortality; memory's predictive brace was the decisive selective advantage. Memory_cost (+2/tick, "memory is not free") and the 20% false-alarm tax were insufficient to overturn selection.

**Contrast / boundary:** Under fully *unpredictable* lethal shocks (no warning, high death_rate), memory collapses with everything else — the brace can't help if there's nothing to brace against. Memory is an *investment* with positive expected return only where warnings precede shocks.

---

## Law #058 — Lineage Crossover Flows Toward the Fittr Strategy

When offspring can mutate between two strategies (reactive ↔ memory), the crossover does not settle into a stable polymorphic mix — it flows toward the fitter lineage and drives the unfit one extinct.

**Evidence:** E035 — 290 lineage switches fired; the `sibling`-field crossover mechanism works. Starting from equal 1-cell seeds, memory (fitr under warning→shock environment) absorbed the reactive lineage rather than coexisting with it. Caps that froze each lineage at 200/200 (L#026) *hid* this; raising `max_instances` revealed fixation.

**Implication:** Hereditary strategy switching is not a diversifying force by itself — given a consistent environment it is a *selective sweep accelerator*. Stable polymorphism requires either a fluctuating environment (L#049/L#050) or a frequency-dependent fitness.

---

## Law #059 — Fixation Reduces Diversity And Creates Systemic Extinction Risk

Selection that maximizes lineage fitness can minimize ecosystem robustness. Once memory crowded out reactive in E035, a shock the now-low-diversity population could not absorb caused *total* collapse (population → 0 by ~tick 940 under harsher shocks), even though memory was individually fitter at every prior tick.

**Evidence:** E035 failure-mode run (shock death_rate 0.10) — memory reached 84% by tick 100, then the whole system went extinct; contrast with the tuned run (death_rate 0.04) where coexistence-then-fixation persisted to 800 cells.

**Implication:** Lineage-level fitness and ecosystem-level robustness are distinct, often opposed, objectives. A lineage that "wins" the selection game can doom the substrate it depends on — an early echo of diversity-stability tradeoffs in ecological/evolutionary systems.

---

## Law #060 — Communication Converts Individual Information Into Shared Information

When a cell that perceives a warning emits a *signal* (rather than only self-bracing), its prediction becomes accessible to other cells — individual information becomes *shared* information. The receiving cell braces on the *received* signal, not on its own perception.

**Evidence:** E036 — runtime modified so `prepared_ticks` is granted to **recipients** of type-50 (not just the emitter). Result: 806 `memory_cell --[50]--> listener_cell` deliveries; L#060 fired at runtime. The protective prediction made by one memory cell demonstrably protected cells of a *different lineage*.

**Mechanism:** `distribute_signals` now sets `cell.prepared_ticks = PREPARED_DURATION` on every cell that *receives* a type-50, in both broadcast and directed branches. L#060 discovered when a type-50 delivery reaches a cell whose lineage ≠ the emitter's.

**Boundary:** The channel is real and used, but — see L#062 — sharing the benefit does not enrich the lineage that *pays* to produce it.

---

## Law #061 — A Single Predictor Can Protect A Whole Lineage

One cell that bears the cost of perception+prediction can, via a broadcast warning, brace every listener in the population each shock cycle. The protective effect is *amplified* across the lineage by communication.

**Evidence:** E036 — a handful of `memory_cell`s (clamped at 200) emitting type-50 each warning cycle braced the entire listener+reactive population holding affinity 50 through every shock. The predictive labor of few subsidized the survival of many.

**Implication:** Predictive capacity is a *public good* once communication exists. Whoever produces it creates a commons.

---

## Law #062 — Cheap Listeners Free-Ride On Expensive Predictors

When the protective signal is *shared* (L#060/L#061), the lineage that *pays* to perceive+predict (memory, +2/tick) is suppressed, while lineages that merely *consume* the signal — or ignore it entirely — capture the benefit without the tax.

**Evidence:** E036 — `listener_cell` (free brace on receiving type-50, no memory cost) *rose then declined* (53→192→85); `memory_cell` clamped at its 200 cap (producer, taxed); and `reactive_cell` (no memory, no listening) *won* (10→315). The naive prediction ("free protection should make listeners dominate") was wrong: the listener still competes for the food FIFO with no steady-state edge, while reactive is unburdened by *either* cost. The producer is suppressed; the cheapest non-contributor wins.

**Implication:** This is the dark twin of L#057. Shared information *dissipates* the fitness of its source. Communication evolves (the channel is real and used) but does **not** enrich the lineage that invented it — a public-goods tragedy at the level of selection. Stable polymorphism here required the producer's cap; without it, the taxed predictor is driven toward extinction by its own commons.

---

## Law #063 — Selective Communication Protects Cooperative Information Systems

```
broadcast information
   ↓
benefits diffuse
   ↓
free riders prosper
   ↓
producers decline

selective information
   ↓
benefits remain local
   ↓
contributors prosper
   ↓
cooperation persists
```

If a predictive signal is *directed* only toward cooperating recipients (kin / contributors / trusted) rather than broadcast to all cells, the fitness benefit generated by the information remains within the communicating lineage. This suppresses free-rider expansion and stabilizes cooperative information exchange.

**Evidence:** E037 — `memory_cell` given `recipient_policy: "kin"` under `--directed-warning`; its type-50 is rewritten by the runtime into DIRECTED copies to alive same-lineage cells only (382 `memory_cell --[50]--> memory_cell` edges; L#063 fired). Head-to-head vs E036 (broadcast): the cheap non-contributor `reactive_cell` collapsed from **315 → 23** (no longer receives the free public good), while the listener lineage rose to dominate (341). Memory retained a healthy 236 (not clamped-collapsed). The predictor's benefit was *recaptured* by excluding free-riders.

**Mechanism:** `Gene.recipient_policy` + `Runtime.directed_warning`. Per tick the runtime precomputes `kin_recipients: HashMap<lineage, Vec<cell_id>>`; a "kin" emitter's type-50 broadcast is split into one directed signal per same-lineage alive cell (recipients still brace on receipt via L#060). Implemented at the runtime layer (which knows lineages) so the cell ABI stays stable (L#007).

**Implication:** Information's *value* is a function of **who receives it**, not merely that it is transmitted. Broadcasting maximizes reach but diffuses benefit to free-riders (L#062); selective sharing keeps the benefit local so contributors prosper and cooperation persists. The "best" communication protocol is environment-dependent: broadcast when predictors are abundant and cheap to replace; directed when the predictor is scarce and taxed. A third axis (contributor-only, trust-graph) remains untested.

## Law #064 — Information Ecosystems Require A Minimum Producer Fraction

An information-sharing ecosystem (a population sustained by a protective signal network) collapses once the fraction of *producing* cells falls below a critical threshold. Below the threshold the warning network goes dark; above it, the network self-sustains.

```
producer fraction < threshold
    ↓
warning frequency collapses
    ↓
protection collapses
    ↓
(cooperative) population collapses
```

**Measured threshold (E038):** In the directed-kin setup (E037 genome), applying extra per-tick mortality only to producers (`is_memory`, via `--producer-death-bonus`) locates the cliff precisely:
- bonus **0.1%/tick** → producers *stabilize* at ~22–37% of the population; ~1.36M directed type-50 edges over 3000 ticks; warning network healthy.
- bonus **0.2%/tick** → producers *erode to 0%* by ~tick 2000; warning network goes dark.
- bonus **≥0.3%/tick** → producers crash to ~2% within 500 ticks and go extinct.

The collapse cliff lies between **0.1% and 0.2% extra producer mortality per tick** — i.e. the producer fraction must stay above roughly **0.7% of the population (~4 of 600 cells)** or the directed-warning network extinguishes. This is the measurable minimum-producer-fraction law: a small protected producer class can sustain the whole ecosystem, but once suppressed below the cliff the information good vanishes.

**Evidence:** E038 — `memory_cell` (producer, `recipient_policy:"kin"`) carries `--producer-death-bonus` ∈ {0.0, 0.001, 0.002, 0.003, 0.005, 0.015, 0.03}. Snapshots emit `producer_fraction` (new field). At bonus ≥ 0.002 the producer fraction trends to 0 and type-50 edge count falls to 0 (at bonus 0.03: 0 edges vs 1.36M at bonus 0.0).

**Mechanism:** Producers pay a standing `memory_cost` (energy/tick) *and* now an extra mortality load. When that load exceeds the lineage's reproductive replacement rate, producers are driven extinct; with no emitter, `distribute_signals` produces no directed type-50, so no cell braces, so the protective signal network (the information ecosystem) ceases to exist.

**Implication (the free-rider mirror of L#063):** Selective communication (L#063) protects producers from *external* free-riders, but a producer class can still be collapsed by *internal* suppression (extra mortality / cost). An information ecosystem is only as durable as its minimum viable producer fraction. Unchecked free-rider expansion (L#062) or direct producer suppression both push the fraction under the cliff — **unchecked free riders (or producer loss) collapse information ecosystems.** The threshold is *quantifiable*, making this the first fully measured law in the catalog.

(End of file)
