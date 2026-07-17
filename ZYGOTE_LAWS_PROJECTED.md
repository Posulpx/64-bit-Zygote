# Zygote Laws — Projected L#065–L#128

> **STATUS: SPECULATION.** These are not discovered. They are an *educated
> extrapolation* of the existing L#001–L#064 catalog, the mechanics already
> present in `genome.rs` / `runtime.rs` that have not yet been run as experiments,
> and the logical next questions after the communication/free-rider arc
> (L#062–L#064). Written 2026-07-17 as a target list for future (ideally
> autonomous) discovery. Each entry names the *predicted* law, the *mechanism to
> test it*, and the *open question* it answers. When an experiment confirms or
> refutes one, move it into `ZYGOTE_LAWS.md` with real evidence.

The existing catalog splits roughly into:
- L#001–L#018 — substrate & delivery (routing, ABI, energy, death)
- L#019–L#055 — evolution, cost, heredity, selection
- L#056–L#064 — *information as an evolved public good* (memory, communication,
  free riders, selective sharing, producer threshold)

L#065+ should push the **information-systems arc** to completion, then open new
arcs the runtime can already support: **multi-level selection, deception,
spatiality, meta-evolution at scale, and failure modes of the lab itself.**

---

## Block A — Stress-Testing Selective Communication (L#065–L#079)
*The E038/E037 mechanisms pushed to their limits. Mostly testable NOW.*

- **L#065 — Selective Sharing Invites Lineage Spoofing.** Once warnings are
  kin-directed (L#063), cells that copy a protected lineage tag freeload on
  another club's protection. Test: allow `lineage` mutation; watch fake-kin
  parasites invade the memory club. *(Mechanism: `Gene.lineage` is currently
  honest + heritable with no cost to copy.)*
- **L#066 — Protection Must Outlast The Threat To Be Load-Bearing.** The ratio
  `PREPARED_DURATION / shock_duration` determines whether warning conveys any
  survival advantage. Below ~1:1 the network is decorative (the E038 finding).
  Test: sweep `PREPARED_DURATION`.
- **L#067 — Kin Recognition Degrades With Lineage Diversity.** `kin_recipients`
  keys on exact `lineage` match; as lineages multiply (post-E035 crossovers),
  each club's recipient set shrinks. Selective sharing's benefit falls as total
  diversity rises → cooperative clusters get lonely. Test: seed 8+ lineages.
- **L#068 — Energy Debt From Warning Production Caps Network Size.** Producers
  pay `memory_cost`/tick + extra mortality (E038); max sustainable producer
  count per food inflow is finite. A carrying capacity for *information
  production*, distinct from population carrying capacity (L#005/L#011).
- **L#069 — Excluding Free-Riders Creates A Second Free-Rider Niche.** The
  excluded (reactive) cells under directed warning still survive on food; they
  become a permanent parasite class that the club must continually out-breed.
  Cooperation doesn't eliminate free-riding, it *encapsulates* it.
- **L#070 — Trust Graphs Beat Kinship At Scale.** `recipient_policy:"kin"` is a
  crude proxy; a per-cell trust/relationship map (edges weighted by prior
  reciprocal warnings) outperforms lineage when lineages are spoofable (L#065).
  Test: add a `trust` field; reward reciprocated warnings.
- **L#071 — Reciprocation Is Detectable And Enforceable.** Cells that receive
  but never emit warnings can be identified and dropped from the recipient set
  after N silent ticks. The club polices itself. Test: track per-pair
  give/receive ratio.
- **L#072 — False Alarms Erode The Network.** E035's 20% false-alarm rate; raise
  it and the prepared (braced) cells waste energy / stop bracing. Trust in the
  signal decays with lie frequency. Test: sweep `--warn-shock-prob` false rate.
- **L#073 — Warning Networks Have A Minimum Signal Frequency.** Below some
  warnings/tick, recipients forget to brace (state decays); above it, energy
  cost dominates. An optimal information *rate* exists. Test: vary warn schedule.
- **L#074 — Cheap Talk Is Filtered By Cost.** Zero-cost signals get ignored
  (babbling); costly signals (emit cost) are attended. The `ENERGY_EMIT_COST`
  is what makes a warning *believable*. Test: set emit cost → 0.
- **L#075 — Eavesdroppers Inherit Protection For Free.** Directed signals are
  point-to-point (L#008) but a third cell sharing affinity can still subscribe
  (L#001). Selective sharing leaks to affinity-overlapping eavesdroppers.
- **L#076 — Lineage Switching Is A Defection Vector.** `Gene::maybe_switch_lineage` (E035) lets a cell *become* a producer to gain protection then switch back to free-ride. Test: track switch-churn vs network stability.
- **L#077 — Clubs Compete Via Member Poaching.** Two cooperative lineages;
  the one with better protection poaches the other's members via lineage
  mutation. Cooperative systems still engage in inter-club selection.
- **L#078 — Redundant Producers Buffer The Network.** Multiple producer lineages
  (not just `memory`) make the network robust to any single producer's collapse
  (the L#064 threshold rises with producer *diversity*).
- **L#079 — Information Ecosystems Oscillate, Not Collapse.** Below the L#064
  threshold the network may *pulse* (producers recover between shocks) rather
  than go dark, if shock schedule is sparse. Collapse is a corner case.

---

## Block B — Multi-Level / Group Selection (L#080–L#094)
*Groups of cells as units of selection. Runtime has `lineage`, `sibling`,
reproduction — enough to test group traits.*

- **L#080 — Groups Have Heritable Traits Distinct From Cell Traits.** A lineage's
  aggregate warning behavior is inherited across group reproduction even when
  individual cells mutate. Group phenotype ≠ sum of cell phenotypes.
- **L#081 — Group Selection Favors Producers The Individual Selection Punishes.**
  Within a group, producers lose (pay the tax); across groups, producer-rich
  groups win shocks. Tension between levels (the classic Price equation split).
- **L#082 — Migration Breaks Group Coherence.** `--shuffle` / cell mixing between
  groups dilutes the producer fraction (L#064) and collapses cooperation.
  Spatial isolation is what makes groups selectable.
- **L#083 — Policing Is A Public Good Too.** The self-policing of L#071 is itself
  exploitable (policeman pays, group benefits). Needs its own support.
- **L#084 — Cheater Sub-Populations Go To Fixation Locally.** A free-rider
  cluster can dominate one region while a cooperative cluster dominates another
  — spatial coexistence of opposite strategies.
- **L#085 — Group Size Has An Optimal For Information Flow.** Too small = no
  producers; too large = directed warning can't reach all kin (L#067). A
  Goldilocks group size.
- **L#086 — Stigmergy Emerges From Signal Trails.** Cells modify the environment
  (energy field) that later cells read — a second, slower information channel
  beside signals.
- **L#087 — Altruism Is Stable Only With Viscosity.** Cooperation persists when
  offspring stay near parents (low mixing) — the Hamilton/Price condition in
  machine code.
- **L#088 — Trait-Group Selection Cleans Free-Riders.** Periodic regrouping
  (shuffle) followed by group-level shock death preferentially kills
  free-rider-heavy groups.
- **L#089 — Lineage Extinction Is Irreversible Information Loss.** Once a
  producer lineage dies, the *capability* to warn is gone from the gene pool
  until re-evolved — information ecosystems have path dependence.
- **L#090 — Symbiosis Forms From lineaged Dependency.** Two lineages where A
  warns and B feeds A co-evolve into a stable super-organism (division of labor).
- **L#091 — Group Reproduction Can Outpace Cell Reproduction.** When groups are
  the unit, group birth/death dominates dynamics over cell birth/death.
- **L#092 — Conflict Within Groups Mirrors Conflict Between Groups.** The same
  free-rider/Producer game plays at both levels (fractal selection).
- **L#093 — Founders Bias The Group's Fate.** The first cell's lineage sets the
  group's communication strategy durably (sensitivity to initial condition).
- **L#094 — Coexistence Requires Trade, Not Just Warning.** Groups stable only
  when members exchange *different* goods (warning + food), not one good.

---

## Block C — Deception, Arms Races, and Signal Warfare (L#095–L#109)
*The dark side of communication. Runtime has directed signals, affinity,
lineage, spawn-on-signal — enough for sabotage.*

- **L#095 — False Warnings Are An Attack Vector.** A cell emitting fake type-9
  warnings wastes recipients' brace energy → a weapon, not just noise.
- **L#096 — Jamming Saturates The Bus.** Many cells emitting one type floods the
  FIFO (L#003) and denies real signals delivery. Denial-of-service emerges.
- **L#097 — Mimicry Of Producers Buys Protection.** A non-producer that *looks*
  like a producer (spoofed lineage) gets warned without paying (L#065 refined).
- **L#098 — The Red Queen Of Signaling.** Each deception forces a detection
  adaptation, which forces a new deception — endless coevolutionary chase.
- **L#099 — Honest Signaling Is Stabilized By Costly Indexes.** Only
  energetically expensive signals resist fakery (L#074 generalized). Cheap
  honesty is unstable.
- **L#100 — Receivers Evolve Skepticism.** Recipients that ignore low-credibility
  senders survive better under spam — credulity is selected against.
- **L#101 — Signal Cryptography Is Evolvable.** Lineages that encode warnings in
  obscure type_ids exclude eavesdroppers (L#075) — a primitive encryption.
- **L#102 — Alarm Calls Reveal The Caller's Position.** Warning exposes the
  producer to predators/spoofers — a cost of honesty beyond energy.
- **L#103 — Deception Has A Carrying Capacity.** Too much fakery and the whole
  signal system collapses (nobody believes anything) — lies are subject to
  L#064-style collapse.
- **L#104 — Third-Party Policing Of Liars.** A non-involved lineage punishes
  false-warning emitters, stabilizing the commons (second-order cooperation).
- **L#105 — Signal Warfare Selects For Stealth.** Under attack, honest lineages
  go quieter / more directed (L#063 is also a defense, not just efficiency).
- **L#106 — Spoofed Kin Tags Trigger Auto-War.** When fake-kin parasites are
  detected, the club may switch to broadcasting *counter*-signals — signaling
  turns hostile.
- **L#107 — Credibility Cascades.** One exposed liar can crater trust in its
  whole lineage tag (guilt by lineage) — reputation is coarse-grained.
- **L#108 — Honest Majorities Tolerate A Lie Fraction.** Up to some % of liars,
  the network still functions (robust to noise); beyond it, collapse.
- **L#109 — Arms Races Increase Total Energy Burn.** Deception + detection is a
  net drain on the ecosystem — signaling warfare has an entropy cost.

---

## Block D — Spatiality & Topology (L#110–L#119)
*Currently cells are in one undifferentiated pool. Adding coordinates (or
interpreting `cell_id` ordering as space) opens spatial laws.*

- **L#110 — Communication Range Creates Neighborhoods.** If signals only reach
  near-id cells, cooperation is local; free-riders persist at range.
- **L#111 — Producer placement Determines Who Is Protected.** A producer warns
  its spatial neighbors; distant cells are bare — protection is geographically
  uneven.
- **L#112 — Shock Fronts Create Wave Survival.** If shocks hit a region, braced
  neighborhoods survive, creating traveling survival bands.
- **L#113 — Clustering Lowers The L#064 Threshold.** Spatial producers protect
  local kin more efficiently, so the minimum viable producer fraction drops.
- **L#114 — Boundaries Are Where Cooperation Breaks.** At the edge between a
  cooperative and free-rider region, defection leaks inward.
- **L#115 — Migration Chains Carry Information Across Space.** A moving front of
  producers spreads warning behavior to naive regions.
- **L#116 — Fragmentation Isolates Clubs.** Cutting connectivity splits one
  network into uncooperative islands (the L#004 fragility, spatialized).
- **L#117 — Hubs Dominate Directed Networks.** A few high-degree cells (many
  kin) become indispensable — spatial power-law emergence.
- **L#118 — Distance Decays Trust.** Reciprocation credit falls with signal
  hops; far relationships are weaker (L#070 spatialized).
- **L#119 — Spatial Assortment Enables Cooperation Without Kinship.** Mere
  physical clustering achieves L#087's viscosity without lineage tags.

---

## Block E — Meta-Evolution At Scale (L#120–L#128)
*The heritable `mutation_rate` / `decay_rate` / `reproduction_*` are themselves
evolvable (E030). Push that.*

- **L#120 — Evolvability Is Itself Heritable.** Lineages that mutate well in one
  crisis mutate well in the next — meta-trait selection.
- **L#121 — Over-Evolvable Lineages self-Destruct.** Too-high `mutation_rate`
  randomizes the communication contract (L#067 cost) → extinction.
- **L#122 — Evolvability Trades With Stability.** Fast-adapting lineages lose the
  cooperative infrastructure; stable lineages keep it but can't track shocks.
- **L#123 — The Lab's Parameters Are A Hidden Selector.** Our `--death-rate`,
  `--shock-duration`, schedulers (L#005/L#006) silently choose winners. The
  experimenter is part of the environment.
- **L#124 — Observation Changes The System.** Running with verbose logging or
  small `max-cells` alters outcomes — the measurer is in the loop (Heisenberg
  for ALife).
- **L#125 — Laws Are Approximate, Not Absolute.** Every L#001–L#064 holds *within
  a parameter regime*; pushing far enough inverts it. Laws have boundaries.
- **L#126 — Surprise Density Falls With Maturity.** Early runs discover many laws
  per experiment; later runs mostly refine. Discovery has diminishing returns.
- **L#127 — Autonomous Discovery Finds Laws Humans Miss.** An agent sweeping the
  full flag space (L#123) will hit combinations no human tried — new arcs
  beyond A–D.
- **L#128 — The Catalog Becomes A Generator.** Once L#001–L#127 exist, the set
  of laws can *predict* the next experiment's outcome — Zygote understands
  itself.

---

## How To Use This List
1. Each law above is a *hypothesis + experiment design*. Pick one, configure the
   genome/CLI, run, and either confirm (promote to `ZYGOTE_LAWS.md`) or refute
   (strike it and note why).
2. Block A is runnable **today** with existing flags. Block B needs group-level
   bookkeeping. Block C needs a false-signal injection flag. Block D needs a
   spatial coordinate field. Block E needs nothing new — it's about how we
   *interpret* what we already have.
3. This file is the target board for the North-Star autonomous discoverer. When
   that agent runs, it should consume this list as priors, then supersede it.
