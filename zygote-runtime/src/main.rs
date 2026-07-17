mod types;
mod signal;
mod genome;
mod cell;
mod runtime;
mod sandbox;
mod observer;

use clap::Parser;
use genome::Genome;
use rand::Rng;
use runtime::Runtime;
use types::*;
use std::time::Duration;

#[derive(Debug)]
struct ParamChange {
    param: String,
    value: String,
    tick: u64,
}

struct ShockState {
    active: bool,
    remaining_ticks: u64,
    base_death_rate: f64,
}

#[derive(Parser)]
#[command(name = "zygote-runtime", about = "Rust foster parent for ZygoteAI Assembly cells")]
struct Cli {
    #[arg(short, long, default_value = "genomes/default.json")]
    genome: String,

    #[arg(short, long, default_value_t = 100)]
    ticks: u64,

    #[arg(long, default_value_t = 1000)]
    max_cells: usize,

    #[arg(long)]
    inject: Option<String>,

    #[arg(long, help = "Re-inject the signal every N ticks (requires --inject)")]
    inject_interval: Option<u64>,

    #[arg(long, help = "Disable a cell type mid-run. Format: cell_type@tick")]
    disable: Vec<String>,

    #[arg(long, help = "Shuffle cell pool order each tick (randomizes delivery order)")]
    shuffle: bool,

    #[arg(long, help = "Alternate pool iteration direction each tick (true round-robin, A/B/A/B alternation)")]
    round_robin: bool,

    #[arg(long, help = "Adopt orphan signal types to remaining cells when their listener dies")]
    dynamic_affinity: bool,

    #[arg(long, help = "Spawn cells when their trigger signal type appears in the bus")]
    spawn_on_signal: bool,

    #[arg(long, help = "Random death probability per cell per tick (e.g., 0.001 = 0.1%)")]
    death_rate: f64,

    #[arg(long, help = "Probability per tick of a stress event (e.g., 0.01 = 1%)")]
    shock_prob: Option<f64>,

    #[arg(long, help = "Death rate during stress event (default: 0.01)")]
    shock_death_rate: Option<f64>,

    #[arg(long, help = "Duration of stress event in ticks (default: 100)")]
    shock_duration: Option<u64>,

    #[arg(long, help = "Path to write event log (JSONL) for ZMON")]
    events_log: Option<String>,

    #[arg(long, help = "Change a parameter mid-run. Format: param=value@tick (params: death_rate, inject_interval)")]
    change: Vec<String>,

    #[arg(long, help = "Emit a broadcast warning signal at a tick. Format: type@tick (e.g. 9@100)")]
    warn_signal: Vec<String>,

    #[arg(long, help = "If set, each warning triggers a real shock with this probability (e.g. 0.9). Otherwise a false alarm. Shock fires warn_shock_delay ticks later.")]
    warn_shock_prob: Option<f64>,

    #[arg(long, help = "Ticks between a warning and its (possible) shock (default: 3)")]
    warn_shock_delay: Option<u64>,
    #[arg(long, help = "E037: cells with recipient_policy 'kin' emit protective type-50 as DIRECTED signals to same-lineage cells only (selective sharing).")]
    directed_warning: bool,

    #[arg(long, help = "E038: extra per-tick mortality applied ONLY to producer (is_memory) cells. Lets us gradualy suppress the warning network and find the ecosystem-collapse threshold. Range 0.0..1.0")]
    producer_death_bonus: Option<f64>,

    #[arg(long, help = "Enable per-cell debug eprintln! output (heredity, reproduction, mortality). Off by default — those lines dominate I/O on long runs.")]
    verbose: bool,

    #[arg(long, help = "Skip per-signal event writes; keep only snapshots/laws/births. Cuts event files ~100x. Use with extract scripts that read snapshots.")]
    summary_log: bool,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    let genome = Genome::from_file(&cli.genome)?;
    eprintln!("Genome '{}' loaded (v{}, {} genes)", genome.name, genome.version, genome.genes.len());

    let mut rt = Runtime::new(genome, cli.max_cells);
    if let Some(path) = &cli.events_log {
        rt = rt.with_event_logging(path)?;
    }
    rt.shuffle = cli.shuffle;
    rt.round_robin = cli.round_robin;
    rt.dynamic_affinity = cli.dynamic_affinity;
    rt.spawn_on_signal = cli.spawn_on_signal;
    rt.directed_warning = cli.directed_warning;
    rt.producer_death_bonus = cli.producer_death_bonus.unwrap_or(0.0);
    rt.verbose = cli.verbose;
    rt.summary_log = cli.summary_log;
    rt.death_rate = cli.death_rate;
    rt.base_death_rate = cli.death_rate;
    rt.init()?;
    eprintln!("Runtime initialized with {} cells", rt.running_cells());

    let mut disable_schedule: Vec<(String, u64)> = Vec::new();
    for entry in &cli.disable {
        let parts: Vec<&str> = entry.split('@').collect();
        if parts.len() == 2 {
            let cell_type = parts[0].to_string();
            if let Ok(tick) = parts[1].parse::<u64>() {
                disable_schedule.push((cell_type, tick));
            }
        }
    }

    #[derive(Debug)]
    struct ParamChange {
        param: String,
        value: String,
        tick: u64,
    }
    
    let mut change_schedule: Vec<ParamChange> = Vec::new();
    for entry in &cli.change {
        // Format: param=value@tick
        let parts: Vec<&str> = entry.split('@').collect();
        if parts.len() == 2 {
            let param_value = parts[0];
            if let Ok(tick) = parts[1].parse::<u64>() {
                let pv_parts: Vec<&str> = param_value.split('=').collect();
                if pv_parts.len() == 2 {
                    change_schedule.push(ParamChange {
                        param: pv_parts[0].to_string(),
                        value: pv_parts[1].to_string(),
                        tick,
                    });
                }
            }
        }
    }

    // Warning signal schedule: type@tick -> broadcast signal injected at tick
    let mut warn_schedule: Vec<(u8, u64)> = Vec::new();
    for entry in &cli.warn_signal {
        let parts: Vec<&str> = entry.split('@').collect();
        if parts.len() == 2 {
            if let (Ok(ty), Ok(tk)) = (parts[0].parse::<u8>(), parts[1].parse::<u64>()) {
                warn_schedule.push((ty, tk));
            }
        }
    }
    change_schedule.sort_by_key(|c| c.tick);

    let mut external_signals = cli.inject.clone();
    let inject_interval = cli.inject_interval.unwrap_or(0);
    
    // Shock state
    struct ShockState {
        active: bool,
        remaining_ticks: u64,
        base_death_rate: f64,
    }

    #[derive(Clone)]
    struct ShockRecovery {
        shock_start_tick: u64,
        pre_shock_population: usize,
        recovery_tick: Option<u64>,
        min_population: usize,
    }

    let shock_prob = cli.shock_prob.unwrap_or(0.0);
    let shock_death_rate = cli.shock_death_rate.unwrap_or(0.01);
    let shock_duration = cli.shock_duration.unwrap_or(100);
    let warn_shock_prob = cli.warn_shock_prob.unwrap_or(0.0);
    let warn_shock_delay = cli.warn_shock_delay.unwrap_or(3);
    let mut shock_state = ShockState {
        active: false,
        remaining_ticks: 0,
        base_death_rate: cli.death_rate,
    };
    let mut rng = rand::thread_rng();

    // Scheduled shocks originating from warnings: (fire_tick, death_rate, duration)
    let mut pending_shocks: Vec<(u64, f64, u64)> = Vec::new();

    // Recovery tracking
    let mut shock_recoveries: Vec<ShockRecovery> = Vec::new();
    let mut current_recovery: Option<ShockRecovery> = None;

    for t in 0..cli.ticks {
        if let Some(ref cmd) = external_signals {
            if let Some(sig) = parse_signal(cmd) {
                rt.inject_signal(sig);
            }
            if inject_interval > 0 {
                // Keep the signal for periodic re-injection
            } else {
                external_signals = None;
            }
        } else if inject_interval > 0 && t > 0 && t % inject_interval == 0 {
            // Re-inject the original signal
            if let Some(ref cmd) = cli.inject {
                if let Some(sig) = parse_signal(cmd) {
                    rt.inject_signal(sig);
                }
            }
        }

        for (cell_type, kill_tick) in &disable_schedule {
            if *kill_tick == t {
                let killed = rt.kill_by_type(cell_type);
                eprintln!("[tick {}] Killed {} cells of type '{}'", t, killed, cell_type);
            }
        }

        // Emit scheduled warning broadcast signals; possibly pair with a shock.
        for (ty, tk) in &warn_schedule {
            if *tk == t {
                rt.inject_signal(Signal::new(*ty, 0xFFFF, 0xFFFFFFFF));
                rt.warning_emitted = true;
                eprintln!("[tick {}] WARNING signal type {} broadcast", t, ty);
                if warn_shock_prob > 0.0 {
                    if rng.gen::<f64>() < warn_shock_prob {
                        pending_shocks.push((t + warn_shock_delay, shock_death_rate, shock_duration));
                    } else {
                        eprintln!("[tick {}] FALSE ALARM (no shock follows warning)", t);
                    }
                }
            }
        }

        // Fire shocks scheduled from warnings (skip if a shock is already active).
        if !shock_state.active {
            let mut idx = 0;
            while idx < pending_shocks.len() {
                if pending_shocks[idx].0 == t {
                    let (_, dr, dur) = pending_shocks.remove(idx);
                    shock_state.active = true;
                    shock_state.remaining_ticks = dur;
                    shock_state.base_death_rate = rt.death_rate;
                    let pre_shock_population = rt.running_cells();
                    rt.death_rate = dr;
                    eprintln!("[tick {}] SHOCK! death_rate -> {} for {} ticks (pre-shock pop: {})", t, dr, dur, pre_shock_population);
                    if let Some(ew) = &mut rt.event_writer {
                        let _ = ew.write_shock(t, dr, dur, pre_shock_population);
                    }
                    rt.discover_law("L049", "Environmental Variation Maintains Diversity");
                    current_recovery = Some(ShockRecovery {
                        shock_start_tick: t,
                        pre_shock_population,
                        recovery_tick: None,
                        min_population: pre_shock_population,
                    });
                } else {
                    idx += 1;
                }
            }
        }

        // Process parameter changes
        while let Some(change) = change_schedule.first() {
            if change.tick == t {
                let change = change_schedule.remove(0);
                match change.param.as_str() {
                    "death_rate" => {
                        if let Ok(val) = change.value.parse::<f64>() {
                            rt.death_rate = val;
                            eprintln!("[tick {}] Changed death_rate to {}", t, val);
                        }
                    }
                    "inject_interval" => {
                        if let Ok(val) = change.value.parse::<u64>() {
                            eprintln!("[tick {}] Changed inject_interval to {}", t, val);
                            // Note: inject_interval is only used in the main loop logic
                        }
                    }
                    _ => eprintln!("[tick {}] Unknown parameter: {}", t, change.param),
                }
            } else {
                break;
            }
        }

        // Shock logic
        if shock_prob > 0.0 {
            if shock_state.active {
                // Count down shock duration
                shock_state.remaining_ticks = shock_state.remaining_ticks.saturating_sub(1);
                if shock_state.remaining_ticks == 0 {
                    // Shock ended, restore base death rate
                    rt.death_rate = shock_state.base_death_rate;
                    shock_state.active = false;
                    eprintln!("[tick {}] Shock ended, death_rate restored to {}", t, rt.death_rate);
                    
                    // Start recovery tracking
                    if current_recovery.is_some() {
                        // Recovery already in progress (nested shocks), extend it
                    } else {
                        // Should not happen - recovery should be set at shock start
                        current_recovery = Some(ShockRecovery {
                            shock_start_tick: t - 1,
                            pre_shock_population: rt.running_cells(),
                            recovery_tick: None,
                            min_population: rt.running_cells(),
                        });
                    }
                }
            } else {
                // Random shock trigger
                if rng.gen::<f64>() < shock_prob {
                    shock_state.active = true;
                    shock_state.remaining_ticks = shock_duration;
                    shock_state.base_death_rate = rt.death_rate;
                    let pre_shock_population = rt.running_cells();
                    let shock_start = t;
                    rt.death_rate = shock_death_rate;
                    eprintln!("[tick {}] SHOCK! death_rate -> {} for {} ticks (pre-shock pop: {})", t, shock_death_rate, shock_duration, pre_shock_population);

                    if let Some(ew) = &mut rt.event_writer {
                        let _ = ew.write_shock(t, shock_death_rate, shock_duration, pre_shock_population);
                    }
                    rt.discover_law("L049", "Environmental Variation Maintains Diversity");
                    
                    // Store for recovery tracking
                    current_recovery = Some(ShockRecovery {
                        shock_start_tick: shock_start,
                        pre_shock_population,
                        recovery_tick: None,
                        min_population: pre_shock_population,
                    });
                }
            }
        }
        
        // Track recovery
        if let Some(ref mut recovery) = current_recovery {
            let current_pop = rt.running_cells();
            recovery.min_population = recovery.min_population.min(current_pop);
            // Recovery = population returns to >= 90% of pre-shock level
            if current_pop >= (recovery.pre_shock_population as f64 * 0.9) as usize {
                recovery.recovery_tick = Some(t);
                let rec = current_recovery.take().unwrap();
                shock_recoveries.push(rec.clone());
                if let Some(ew) = &mut rt.event_writer {
                    let _ = ew.write_recovery(
                        rec.shock_start_tick,
                        t,
                        t.saturating_sub(rec.shock_start_tick),
                        rec.pre_shock_population,
                        rec.min_population,
                    );
                }
                rt.discover_law("L053", "Uncertainty Preserves Variance");
            }
        }

        rt.step();

        if (t + 1) % 10 == 0 || t == 0 || t == cli.ticks - 1 {
            let total_energy: u32 = rt.pool.iter().map(|c| c.energy).sum();
            eprintln!(
                "[tick {}] cells: {} | bus: {} pending | emitted: {} | energy: {}",
                rt.tick, rt.running_cells(), rt.bus.pending_count(), rt.stats.total_signals_emitted, total_energy
            );
        }

        std::thread::sleep(Duration::from_millis(10));
    }

    eprintln!("\n--- Stats ---");
    eprintln!("Ticks: {}", rt.stats.total_ticks);
    eprintln!("Cells born: {} | died: {} | energy_deaths: {} | peak: {}",
        rt.stats.total_cells_born, rt.stats.total_cells_died, rt.stats.total_energy_deaths, rt.stats.peak_cells);
    eprintln!("Signals emitted: {} | delivered: {}",
        rt.stats.total_signals_emitted, rt.stats.total_signals_delivered);
    let energy_remaining: u32 = rt.pool.iter().map(|c| c.energy).sum();
    let energy_consumed = rt.stats.total_energy_budget.saturating_sub(energy_remaining as u64);
    eprintln!("Energy budget: {} | consumed: {} | remaining: {}",
        rt.stats.total_energy_budget, energy_consumed, energy_remaining);
    if energy_consumed > 0 {
        let fitness = rt.stats.total_signals_emitted as f64 / energy_consumed as f64;
        eprintln!("Fitness (signals/energy): {:.4}", fitness);
    }

    eprintln!("{}", rt.observer.report());

    // Trait frequency summary
    use std::collections::HashMap;
    let mut cost_counts: HashMap<u32, usize> = HashMap::new();
    let mut mut_counts: HashMap<u32, usize> = HashMap::new(); // scaled * 10000
    let mut threshold_counts: HashMap<u32, usize> = HashMap::new();
    let mut energy_counts: HashMap<u32, usize> = HashMap::new();
    
    let alive_cells: Vec<_> = rt.pool.iter().filter(|c| c.alive).collect();
    let alive_count = alive_cells.len();
    
    for cell in &alive_cells {
        let g = &cell.gene;
        *cost_counts.entry(g.reproduction_cost).or_insert(0) += 1;
        *mut_counts.entry((g.mutation_rate * 10000.0) as u32).or_insert(0) += 1;
        *threshold_counts.entry(g.reproduction_threshold).or_insert(0) += 1;
        *energy_counts.entry(g.initial_energy).or_insert(0) += 1;
    }
    
    eprintln!("\n--- Trait Frequencies (alive: {}) ---", alive_count);
    eprintln!("reproduction_cost:");
    let mut costs: Vec<_> = cost_counts.into_iter().collect();
    costs.sort_by_key(|&(k, _)| k);
    for (cost, count) in costs {
        eprintln!("  cost={}: {} cells ({:.1}%)", cost, count, count as f64 / alive_count as f64 * 100.0);
    }
    eprintln!("mutation_rate:");
    let mut muts: Vec<_> = mut_counts.into_iter().collect();
    muts.sort_by_key(|&(k, _)| k);
    for (mr, count) in muts {
        eprintln!("  mut={:.4}: {} cells ({:.1}%)", mr as f64 / 10000.0, count, count as f64 / alive_count as f64 * 100.0);
    }
    eprintln!("reproduction_threshold:");
    let mut threshs: Vec<_> = threshold_counts.into_iter().collect();
    threshs.sort_by_key(|&(k, _)| k);
    for (thresh, count) in threshs {
        eprintln!("  thresh={}: {} cells ({:.1}%)", thresh, count, count as f64 / alive_count as f64 * 100.0);
    }
    eprintln!("initial_energy:");
    let mut energies: Vec<_> = energy_counts.into_iter().collect();
    energies.sort_by_key(|&(k, _)| k);
    for (energy, count) in energies {
        eprintln!("  energy={}: {} cells ({:.1}%)", energy, count, count as f64 / alive_count as f64 * 100.0);
    }

    // Recovery statistics
    if !shock_recoveries.is_empty() {
        eprintln!("\n--- Shock Recovery Stats ---");
        eprintln!("Total shocks with recovery: {}", shock_recoveries.len());
        let mut recovery_times: Vec<u64> = shock_recoveries.iter()
            .filter_map(|r| r.recovery_tick.map(|rt| rt.saturating_sub(r.shock_start_tick)))
            .collect();
        if !recovery_times.is_empty() {
            recovery_times.sort();
            let sum: u64 = recovery_times.iter().sum();
            let mean = sum as f64 / recovery_times.len() as f64;
            let median = recovery_times[recovery_times.len() / 2];
            let min = recovery_times[0];
            let max = recovery_times[recovery_times.len() - 1];
            eprintln!("Recovery time (ticks): mean={:.1} median={} min={} max={}", mean, median, min, max);
            
            // Per-shock detail
            for (i, r) in shock_recoveries.iter().enumerate() {
                if let Some(rt) = r.recovery_tick {
                    let duration = rt.saturating_sub(r.shock_start_tick);
                    eprintln!("  Shock {}: start={} recovery={} duration={} pre_pop={}", 
                        i+1, r.shock_start_tick, rt, duration, r.pre_shock_population);
                } else {
                    eprintln!("  Shock {}: start={} NO RECOVERY pre_pop={}", 
                        i+1, r.shock_start_tick, r.pre_shock_population);
                }
            }
        }
    }

    Ok(())
}

fn parse_signal(s: &str) -> Option<Signal> {
    let parts: Vec<&str> = s.split(',').collect();
    if parts.len() < 3 {
        eprintln!("signal format: type_id,source_id,target_id");
        return None;
    }
    let type_id = parts[0].parse().ok()?;
    let source_id = parts[1].parse().ok()?;
    let target_id = parts[2].parse().ok()?;
    Some(Signal::new(type_id, source_id, target_id))
}
