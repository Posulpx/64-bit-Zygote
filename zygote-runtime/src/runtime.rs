use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::{BufWriter, Write};
use crate::cell::*;
use crate::genome::*;
use crate::observer::Observer;
use crate::signal::*;
use crate::types::*;

/// Debug-only eprintln!. Prints only when `rt.verbose` is true. Replaces the
/// old unconditional debug spam that dominated I/O on long runs. Usage mirrors
/// eprintln! but the first arg must be the `&Runtime` (so it can check the flag).
macro_rules! dlog {
    ($rt:expr, $($arg:tt)*) => {
        if $rt.verbose { eprintln!($($arg)*); }
    };
}

#[derive(Debug)]
pub struct EventWriter {
    writer: BufWriter<File>,
}

impl EventWriter {
    pub fn new(path: &str) -> std::io::Result<Self> {
        let file = File::create(path)?;
        Ok(Self { writer: BufWriter::new(file) })
    }

    pub fn write_tick(&mut self, tick: u64, population: usize, energy: u64, signals_emitted: u64, signals_delivered: u64) -> std::io::Result<()> {
        let event = serde_json::json!({
            "type": "tick",
            "tick": tick,
            "population": population,
            "energy": energy,
            "signals_emitted": signals_emitted,
            "signals_delivered": signals_delivered,
        });
        writeln!(self.writer, "{}", event)
    }

    pub fn write_birth(&mut self, tick: u64, cell_id: u32, cell_type: &str, parent_id: Option<u32>, generation: u32, reproduction_cost: u32, reproduction_threshold: u32, mutation_rate: f64, initial_energy: u32, affinities: &[u8], mutation_step: u32, decay_rate: f64, meta_mutation_rate: f64) -> std::io::Result<()> {
        let event = serde_json::json!({
            "type": "birth",
            "tick": tick,
            "cell_id": cell_id,
            "cell_type": cell_type,
            "parent_id": parent_id,
            "generation": generation,
            "reproduction_cost": reproduction_cost,
            "reproduction_threshold": reproduction_threshold,
            "mutation_rate": mutation_rate,
            "initial_energy": initial_energy,
            "affinities": affinities,
            "mutation_step": mutation_step,
            "decay_rate": decay_rate,
            "meta_mutation_rate": meta_mutation_rate,
        });
        writeln!(self.writer, "{}", event)
    }

    pub fn write_death(&mut self, tick: u64, cell_id: u32, cell_type: &str, cause: &str, age: u64, generation: u32) -> std::io::Result<()> {
        let event = serde_json::json!({
            "type": "death",
            "tick": tick,
            "cell_id": cell_id,
            "cell_type": cell_type,
            "cause": cause,
            "age": age,
            "generation": generation,
        });
        writeln!(self.writer, "{}", event)
    }

    pub fn write_signal(&mut self, tick: u64, signal_id: u64, signal_type: u8, source_id: u32, source_type: &str, target_id: u32, target_type: &str, delivered: bool) -> std::io::Result<()> {
        let event = serde_json::json!({
            "type": "signal",
            "tick": tick,
            "signal_id": signal_id,
            "signal_type": signal_type,
            "source_id": source_id,
            "source_type": source_type,
            "target_id": target_id,
            "target_type": target_type,
            "delivered": delivered,
        });
        writeln!(self.writer, "{}", event)
    }

    pub fn write_reproduction(&mut self, tick: u64, parent_id: u32, parent_type: &str, child_id: u32, child_type: &str, parent_generation: u32, child_generation: u32, cost: u32, threshold: u32, mut_rate: f64, mutations: &[String]) -> std::io::Result<()> {
        let event = serde_json::json!({
            "type": "reproduction",
            "tick": tick,
            "parent_id": parent_id,
            "parent_type": parent_type,
            "child_id": child_id,
            "child_type": child_type,
            "parent_generation": parent_generation,
            "child_generation": child_generation,
            "cost": cost,
            "threshold": threshold,
            "mut_rate": mut_rate,
            "mutations": mutations,
        });
        writeln!(self.writer, "{}", event)
    }

    pub fn write_shock(&mut self, tick: u64, death_rate: f64, duration: u64, pre_shock_population: usize) -> std::io::Result<()> {
        let event = serde_json::json!({
            "type": "shock",
            "tick": tick,
            "death_rate": death_rate,
            "duration": duration,
            "pre_shock_population": pre_shock_population,
        });
        writeln!(self.writer, "{}", event)
    }

    pub fn write_recovery(&mut self, shock_start_tick: u64, recovery_tick: u64, duration_ticks: u64, pre_shock_population: usize, min_population: usize) -> std::io::Result<()> {
        let event = serde_json::json!({
            "type": "recovery",
            "shock_start_tick": shock_start_tick,
            "recovery_tick": recovery_tick,
            "duration_ticks": duration_ticks,
            "pre_shock_population": pre_shock_population,
            "min_population": min_population,
        });
        writeln!(self.writer, "{}", event)
    }

    pub fn write_snapshot(&mut self, tick: u64, total_cells: usize, cost_dist: &HashMap<u32, usize>, mut_dist: &HashMap<u32, usize>, thresh_dist: &HashMap<u32, usize>, energy_dist: &HashMap<u32, usize>, gen_dist: &HashMap<u32, usize>, lineage_dist: &HashMap<String, usize>, producer_fraction: f64) -> std::io::Result<()> {
        let event = serde_json::json!({
            "type": "snapshot",
            "tick": tick,
            "total_cells": total_cells,
            "cost_distribution": cost_dist,
            "mut_distribution": mut_dist,
            "thresh_distribution": thresh_dist,
            "energy_distribution": energy_dist,
            "generation_distribution": gen_dist,
            "lineage_distribution": lineage_dist,
            "producer_fraction": producer_fraction,
        });
        writeln!(self.writer, "{}", event)
    }

    pub fn write_law(&mut self, tick: u64, law_id: &str, title: &str) -> std::io::Result<()> {
        let event = serde_json::json!({
            "type": "law",
            "tick": tick,
            "law_id": law_id,
            "title": title,
        });
        writeln!(self.writer, "{}", event)
    }

    pub fn flush(&mut self) -> std::io::Result<()> {
        self.writer.flush()
    }
}

#[derive(Debug, Clone, Default)]
pub struct GenerationTraitSnapshot {
    pub generation: u32,
    pub reproduction_cost: HashMap<u32, usize>,
    pub reproduction_threshold: HashMap<u32, usize>,
    pub initial_energy: HashMap<u32, usize>,
    pub mutation_rate: HashMap<u32, usize>,
    pub affinity_counts: HashMap<u8, usize>,
}

impl GenerationTraitSnapshot {
    pub fn new(generation: u32) -> Self {
        Self {
            generation,
            ..Default::default()
        }
    }
}

pub struct Runtime {
    pub tick: u64,
    pub genome: Genome,
    pub pool: CellPool,
    pub bus: SignalBus,
    pub affinity_map: HashMap<u8, Vec<u32>>,
    pub max_cells: usize,
    pub stats: RuntimeStats,
    pub observer: Observer,
    pub shuffle: bool,
    pub round_robin: bool,
    pub dynamic_affinity: bool,
    pub spawn_on_signal: bool,
    pub signal_adoptions: u64,
    pub trait_snapshots: Vec<GenerationTraitSnapshot>,
    pub death_rate: f64,
    /// Baseline (pre-shock) death rate, used to detect when a shock is active.
    pub base_death_rate: f64,
    pub event_writer: Option<EventWriter>,
    pub discovered_laws: std::collections::HashSet<String>,
    pub warning_emitted: bool,
    /// E037: when true, a cell whose gene has `recipient_policy: "kin"`
    /// emits its protective type-50 as DIRECTED signals to same-lineage
    /// cells only (not broadcast). This makes warning a selectively-shared
    /// good — the antidote to the L#062 free-rider tragedy.
    pub directed_warning: bool,
    /// E038: extra per-tick mortality applied ONLY to producer (is_memory)
    /// cells. Lets us gradualy suppress the warning network and find the
    /// ecosystem-collapse threshold (Law #064).
    pub producer_death_bonus: f64,
    /// When true, emit the per-cell debug `eprintln!` lines (heredity,
    /// reproduction, mortality). Default false — those lines dominate I/O and
    /// are the main speed bottleneck for long runs.
    pub verbose: bool,
    /// When true, skip per-signal event writes (the 1.36M-edge JSON spam).
    /// Keeps only snapshots/laws/births — cuts event files ~100x.
    pub summary_log: bool,
}

#[derive(Debug, Default)]
pub struct RuntimeStats {
    pub total_ticks: u64,
    pub total_signals_emitted: u64,
    pub total_signals_delivered: u64,
    pub total_cells_born: u32,
    pub total_cells_died: u32,
    pub total_energy_deaths: u32,
    pub total_energy_consumed: u64,
    pub total_energy_budget: u64,
    pub peak_cells: usize,
    pub total_adoptions: u64,
    pub total_spawns_from_signal: u32,
}

impl Runtime {
    pub fn new(genome: Genome, max_cells: usize) -> Self {
        Self {
            tick: 0,
            genome,
            pool: CellPool::new(),
            bus: SignalBus::new(),
            affinity_map: HashMap::new(),
            max_cells,
            stats: RuntimeStats::default(),
            observer: Observer::new(),
            shuffle: false,
            round_robin: false,
            dynamic_affinity: false,
            spawn_on_signal: false,
            signal_adoptions: 0,
            trait_snapshots: Vec::new(),
            death_rate: 0.0,
            base_death_rate: 0.0,
            event_writer: None,
            discovered_laws: std::collections::HashSet::new(),
            warning_emitted: false,
            directed_warning: false,
            producer_death_bonus: 0.0,
            verbose: false,
            summary_log: false,
        }
    }

    pub fn with_event_logging(mut self, path: &str) -> std::io::Result<Self> {
        self.event_writer = Some(EventWriter::new(path)?);
        Ok(self)
    }

    pub fn init(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let startup: Vec<_> = self.genome.startup_genes().cloned().collect();
        for gene in &startup {
            self.spawn_cell(&gene)?; // seed: 1 cell per startup gene
        }
        Ok(())
    }

    pub fn spawn_cell(&mut self, gene: &Gene) -> Result<u32, Box<dyn std::error::Error>> {
        if self.pool.len() >= self.max_cells {
            return Err("max cells reached".into());
        }
        // E035: seed lineage tag defaults to cell_type if not specified.
        let mut gene = gene.clone();
        if gene.lineage.is_empty() {
            gene.lineage = gene.cell_type.clone();
        }
        let id = self.pool.spawn(&gene, self.tick)?;
        if let Some(cell) = self.pool.get_mut(id) {
            cell.energy = cell.energy.saturating_sub(ENERGY_SPAWN_COST);
            self.stats.total_energy_budget += cell.energy as u64;
        }
        self.observer.register_cell(id, &gene.cell_type);
        for &type_id in &gene.signal_affinity {
            self.affinity_map.entry(type_id).or_default().push(id);
        }
        self.stats.total_cells_born += 1;
        self.stats.peak_cells = self.stats.peak_cells.max(self.pool.len());
        
        // Log birth event
        if let Some(ew) = &mut self.event_writer {
            let _ = ew.write_birth(
                self.tick, id, &gene.cell_type, None, gene.version,
                gene.reproduction_cost, gene.reproduction_threshold,
                gene.mutation_rate, gene.initial_energy,
                &gene.signal_affinity, gene.mutation_step,
                gene.decay_rate, gene.meta_mutation_rate
            );
        }
        
        Ok(id)
    }

    fn spawn_child(&mut self, parent_id: u32, parent_version: u32, child_gene: Gene) -> Result<u32, Box<dyn std::error::Error>> {
        if self.pool.len() >= self.max_cells {
            return Err("max cells reached".into());
        }
        let id = self.pool.spawn(&child_gene, self.tick)?;
        if let Some(cell) = self.pool.get_mut(id) {
            cell.energy = cell.energy.saturating_sub(ENERGY_SPAWN_COST);
            self.stats.total_energy_budget += cell.energy as u64;
        }
        self.observer.register_cell(id, &child_gene.cell_type);
        for &type_id in &child_gene.signal_affinity {
            self.affinity_map.entry(type_id).or_default().push(id);
        }
        self.stats.total_cells_born += 1;
        self.stats.peak_cells = self.stats.peak_cells.max(self.pool.len());
        dlog!(self, "[tick {}] heredity: parent {} (v{}) -> child {} (v{}, aff={:?}, thresh={}, cost={}, energy={})",
            self.tick, parent_id, parent_version, id, child_gene.version,
            child_gene.signal_affinity, child_gene.reproduction_threshold, child_gene.reproduction_cost, child_gene.initial_energy);
        Ok(id)
    }

    pub fn step(&mut self) {
        self.tick += 1;
        self.stats.total_ticks += 1;

        if self.shuffle {
            self.pool.shuffle();
        }
        if self.round_robin {
            self.pool.reverse();
        }

        self.distribute_signals();

        // E037: precompute, per lineage, the alive same-lineage recipients
        // so a "kin" memory cell can direct its warning instead of
        // broadcasting it. Done once per tick (outside the per-cell loop)
        // to avoid borrowing self.pool while it is mutably iterated.
        let kin_recipients: HashMap<String, Vec<u32>> = if self.directed_warning {
            let mut map: HashMap<String, Vec<u32>> = HashMap::new();
            for c in self.pool.iter() {
                if c.alive {
                    map.entry(c.gene.lineage.clone()).or_default().push(c.id);
                }
            }
            map
        } else {
            HashMap::new()
        };

        let mut energy_dead_this_tick = 0u32;
        let mut mortality_dead_this_tick = 0u32;
        let mut predictive_emitted_this_tick = false;
        let mut kin_warn_this_tick = false;
        use rand::Rng;
        let mut rng = rand::thread_rng();
        for cell in self.pool.iter_mut() {
            if !cell.alive {
                continue;
            }
            // Mortality: random death chance per tick.
            // A "prepared" cell (braced via predictive signal) gets a discount
            // while a shock is active (elevated death_rate).
            let eff_death_rate = if cell.prepared_ticks > 0 && self.death_rate > self.base_death_rate {
                self.death_rate * PREDICTIVE_DISCOUNT
            } else {
                self.death_rate
            };
            // E038: producers (is_memory) carry an extra mortality load,
            // simulating gradualy rising free-rider pressure. This lets us
            // drive the producer fraction down and locate the collapse point.
            let eff_death_rate = if cell.gene.is_memory && self.producer_death_bonus > 0.0 {
                (eff_death_rate + self.producer_death_bonus).min(1.0)
            } else {
                eff_death_rate
            };
            if self.death_rate > 0.0 && rng.gen::<f64>() < eff_death_rate {
                cell.alive = false;
                mortality_dead_this_tick += 1;
                dlog!(self, "[tick {}] cell {} died from mortality{}", self.tick, cell.id,
                    if cell.prepared_ticks > 0 { " (prepared, discounted)" } else { "" });
                
                // Log death event
                if let Some(ew) = &mut self.event_writer {
                    let age = self.tick.saturating_sub(cell.birth_tick);
                    let _ = ew.write_death(
                        self.tick, cell.id, &cell.gene.cell_type, "mortality",
                        age, cell.gene.version
                    );
                }
                continue;
            }
            let before_alive = cell.alive;
            let cell_id = cell.id;
            let result = cell.tick();
            match result {
                CellTickResult::Ok => {
                    if !cell.alive && before_alive && cell.energy == 0 {
                        energy_dead_this_tick += 1;
                        
                        // Log energy death event
                        if let Some(ew) = &mut self.event_writer {
                            let age = self.tick.saturating_sub(cell.birth_tick);
                            let _ = ew.write_death(
                                self.tick, cell.id, &cell.gene.cell_type, "energy",
                                age, cell.gene.version
                            );
                        }
                    }
                    let emitted = self.bus.collect_output(cell.id, &mut cell.output_fifo);
                    // E037: selective (directed) warning. If directed_warning is on
                    // and this gene's policy is "kin", rewrite its emitted type-50
                    // broadcast into DIRECTED copies addressed only to alive
                    // same-lineage cells. The emitter still braces itself.
                    let kin_ids: Vec<u32> = if self.directed_warning
                        && cell.gene.recipient_policy == "kin"
                        && emitted.iter().any(|s| s.type_id == 50)
                    {
                        kin_recipients.get(&cell.gene.lineage)
                            .map(|v| v.iter().copied().filter(|&id| id != cell.id).collect())
                            .unwrap_or_default()
                    } else {
                        Vec::new()
                    };
                    let emitted = if kin_ids.is_empty() {
                        emitted
                    } else {
                        let mut out = Vec::new();
                        let mut saw_kin_warn = false;
                        for sig in emitted {
                            if sig.type_id == 50 {
                                predictive_emitted_this_tick = true;
                                saw_kin_warn = true;
                                for &kid in &kin_ids {
                                    let mut d = sig;
                                    d.target_id = kid;
                                    self.bus.emit(d);
                                    out.push(d);
                                }
                            } else {
                                self.bus.emit(sig);
                                out.push(sig);
                            }
                        }
                        if saw_kin_warn {
                            kin_warn_this_tick = true;
                        }
                        out
                    };
                    self.stats.total_signals_emitted += emitted.len() as u64;
                    for sig in &emitted {
                        self.observer.observe_emission(cell_id, sig.type_id);
                        if sig.type_id == 50 {
                            predictive_emitted_this_tick = true;
                            // The cell braced: stay prepared for a few ticks so
                            // it survives an imminent shock with a mortality discount.
                            cell.prepared_ticks = PREPARED_DURATION;
                        }
                    }
                }
                CellTickResult::Crashed(reason) => {
                    dlog!(self, "[tick {}] cell {} crashed: {}", self.tick, cell.id, reason);
                    cell.alive = false;
                    
                    // Log crash death event
                    if let Some(ew) = &mut self.event_writer {
                        let age = self.tick.saturating_sub(cell.birth_tick);
                        let _ = ew.write_death(
                            self.tick, cell.id, &cell.gene.cell_type, "crash",
                            age, cell.gene.version
                        );
                    }
                }
                CellTickResult::Timeout => {
                    dlog!(self, "[tick {}] cell {} timed out", self.tick, cell.id);
                    cell.alive = false;
                    
                    // Log timeout death event
                    if let Some(ew) = &mut self.event_writer {
                        let age = self.tick.saturating_sub(cell.birth_tick);
                        let _ = ew.write_death(
                            self.tick, cell.id, &cell.gene.cell_type, "timeout",
                            age, cell.gene.version
                        );
                    }
                }
            }
        }
        self.stats.total_energy_deaths += energy_dead_this_tick;
        self.stats.total_cells_died += mortality_dead_this_tick;
        if mortality_dead_this_tick > 0 || energy_dead_this_tick > 0 {
            self.discover_law("L028", "Mortality Unfreezes Selection");
        }
        if predictive_emitted_this_tick && self.warning_emitted {
            self.discover_law("L056", "Predictive Information Has Fitness Value");
        }
        if kin_warn_this_tick {
            self.discover_law("L063", "Information Becomes More Valuable When Selectively Shared");
        }

        // Age out the "prepared" state for surviving cells.
        for cell in self.pool.iter_mut() {
            if cell.prepared_ticks > 0 {
                cell.prepared_ticks -= 1;
            }
        }

        let died = self.pool.sweep();
        self.stats.total_cells_died += died as u32;
        
        // Log sweep deaths (energy depletion after sweep)
        if let Some(ew) = &mut self.event_writer {
            for cell in self.pool.iter() {
                if !cell.alive && cell.death_tick == Some(self.tick) {
                    let age = self.tick.saturating_sub(cell.birth_tick);
                    let _ = ew.write_death(
                        self.tick, cell.id, &cell.gene.cell_type, "energy",
                        age, cell.gene.version
                    );
                }
            }
        }

        self.trigger_earned_reproduction();

        if self.dynamic_affinity {
            self.adopt_orphan_signals();
        }

        if self.spawn_on_signal {
            self.trigger_spawn_on_signal();
        }

        self.record_trait_snapshot();

        // Write tick event for ZMON
        if let Some(ref mut ew) = self.event_writer {
            let total_energy: u64 = self.pool.iter().map(|c| c.energy as u64).sum();
            let _ = ew.write_tick(
                self.tick,
                self.pool.len(),
                total_energy,
                self.stats.total_signals_emitted,
                self.stats.total_signals_delivered,
            );
        }
    }

    fn distribute_signals(&mut self) {
        let cell_info: Vec<(u32, Vec<u8>)> = self.pool.iter()
            .filter(|c| c.alive)
            .map(|c| (c.id, c.gene.signal_affinity.clone()))
            .collect();

        let signals: Vec<Signal> = self.bus.drain();
        let mut dropped: u64 = 0;

        for sig in &signals {
            let src_id = sig.source_id;
            let sig_type = sig.type_id;
            let src_type = self.observer.cell_types.get(&src_id)
                .cloned().unwrap_or_default();
            if sig.is_broadcast() {
                for &(cell_id, ref affinity) in &cell_info {
                    if affinity.contains(&sig_type) {
                        if let Some(cell) = self.pool.get_mut(cell_id) {
                            if cell.input_fifo.push(sig) {
                                 self.stats.total_signals_delivered += 1;
                                self.observer.observe_delivery(cell_id, sig_type,
                                                               src_id, &src_type);
                                let tgt_type = cell.gene.cell_type.clone();
                                if sig_type == 50 {
                                    // E036: a cell that RECEIVES a protective warning
                                    // (type-50) braces too — this is the crux of
                                    // communication. One memory cell's prediction
                                    // now protects *others*, not just itself.
                                    cell.prepared_ticks = PREPARED_DURATION;
                                }
                            let saw_warn = sig_type == 50 && self.warning_emitted;
                            if sig_type == 50 && self.warning_emitted && src_type != tgt_type {
                                self.discover_law("L060", "Communication Converts Individual Information Into Shared Information");
                            }
                                // E036: communication law — a protective warning
                                // reaching a cell of a DIFFERENT lineage than its
                                // emitter is the birth of shared information.
                                if sig_type == 50 && self.warning_emitted && src_type != tgt_type {
                                    self.discover_law("L060", "Communication Converts Individual Information Into Shared Information");
                                }
                                if let Some(ew) = &mut self.event_writer {
                                    if !self.summary_log {
                                        let _ = ew.write_signal(
                                            self.tick, self.stats.total_signals_delivered,
                                            sig_type, src_id, &src_type,
                                            cell_id, &tgt_type, true,
                                        );
                                    }
                                }
                                if saw_warn {
                                    // Deferred: discover_law needs &mut self, which
                                    // conflicts with the live `cell` borrow above.
                                    self.discover_law("L056", "Predictive Information Has Fitness Value");
                                }
                            } else {
                                dropped += 1;
                            }
                        }
                    }
                }
            } else {
                let tgt = sig.target_id;
                if let Some(cell) = self.pool.get_mut(tgt) {
                    if cell.alive {
                        if cell.input_fifo.push(sig) {
                            self.stats.total_signals_delivered += 1;
                            self.observer.observe_delivery(tgt, sig_type,
                                                           src_id, &src_type);
                            let tgt_type = cell.gene.cell_type.clone();
                            if sig_type == 50 {
                                // E036: recipient of a protective warning braces.
                                cell.prepared_ticks = PREPARED_DURATION;
                            }
                            let saw_warn = sig_type == 50 && self.warning_emitted;
                            if let Some(ew) = &mut self.event_writer {
                                if !self.summary_log {
                                    let _ = ew.write_signal(
                                        self.tick, self.stats.total_signals_delivered,
                                        sig_type, src_id, &src_type,
                                        tgt, &tgt_type, true,
                                    );
                                }
                            }
                            if saw_warn {
                                self.discover_law("L056", "Predictive Information Has Fitness Value");
                            }
                        } else {
                            dropped += 1;
                        }
                    }
                }
            }
        }
    }

    pub fn kill_by_type(&mut self, cell_type: &str) -> usize {
        let ids: Vec<u32> = self.pool.iter()
            .filter(|c| c.alive && c.gene.cell_type == cell_type)
            .map(|c| c.id)
            .collect();
        let count = ids.len();
        for id in &ids {
            self.pool.kill(*id);
            self.observer.cell_killed(*id);
            if self.dynamic_affinity {
                if let Some(cell) = self.pool.iter().find(|c| c.id == *id) {
                    for &t in &cell.gene.signal_affinity {
                        if let Some(ids) = self.affinity_map.get_mut(&t) {
                            ids.retain(|&x| x != *id);
                        }
                    }
                }
            }
        }
        count
    }

    fn adopt_orphan_signals(&mut self) {
        let alive_types: HashSet<u8> = self.pool.iter()
            .filter(|c| c.alive)
            .flat_map(|c| c.gene.signal_affinity.iter().copied())
            .collect();

        let live_signals: Vec<Signal> = self.bus.peek_all();
        let mut orphan_types: HashSet<u8> = HashSet::new();
        for sig in &live_signals {
            if !alive_types.contains(&sig.type_id) {
                orphan_types.insert(sig.type_id);
            }
        }

        let remaining: Vec<u32> = self.pool.iter()
            .filter(|c| c.alive)
            .map(|c| c.id)
            .collect();

        for &t in &orphan_types {
            if remaining.is_empty() {
                break;
            }
            let adopted = self.pool.adopt_orphan_affinity(t);
            if adopted > 0 {
                self.stats.total_adoptions += adopted as u64;
                self.signal_adoptions += 1;
                dlog!(self, "[tick {}] orphan type {} adopted by a cell", self.tick, t);
            }
        }
    }

    fn trigger_spawn_on_signal(&mut self) {
        let signals: Vec<Signal> = self.bus.peek_all();
        let active_types: HashSet<u8> = signals.iter().map(|s| s.type_id).collect();

        let to_spawn: Vec<Gene> = self.genome.genes.iter()
            .filter(|g| {
                if let Some(ref sig_name) = g.spawn_condition.on_signal {
                    if let Ok(t) = sig_name.parse::<u8>() {
                        return active_types.contains(&t);
                    }
                    false
                } else {
                    false
                }
            })
            .cloned()
            .collect();

        use rand::Rng;
        let mut rng = rand::thread_rng();

        for gene in &to_spawn {
            if self.pool.len() >= self.max_cells {
                break;
            }
            let existing = self.pool.iter().filter(|c| c.gene.cell_type == gene.cell_type).count() as u32;
            if existing >= gene.max_instances {
                continue;
            }
            let parent_ids: Vec<u32> = self.pool.iter()
                .filter(|c| c.gene.cell_type == gene.cell_type)
                .filter(|c| c.alive)
                .map(|c| c.id)
                .collect();
            if let Some(&parent_id) = parent_ids.first() {
                if let Some(parent) = self.pool.get_mut(parent_id) {
                    parent.energy = parent.energy.saturating_sub(ENERGY_SPAWN_COST);
                    let parent_version = parent.gene.version;
                    let parent_gene = parent.gene.clone();
                    let parent_type = parent_gene.cell_type.clone();
                    dlog!(self, "[tick {}] parent cell {} paid {} for spawn", self.tick, parent_id, ENERGY_SPAWN_COST);

                    let mut child_gene = parent_gene;
                    child_gene.version += 1;
                    // Use heritable decay_rate from parent
                    child_gene.mutation_rate = (child_gene.mutation_rate * child_gene.decay_rate).max(0.001);

                    // Use heritable mutation_step for mutations
                    let mutation_step = child_gene.mutation_step;

                    // TINY mutations only
                    if rng.gen::<f64>() < child_gene.mutation_rate {
                        if !child_gene.signal_affinity.is_empty() {
                            if rng.gen::<bool>() {
                                // Add ONE type near existing
                                let base_type = child_gene.signal_affinity[rng.gen_range(0..child_gene.signal_affinity.len())];
                                let new_type = rng.gen_range(base_type.saturating_sub(5)..=base_type.saturating_add(5)).clamp(1, 255);
                                if !child_gene.signal_affinity.contains(&new_type) {
                                    child_gene.signal_affinity.push(new_type);
                                    dlog!(self, "[tick {}] mutation: signal-spawn child affinity +{} -> {:?}", self.tick, new_type, child_gene.signal_affinity);
                                }
                            } else if child_gene.signal_affinity.len() > 1 {
                                // Remove ONE type
                                let idx = rng.gen_range(0..child_gene.signal_affinity.len());
                                let removed = child_gene.signal_affinity.remove(idx);
                                dlog!(self, "[tick {}] mutation: signal-spawn child affinity -{} -> {:?}", self.tick, removed, child_gene.signal_affinity);
                            }
                        }
                    }
                    if rng.gen::<f64>() < child_gene.mutation_rate {
                        // Use heritable mutation_step for mutations
                        let change = rng.gen_range(-(mutation_step as i32)..=(mutation_step as i32));
                        child_gene.reproduction_threshold = child_gene.reproduction_threshold.saturating_add_signed(change).clamp(50, 300);
                        dlog!(self, "[tick {}] mutation: signal-spawn child threshold {:+} -> {}", self.tick, change, child_gene.reproduction_threshold);
                    }
                    if rng.gen::<f64>() < child_gene.mutation_rate {
                        let change = rng.gen_range(-(mutation_step as i32)..=(mutation_step as i32));
                        child_gene.reproduction_cost = child_gene.reproduction_cost.saturating_add_signed(change).max(1);
                        dlog!(self, "[tick {}] mutation: signal-spawn child cost {:+} -> {}", self.tick, change, child_gene.reproduction_cost);
                    }
                    if rng.gen::<f64>() < child_gene.mutation_rate {
                        let change = rng.gen_range(-(mutation_step as i32)..=(mutation_step as i32));
                        child_gene.initial_energy = child_gene.initial_energy.saturating_add_signed(change).max(1);
                        dlog!(self, "[tick {}] mutation: signal-spawn child initial_energy {:+} -> {}", self.tick, change, child_gene.initial_energy);
                    }

                    // Use heritable decay_rate
                    child_gene.mutation_rate = (child_gene.mutation_rate * child_gene.decay_rate).max(0.001);

                    // E035: child may switch lineage (reactive <-> memory).
                    let switched = self.apply_lineage_switch(&mut child_gene);
                    if switched {
                        dlog!(self, "[tick {}] LINEAGE SWITCH: child {} -> {} (from {})",
                            self.tick, parent_id, child_gene.cell_type, parent_type);
                    }

                    if let Ok(id) = self.spawn_child(parent_id, parent_version, child_gene) {
                        self.stats.total_spawns_from_signal += 1;
                        dlog!(self, "[tick {}] spawned cell {} (type '{}') from signal trigger ({}/{})",
                            self.tick, id, gene.cell_type, existing + 1, gene.max_instances);
                    }
                }
            }
        }
    }

    /// E035: give a freshly-built child gene a chance to switch lineage
    /// (cell_type + asm_path + affinity + is_memory) into its parent's
    /// declared sibling. Returns true if a switch happened. The child still
    /// carries the parent's `sibling` field at this point, so we read it
    /// from `child_gene` itself.
    fn apply_lineage_switch(&self, child_gene: &mut Gene) -> bool {
        let sib_name = match &child_gene.sibling {
            Some(s) => s.clone(),
            None => return false,
        };
        if let Some(sib) = self.genome.get_gene(&sib_name) {
            return Gene::maybe_switch_lineage(child_gene, Some(sib), &mut rand::thread_rng());
        }
        false
    }

    fn trigger_earned_reproduction(&mut self) {
        let mut candidates: Vec<(u32, Gene)> = self.pool.iter()
            .filter(|c| c.alive)
            .filter(|c| c.gene.reproduction_threshold > 0)
            .filter(|c| c.energy >= c.gene.reproduction_threshold)
            .map(|c| (c.id, c.gene.clone()))
            .collect();

        use rand::Rng;
        let mut rng = rand::thread_rng();

        for (cell_id, parent_gene) in candidates {
            if self.pool.len() >= self.max_cells {
                break;
            }
            let existing = self.pool.iter()
                .filter(|c| c.gene.cell_type == parent_gene.cell_type)
                .count() as u32;
            if existing >= parent_gene.max_instances {
                continue;
            }
            let energy_before;
            if let Some(cell) = self.pool.get_mut(cell_id) {
                energy_before = cell.energy;
                cell.energy = cell.energy.saturating_sub(parent_gene.reproduction_cost);
            } else {
                continue;
            }

let mut child_gene = parent_gene.clone();
            child_gene.version += 1;
            // Use heritable decay_rate from parent
            child_gene.mutation_rate = (child_gene.mutation_rate * child_gene.decay_rate).max(0.001);

            // Use heritable mutation_step for mutations
            let mutation_step = child_gene.mutation_step;

            if rng.gen::<f64>() < child_gene.mutation_rate {
                // Add or remove 1-3 affinity types
                if rng.gen::<bool>() && !child_gene.signal_affinity.is_empty() {
                    // Remove random
                    let idx = rng.gen_range(0..child_gene.signal_affinity.len());
                    child_gene.signal_affinity.remove(idx);
                } else {
                    // Add random 1-3 new
                    let mut_aff = rng.gen_range(1..=3);
                    for _ in 0..mut_aff {
                        let new_type = rng.gen_range(1..=255);
                        if !child_gene.signal_affinity.contains(&new_type) {
                            child_gene.signal_affinity.push(new_type);
                        }
                    }
                }
                dlog!(self, "[tick {}] mutation: child {} affinity changed to {:?}", self.tick, cell_id, child_gene.signal_affinity);
            }
            if rng.gen::<f64>() < child_gene.mutation_rate {
                // Use heritable mutation_step for mutations
                let change = rng.gen_range(-(mutation_step as i32)..=(mutation_step as i32));
                child_gene.reproduction_threshold = child_gene.reproduction_threshold.saturating_add_signed(change).clamp(50, 300);
                dlog!(self, "[tick {}] mutation: child {} threshold changed to {}", self.tick, cell_id, child_gene.reproduction_threshold);
            }
            if rng.gen::<f64>() < child_gene.mutation_rate {
                let change = rng.gen_range(-(mutation_step as i32)..=(mutation_step as i32));
                child_gene.reproduction_cost = child_gene.reproduction_cost.saturating_add_signed(change).max(1);
                dlog!(self, "[tick {}] mutation: child {} cost changed to {}", self.tick, cell_id, child_gene.reproduction_cost);
            }
            if rng.gen::<f64>() < child_gene.mutation_rate {
                let change = rng.gen_range(-(mutation_step as i32)..=(mutation_step as i32));
                child_gene.initial_energy = child_gene.initial_energy.saturating_add_signed(change).max(1);
                dlog!(self, "[tick {}] mutation: child {} initial_energy changed to {}", self.tick, cell_id, child_gene.initial_energy);
            }

            // Use heritable decay_rate
            child_gene.mutation_rate = (child_gene.mutation_rate * child_gene.decay_rate).max(0.001);

            // E035: child may switch lineage (reactive <-> memory).
            let switched = self.apply_lineage_switch(&mut child_gene);
            if switched {
                dlog!(self, "[tick {}] LINEAGE SWITCH: child {} -> {} (from {})",
                    self.tick, cell_id, child_gene.cell_type, parent_gene.cell_type);
            }

            let parent_version = parent_gene.version;
            if let Ok(id) = self.spawn_child(cell_id, parent_version, child_gene.clone()) {
                dlog!(self, "[tick {}] cell {} reproduced -> {} (energy {} -> {}, cost {})",
                    self.tick, cell_id, id, energy_before, energy_before.saturating_sub(parent_gene.reproduction_cost), parent_gene.reproduction_cost);
                self.discover_law("L020", "Heredity Enables Evolution");
                if let Some(ew) = &mut self.event_writer {
                    let _ = ew.write_reproduction(
                        self.tick, cell_id, &parent_gene.cell_type, id, &child_gene.cell_type,
                        parent_version, child_gene.version, child_gene.reproduction_cost,
                        child_gene.reproduction_threshold, child_gene.mutation_rate, &[],
                    );
                }
            }
        }

        // Record trait snapshot for this generation
        self.record_trait_snapshot();
    }

    fn record_trait_snapshot(&mut self) {
        use std::collections::HashMap;
        let mut snapshot = GenerationTraitSnapshot::new(0);

        // Determine max generation (version) among living cells
        let max_gen = self.pool.iter()
            .filter(|c| c.alive)
            .map(|c| c.gene.version)
            .max()
            .unwrap_or(1);
        snapshot.generation = max_gen;

        let mut gen_dist: HashMap<u32, usize> = HashMap::new();
        let mut lineage_dist: HashMap<String, usize> = HashMap::new();
        for cell in self.pool.iter().filter(|c| c.alive) {
            let g = &cell.gene;
            *snapshot.reproduction_cost.entry(g.reproduction_cost).or_insert(0) += 1;
            *snapshot.reproduction_threshold.entry(g.reproduction_threshold).or_insert(0) += 1;
            *snapshot.initial_energy.entry(g.initial_energy).or_insert(0) += 1;
            let mr_scaled = (g.mutation_rate * 10000.0) as u32;
            *snapshot.mutation_rate.entry(mr_scaled).or_insert(0) += 1;
            for &aff in &g.signal_affinity {
                *snapshot.affinity_counts.entry(aff).or_insert(0) += 1;
            }
            *gen_dist.entry(g.version).or_insert(0) += 1;
            // E035: use the immutable lineage tag so descent can be tracked
            // through reactive <-> memory switches (cell_type changes on switch).
            let lineage = if g.lineage.is_empty() { g.cell_type.clone() } else { g.lineage.clone() };
            *lineage_dist.entry(lineage).or_insert(0) += 1;
        }

        // L#005: pool order creates bias — only meaningful with >1 distinct type competing
        let distinct_types: std::collections::HashSet<_> = self.pool.iter()
            .filter(|c| c.alive)
            .map(|c| c.gene.cell_type.clone())
            .collect();
        if distinct_types.len() > 1 {
            self.discover_law("L005", "Pool Order Creates Bias");
        }

        if let Some(ew) = &mut self.event_writer {
            let alive: Vec<&Cell> = self.pool.iter().filter(|c| c.alive).collect();
            let total = alive.len();
            let producers = alive.iter().filter(|c| c.gene.is_memory).count();
            let producer_fraction = if total > 0 { producers as f64 / total as f64 } else { 0.0 };
            let _ = ew.write_snapshot(
                self.tick,
                total,
                &snapshot.reproduction_cost,
                &snapshot.mutation_rate,
                &snapshot.reproduction_threshold,
                &snapshot.initial_energy,
                &gen_dist,
                &lineage_dist,
                producer_fraction,
            );
        }

        self.trait_snapshots.push(snapshot);
    }

    /// Emit a "law discovered" event at most once per law id.
    pub fn discover_law(&mut self, law_id: &str, title: &str) {
        if self.discovered_laws.contains(law_id) {
            return;
        }
        self.discovered_laws.insert(law_id.to_string());
        if let Some(ew) = &mut self.event_writer {
            let _ = ew.write_law(self.tick, law_id, title);
        }
    }

    pub fn inject_signal(&mut self, signal: Signal) {
        self.bus.emit(signal);
    }

    pub fn running_cells(&self) -> usize {
        self.pool.len()
    }
}
