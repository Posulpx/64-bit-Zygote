use clap::Parser;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader, Seek, SeekFrom, Write};
use std::path::PathBuf;
use std::time::Duration;
use std::thread::sleep;

use crate::types::{EventRecord, CellInfo, TypeStats};

#[derive(Parser, Debug)]
#[command(name = "zmon", about = "ZygoteAI Monitor - Text-based live monitor")]
struct Args {
    /// Path to events.jsonl log file
    #[arg(short, long, default_value = "events.jsonl")]
    log: PathBuf,

    /// Refresh interval in milliseconds
    #[arg(short, long, default_value_t = 1000)]
    refresh: u64,

    /// Show only summary (no detailed tables)
    #[arg(long)]
    summary: bool,

    /// Show network topology
    #[arg(long)]
    network: bool,

    /// Show trait histograms
    #[arg(long)]
    traits: bool,

    /// Show leaderboard
    #[arg(long)]
    leaderboard: bool,

    /// Show the consolidated dashboard (all six views)
    #[arg(long)]
    dashboard: bool,
}

#[derive(Debug, Default)]
struct MonitorState {
    tick: u64,
    population: usize,
    total_births: u64,
    total_deaths: u64,
    total_signals: u64,
    energy: u64,
    
    cells: HashMap<u32, CellInfo>,
    max_cell_id: u32,
    
    cost_dist: HashMap<String, usize>,
    mut_dist: HashMap<String, usize>,
    thresh_dist: HashMap<String, usize>,
    energy_dist: HashMap<String, usize>,
    gen_dist: HashMap<String, usize>,
    
    signal_edges: HashMap<(String, String), u64>,
    cell_types: HashMap<String, TypeStats>,

    // ---- Dashboard state ----
    pop_history: Vec<(u64, usize)>, // (tick, population) rolling window
    lineage_dist: HashMap<String, usize>,
    shocks: Vec<ShockAlert>,
    active_shock: Option<ShockAlert>,
    last_population: usize,
    extinction_alerts: Vec<ExtinctionAlert>,
    discovered_laws: std::collections::HashSet<String>,
}

#[derive(Debug, Clone)]
struct ShockAlert {
    start_tick: u64,
    death_rate: f64,
    duration: u64,
    pre_shock_pop: usize,
    min_pop: usize,
    recovery_tick: Option<u64>,
}

#[derive(Debug, Clone)]
struct ExtinctionAlert {
    tick: u64,
    from_pop: usize,
    to_pop: usize,
    drop_pct: u64,
    cause: String,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    
    let mut state = MonitorState::default();

    println!("ZMON - ZygoteAI Monitor");
    println!("Log: {:?}", args.log);
    println!("Refresh: {}ms", args.refresh);
    println!("Press Ctrl+C to exit\n");
    
    let mut file = File::open(&args.log)?;
    let mut position = 0u64;
    
    loop {
        // Read new lines from log file
        if let Ok(_) = file.seek(SeekFrom::Start(position)) {
            let reader = BufReader::new(&file);
            for line in reader.lines().flatten() {
                if let Ok(record) = serde_json::from_str::<EventRecord>(&line) {
                    state.ingest_event(record);
                }
            }
            position = file.stream_position().unwrap_or(position);
        }
        
        // Clear screen and render
        print!("\x1B[2J\x1B[1;1H"); // Clear screen, move cursor to top-left
        if args.dashboard {
            render_dashboard(&state, &args);
        } else {
            render(&state, &args);
        }
        std::io::stdout().flush().unwrap();
        
        sleep(Duration::from_millis(args.refresh));
    }
}

impl MonitorState {
    fn ingest_event(&mut self, event: EventRecord) {
        match event {
            EventRecord::Tick(e) => {
                // Population-drop extinction detection (requires a prior sample)
                if self.last_population > 0 && e.population < self.last_population {
                    let drop = self.last_population - e.population;
                    let pct = (drop as f64 / self.last_population as f64 * 100.0) as u64;
                    if pct >= 25 {
                        self.extinction_alerts.push(ExtinctionAlert {
                            tick: e.tick,
                            from_pop: self.last_population,
                            to_pop: e.population,
                            drop_pct: pct,
                            cause: "population crash".to_string(),
                        });
                        // cap history of alerts
                        if self.extinction_alerts.len() > 50 {
                            self.extinction_alerts.remove(0);
                        }
                    }
                }
                self.last_population = e.population;

                self.tick = e.tick;
                self.population = e.population;
                self.total_signals = e.signals_emitted;
                self.energy = e.energy;

                self.pop_history.push((e.tick, e.population));
                if self.pop_history.len() > 120 {
                    self.pop_history.remove(0);
                }
            }
            EventRecord::Birth(e) => {
                self.total_births += 1;
                self.max_cell_id = self.max_cell_id.max(e.cell_id);
                
                let cell = CellInfo {
                    id: e.cell_id,
                    cell_type: e.cell_type.clone(),
                    generation: e.generation,
                    birth_tick: e.tick,
                    parent_id: e.parent_id,
                    reproduction_cost: e.reproduction_cost,
                    reproduction_threshold: e.reproduction_threshold,
                    mutation_rate: e.mutation_rate,
                    initial_energy: e.initial_energy,
                    affinities: e.affinities,
                    mutation_step: e.mutation_step,
                    decay_rate: e.decay_rate,
                    meta_mutation_rate: e.meta_mutation_rate,
                    energy: e.initial_energy,
                    signals_emitted: 0,
                    signals_received: 0,
                    alive: true,
                    death_tick: None,
                    death_cause: None,
                };
                
                self.cells.insert(e.cell_id, cell);
                
                let type_stats = self.cell_types.entry(e.cell_type).or_default();
                type_stats.count += 1;
                type_stats.births += 1;
            }
            EventRecord::Death(e) => {
                self.total_deaths += 1;
                if let Some(cell) = self.cells.get_mut(&e.cell_id) {
                    cell.alive = false;
                    cell.death_tick = Some(e.tick);
                    cell.death_cause = Some(e.cause);
                }
                
                if let Some(type_stats) = self.cell_types.get_mut(&e.cell_type) {
                    type_stats.count = type_stats.count.saturating_sub(1);
                    type_stats.deaths += 1;
                }
            }
            EventRecord::Signal(e) => {
                self.total_signals = self.total_signals.max(e.signal_id);
                
                if let Some(source) = self.cells.get_mut(&e.source_id) {
                    source.signals_emitted += 1;
                }
                if let Some(target) = self.cells.get_mut(&e.target_id) {
                    target.signals_received += 1;
                }
                
                let edge_key = (e.source_type.clone(), e.target_type.clone());
                *self.signal_edges.entry(edge_key).or_insert(0) += 1;
                
                if let Some(ts) = self.cell_types.get_mut(&e.source_type) {
                    ts.signals_emitted += 1;
                }
                if let Some(ts) = self.cell_types.get_mut(&e.target_type) {
                    ts.signals_received += 1;
                }
            }
            EventRecord::Snapshot(e) => {
                // Keep the most recent non-empty distribution so extinction
                // (empty final snapshot) does not wipe accumulated trait data.
                if !e.cost_distribution.is_empty() { self.cost_dist = e.cost_distribution; }
                if !e.mut_distribution.is_empty() { self.mut_dist = e.mut_distribution; }
                if !e.thresh_distribution.is_empty() { self.thresh_dist = e.thresh_distribution; }
                if !e.energy_distribution.is_empty() { self.energy_dist = e.energy_distribution; }
                if !e.generation_distribution.is_empty() { self.gen_dist = e.generation_distribution; }
                if !e.lineage_distribution.is_empty() { self.lineage_dist = e.lineage_distribution; }

                for (cell_type, stats) in e.by_type {
                    if stats.count > 0 {
                        self.cell_types.entry(cell_type).or_default().count = stats.count;
                    }
                }
            }
            EventRecord::Shock(e) => {
                let alert = ShockAlert {
                    start_tick: e.tick,
                    death_rate: e.death_rate,
                    duration: e.duration,
                    pre_shock_pop: e.pre_shock_population,
                    min_pop: e.pre_shock_population,
                    recovery_tick: None,
                };
                self.active_shock = Some(alert);
            }
            EventRecord::Recovery(e) => {
                let alert = ShockAlert {
                    start_tick: e.shock_start_tick,
                    death_rate: 0.0,
                    duration: e.duration_ticks,
                    pre_shock_pop: e.pre_shock_population,
                    min_pop: e.min_population,
                    recovery_tick: Some(e.recovery_tick),
                };
                self.shocks.push(alert.clone());
                self.active_shock = None;
                // Alert on severe bottleneck (population fell below 50% of pre-shock)
                if e.min_population < (e.pre_shock_population / 2) {
                    self.extinction_alerts.push(ExtinctionAlert {
                        tick: e.shock_start_tick,
                        from_pop: e.pre_shock_population,
                        to_pop: e.min_population,
                        drop_pct: (100.0 - (e.min_population as f64 / e.pre_shock_population.max(1) as f64 * 100.0)) as u64,
                        cause: "shock bottleneck".to_string(),
                    });
                    if self.extinction_alerts.len() > 50 {
                        self.extinction_alerts.remove(0);
                    }
                }
            }
            EventRecord::Law(e) => {
                self.discovered_laws.insert(e.law_id);
            }
            _ => {}
        }
    }
    
    fn fitness(&self) -> f64 {
        if self.energy > 0 {
            self.total_signals as f64 / self.energy as f64
        } else {
            0.0
        }
    }
    
    fn sorted_cells(&self) -> Vec<&CellInfo> {
        let mut cells: Vec<_> = self.cells.values().filter(|c| c.alive).collect();
        cells.sort_by(|a, b| b.generation.cmp(&a.generation));
        cells
    }
}

fn render(state: &MonitorState, args: &Args) {
    // Build dynamic table for cell types
    let mut type_rows = Vec::new();
    for (cell_type, stats) in &state.cell_types {
        if stats.count > 0 {
            type_rows.push((
                cell_type.clone(),
                stats.count.to_string(),
                stats.births.to_string(),
                stats.deaths.to_string(),
                stats.signals_emitted.to_string(),
                stats.signals_received.to_string(),
            ));
        }
    }

    // ---- Header panel (always present) ----
    let header_line = format!(
        "│ Population: {:>5} | Births: {:>6} | Deaths: {:>6} | Signals: {:>8} │",
        state.population, state.total_births, state.total_deaths, state.total_signals
    );
    let header_line2 = format!(
        "│ Energy: {:>12} | Fitness: {:>8.4}                            │",
        state.energy, state.fitness()
    );
    let inner_w = header_line.chars().count() - 2; // strip the two │ borders
    let top = format!("┌{}┐", "─".repeat(inner_w));
    let mid = format!("├{}┤", "─".repeat(inner_w));
    let bot = format!("└{}┘", "─".repeat(inner_w));

    println!("{}", top);
    println!("│ ZMON - ZygoteAI Monitor                                    Tick {:>6} │", state.tick);
    println!("{}", mid);
    println!("{}", header_line);
    println!("{}", header_line2);
    println!("{}", bot);

    // ---- Cell Types panel (its own box) ----
    if !type_rows.is_empty() {
        let name_w = type_rows.iter().map(|r| r.0.len()).max().unwrap_or(10).max(4);
        let count_w = type_rows.iter().map(|r| r.1.len()).max().unwrap_or(5).max(5);
        let born_w = type_rows.iter().map(|r| r.2.len()).max().unwrap_or(4).max(4);
        let died_w = type_rows.iter().map(|r| r.3.len()).max().unwrap_or(4).max(4);
        let emit_w = type_rows.iter().map(|r| r.4.len()).max().unwrap_or(5).max(5);
        let recv_w = type_rows.iter().map(|r| r.5.len()).max().unwrap_or(5).max(5);

        let inner = name_w + count_w + born_w + died_w + emit_w + recv_w + 19;
        let t = format!("┌{}┐", "─".repeat(inner));
        let s = format!("├{}┤", "─".repeat(inner));
        let b = format!("└{}┘", "─".repeat(inner));

        println!("{}", t);
        println!("│ Cell Types ({})", state.cell_types.len());
        // header row
        let mut h = String::from("│ ");
        h.push_str(&pad_left("Type", name_w)); h.push_str(" │ ");
        h.push_str(&pad_right("Count", count_w)); h.push_str(" │ ");
        h.push_str(&pad_right("Born", born_w)); h.push_str(" │ ");
        h.push_str(&pad_right("Died", died_w)); h.push_str(" │ ");
        h.push_str(&pad_right("Emit", emit_w)); h.push_str(" │ ");
        h.push_str(&pad_right("Recv", recv_w)); h.push_str(" │");
        println!("{}", h);
        println!("{}", s);
        for (name, count, born, died, emit, recv) in &type_rows {
            let mut line = String::from("│ ");
            line.push_str(&pad_left(name, name_w)); line.push_str(" │ ");
            line.push_str(&pad_right(count, count_w)); line.push_str(" │ ");
            line.push_str(&pad_right(born, born_w)); line.push_str(" │ ");
            line.push_str(&pad_right(died, died_w)); line.push_str(" │ ");
            line.push_str(&pad_right(emit, emit_w)); line.push_str(" │ ");
            line.push_str(&pad_right(recv, recv_w)); line.push_str(" │");
            println!("{}", line);
        }
        println!("{}", b);
    }

    // ---- Network topology panel ----
    if args.network && !state.signal_edges.is_empty() {
        let mut edges: Vec<_> = state.signal_edges.iter().collect();
        edges.sort_by(|a, b| b.1.cmp(a.1));
        let mut lines: Vec<String> = Vec::new();
        let mut max_w = "Network Topology".len();
        for ((src, tgt), count) in edges.iter().take(15) {
            let l = format!("  {} ──[{:>5}]──> {}", src, count, tgt);
            max_w = max_w.max(l.chars().count());
            lines.push(l);
        }
        let inner = max_w;
        println!("{}", format!("┌{}┐", "─".repeat(inner)));
        println!("│ Network Topology{}│", " ".repeat(inner - "Network Topology".len()));
        println!("{}", format!("├{}┤", "─".repeat(inner)));
        for l in &lines {
            println!("│ {}{}│", l, " ".repeat(inner - l.chars().count()));
        }
        println!("{}", format!("└{}┘", "─".repeat(inner)));
    }

    // ---- Traits panel ----
    if args.traits {
        let mut blocks: Vec<Vec<String>> = Vec::new();
        let mut max_w = "Trait Distributions".len();
        let mut collect = |name: &str, dist: &HashMap<String, usize>| {
            let mut items: Vec<_> = dist.iter().collect();
            items.sort_by_key(|(k, _)| k.parse::<u32>().unwrap_or(0));
            let max_count = items.iter().map(|(_, v)| *v).max().copied().unwrap_or(1);
            let mut lines = vec![format!("│ {:<8} │", name)];
            for (k, v) in items.iter().take(10) {
                let bar = "█".repeat(((**v as f64 / max_count as f64) * 40.0) as usize);
                let l = format!("  {:>4}: {:<40} ({:>4})", k, bar, **v);
                max_w = max_w.max(l.chars().count());
                lines.push(l);
            }
            blocks.push(lines);
        };
        collect("Cost", &state.cost_dist);
        collect("MutRate", &state.mut_dist);
        collect("Thresh", &state.thresh_dist);
        let inner = max_w;
        println!("{}", format!("┌{}┐", "─".repeat(inner)));
        println!("│ Trait Distributions{}│", " ".repeat(inner - "Trait Distributions".len()));
        println!("{}", format!("├{}┤", "─".repeat(inner)));
        for block in &blocks {
            for (i, l) in block.iter().enumerate() {
                if i == 0 {
                    println!("{}", l);
                } else {
                    println!("│ {}{}│", l, " ".repeat(inner - l.chars().count()));
                }
            }
        }
        println!("{}", format!("└{}┘", "─".repeat(inner)));
    }

    // ---- Leaderboard panel ----
    if args.leaderboard {
        let cells = state.sorted_cells();
        let type_w = cells.iter().map(|c| c.cell_type.len()).max().unwrap_or(4).max(4);
        let cost_w = cells.iter().map(|c| c.reproduction_cost.to_string().len()).max().unwrap_or(4).max(4);
        let thresh_w = cells.iter().map(|c| c.reproduction_threshold.to_string().len()).max().unwrap_or(6).max(6);
        let energy_w = cells.iter().map(|c| c.energy.to_string().len()).max().unwrap_or(6).max(6);
        let emit_w = cells.iter().map(|c| c.signals_emitted.to_string().len()).max().unwrap_or(7).max(7);
        let recv_w = cells.iter().map(|c| c.signals_received.to_string().len()).max().unwrap_or(7).max(7);

        let cols = [5, 4, type_w, cost_w, 8, thresh_w, energy_w, emit_w, recv_w, 5];
        let inner = cols.iter().sum::<usize>() + 31;
        let t = format!("┌{}┐", "─".repeat(inner));
        let s = format!("├{}┤", "─".repeat(inner));
        let b = format!("└{}┘", "─".repeat(inner));

        println!("{}", t);
        println!("│ Leaderboard (Top 20 by Generation):");
        // header
        let mut h = String::from("│ ");
        h.push_str(&pad_right("ID", 5)); h.push(' ');
        h.push_str(&pad_right("Gen", 4)); h.push(' ');
        h.push_str(&pad_left("Type", type_w)); h.push(' ');
        h.push_str(&pad_right("Cost", cost_w)); h.push(' ');
        h.push_str(&pad_right("MutRate", 8)); h.push(' ');
        h.push_str(&pad_right("Thresh", thresh_w)); h.push(' ');
        h.push_str(&pad_right("Energy", energy_w)); h.push(' ');
        h.push_str(&pad_right("Emitted", emit_w)); h.push(' ');
        h.push_str(&pad_right("Recv", recv_w)); h.push(' ');
        h.push_str(&pad_right("Age", 5)); h.push(' ');
        h.push('│');
        println!("{}", h);
        println!("{}", s);
        for cell in cells.iter().take(20) {
            let mut line = String::from("│ ");
            line.push_str(&pad_right(&cell.id.to_string(), 5)); line.push(' ');
            line.push_str(&pad_right(&cell.generation.to_string(), 4)); line.push(' ');
            line.push_str(&pad_left(&cell.cell_type, type_w)); line.push(' ');
            line.push_str(&pad_right(&cell.reproduction_cost.to_string(), cost_w)); line.push(' ');
            line.push_str(&pad_right(&format!("{:.4}", cell.mutation_rate), 8)); line.push(' ');
            line.push_str(&pad_right(&cell.reproduction_threshold.to_string(), thresh_w)); line.push(' ');
            line.push_str(&pad_right(&cell.energy.to_string(), energy_w)); line.push(' ');
            line.push_str(&pad_right(&cell.signals_emitted.to_string(), emit_w)); line.push(' ');
            line.push_str(&pad_right(&cell.signals_received.to_string(), recv_w)); line.push(' ');
            line.push_str(&pad_right(&state.tick.saturating_sub(cell.birth_tick).to_string(), 5)); line.push(' ');
            line.push('│');
            println!("{}", line);
        }
        println!("{}", b);
    }
}

// Build a bordered box from a title and content lines. Returns full box lines.
fn build_box(title: &str, content: &[String]) -> Vec<String> {
    let inner = content.iter()
        .chain(std::iter::once(&title.to_string()))
        .map(|l| l.chars().count())
        .max()
        .unwrap_or(title.len())
        .max(12)
        .max(36); // keep columns aligned in the 2-col grid
    let top = format!("┌{}┐", "─".repeat(inner));
    let sep = format!("├{}┤", "─".repeat(inner));
    let bot = format!("└{}┘", "─".repeat(inner));
    let mut lines = vec![top, format!("│ {}{}│", title, " ".repeat(inner - title.chars().count()))];
    if !content.is_empty() {
        lines.push(sep);
        for l in content {
            lines.push(format!("│ {}{}│", l, " ".repeat(inner - l.chars().count())));
        }
    }
    lines.push(bot);
    lines
}

// Render a list of bordered boxes (each a Vec<String> of equal-or-unequal height)
// into a 2-column grid, top-aligned and space-separated. Within a row, both boxes
// are widened to the row's maximum width so vertical edges align. Returns combined lines.
fn render_grid(boxes: &[Vec<String>]) -> Vec<String> {
    let cols = 2;
    let gap = 2;
    let mut out: Vec<String> = Vec::new();
    let mut row_start = 0;
    while row_start < boxes.len() {
        let row: Vec<&Vec<String>> = boxes[row_start..(row_start + cols).min(boxes.len())].iter().collect();
        let heights: Vec<usize> = row.iter().map(|b| b.len()).collect();
        let max_h = heights.iter().copied().max().unwrap_or(0);
        let widths: Vec<usize> = row.iter().map(|b| b[0].chars().count()).collect();
        let row_w = widths.iter().copied().max().unwrap_or(0);
        let padded: Vec<Vec<String>> = row.iter().enumerate()
            .map(|(ci, b)| widen_box(b, widths[ci], row_w))
            .collect();
        for r in 0..max_h {
            let mut line = String::new();
            for (ci, b) in padded.iter().enumerate() {
                if r < b.len() {
                    line.push_str(&b[r]);
                } else {
                    line.push_str(&" ".repeat(row_w));
                }
                if ci + 1 < padded.len() {
                    line.push_str(&" ".repeat(gap));
                }
            }
            out.push(line);
        }
        row_start += cols;
    }
    out
}

// Widen every line of a box to `target_w` by padding before its closing border `│`.
fn widen_box(box_lines: &[String], cur_w: usize, target_w: usize) -> Vec<String> {
    if cur_w >= target_w {
        return box_lines.to_vec();
    }
    let extra = target_w - cur_w;
    box_lines.iter().map(|l| {
        let chars: Vec<char> = l.chars().collect();
        if let Some(pos) = chars.iter().rposition(|&c| c == '│') {
            let mut out: Vec<char> = chars[..pos].to_vec();
            out.extend(std::iter::repeat(' ').take(extra));
            out.push('│');
            out.into_iter().collect()
        } else {
            let mut s = l.clone();
            s.push_str(&" ".repeat(extra));
            s
        }
    }).collect()
}

fn sparkline(history: &[(u64, usize)], width: usize) -> String {
    if history.is_empty() {
        return " ".repeat(width);
    }
    let max_v = history.iter().map(|(_, v)| *v).max().unwrap_or(1).max(1);
    let n = history.len();
    let ramp = "▁▂▃▄▅▆▇█";
    let mut out = String::new();
    for i in 0..width {
        let idx = if width >= n {
            if i < n { i } else { n - 1 }
        } else {
            (i * (n - 1)) / (width - 1)
        };
        let (_, v) = history[idx];
        let lvl = ((v as f64 / max_v as f64) * (ramp.chars().count() - 1) as f64) as usize;
        let ch = ramp.chars().nth(lvl).unwrap_or(' ');
        out.push(ch);
    }
    out
}

fn render_dashboard(state: &MonitorState, _args: &Args) {
    // Header
    println!("┌{}┐", "─".repeat(78));
    println!("│ ZMON Consolidated Dashboard                              Tick {:>6} │", state.tick);
    println!("└{}┘", "─".repeat(78));

    // ---- 1. Population View ----
    let mut pop_content = Vec::new();
    let spark = sparkline(&state.pop_history, 40);
    let peak = state.pop_history.iter().map(|(_, v)| *v).max().unwrap_or(state.population);
    let min_p = state.pop_history.iter().map(|(_, v)| *v).min().unwrap_or(state.population);
    pop_content.push(format!("Pop: {:>4}  Peak: {:>4}  Min: {:>4}", state.population, peak, min_p));
    pop_content.push(format!("{}", spark));
    pop_content.push(format!("Births: {:>5}  Deaths: {:>5}", state.total_births, state.total_deaths));
    pop_content.push(format!("Samples: {}", state.pop_history.len()));
    let box_pop = build_box("1. Population View", &pop_content);

    // ---- 2. Trait Frequency View ----
    let mut trait_content = Vec::new();
    trait_content.push(bar_line("Cost", &state.cost_dist));
    trait_content.push(bar_line("Mut", &state.mut_dist));
    trait_content.push(bar_line("Thr", &state.thresh_dist));
    let box_trait = build_box("2. Trait Frequency", &trait_content);

    // ---- 3. Lineage View ----
    let mut lineage_content = Vec::new();
    let mut lin: Vec<_> = state.lineage_dist.iter().collect();
    lin.sort_by(|a, b| b.1.cmp(a.1));
    let total_lin = lin.iter().map(|(_, v)| **v).sum::<usize>().max(1);
    if lin.is_empty() {
        lineage_content.push("(no data)".to_string());
    }
    for (k, v) in lin.iter().take(8) {
        let pct = (**v as f64 / total_lin as f64 * 100.0) as u32;
        lineage_content.push(format!("{}: {} ({}%)", k, **v, pct));
    }
    let box_lineage = build_box("3. Lineage View", &lineage_content);

    // ---- 4. Energy Distribution ----
    let mut energy_content = Vec::new();
    energy_content.push(format!("Total energy: {}", state.energy));
    let mut en: Vec<_> = state.energy_dist.iter().collect();
    en.sort_by_key(|(k, _)| k.parse::<u32>().unwrap_or(0));
    if en.is_empty() {
        energy_content.push("(no data)".to_string());
    }
    for (k, v) in en.iter().take(8) {
        energy_content.push(format!("E={}: x{}", k, **v));
    }
    let box_energy = build_box("4. Energy Distribution", &energy_content);

    // ---- 5. Signal Heatmap ----
    let mut heat_content = Vec::new();
    let edges: Vec<_> = state.signal_edges.iter().collect();
    if edges.is_empty() {
        heat_content.push("(no signals)".to_string());
    } else {
        // Collect unique source/target types
        let mut types: Vec<String> = edges.iter()
            .flat_map(|((s, t), _)| vec![s.clone(), t.clone()])
            .collect::<std::collections::HashSet<_>>()
            .into_iter().collect();
        types.sort();
        if types.len() <= 8 {
            // header: sources as columns
            let header = format!("     {}", types.iter().map(|t| pad_left(&trunc(&t, 3), 4)).collect::<String>());
            heat_content.push(header);
            let max_edge = edges.iter().map(|(_, c)| **c).max().unwrap_or(1).max(1);
            for src in &types {
                let mut row = format!("{} ", pad_right(&trunc(src, 4), 4));
                for tgt in &types {
                    let cnt = state.signal_edges.get(&(src.clone(), tgt.clone())).copied().unwrap_or(0);
                    let cell = if cnt == 0 {
                        "  . ".to_string()
                    } else {
                        let lvl = ((cnt as f64 / max_edge as f64) * 9.0) as usize;
                        let glyph = " .:-=+*#%@".chars().nth(lvl).unwrap_or('.');
                        format!(" {} ", glyph)
                    };
                    row.push_str(&cell);
                }
                heat_content.push(row);
            }
            heat_content.push("scale: .:-=+*#%@ (low->high)".to_string());
        } else {
            heat_content.push(format!("{} cell types (too many for matrix)", types.len()));
        }
    }
    let box_heat = build_box("5. Signal Heatmap", &heat_content);

    // ---- 6. Extinction Alerts ----
    let mut ext_content = Vec::new();
    if let Some(s) = &state.active_shock {
        ext_content.push(format!("⚠ ACTIVE SHOCK @tick {}", s.start_tick));
        ext_content.push(format!("  death_rate={:.3} dur={}", s.death_rate, s.duration));
        ext_content.push(format!("  pre_pop={} min_pop={}", s.pre_shock_pop, s.min_pop));
    }
    let recent: Vec<_> = state.extinction_alerts.iter().rev().take(6).collect();
    if recent.is_empty() && state.active_shock.is_none() {
        ext_content.push("No extinction events.".to_string());
    }
    for a in recent {
        ext_content.push(format!("@{} {} -{}% ({}->{})", a.tick, a.cause, a.drop_pct, a.from_pop, a.to_pop));
    }
    if !state.shocks.is_empty() {
        let recovered = state.shocks.iter().filter(|s| s.recovery_tick.is_some()).count();
        ext_content.push(format!("Shocks: {} recovered / {}", recovered, state.shocks.len()));
    }
    let box_ext = build_box("6. Extinction Alerts", &ext_content);

    let left = render_grid(&[box_pop, box_trait, box_lineage, box_energy, box_heat, box_ext]);
    let right = render_law_box(state);
    print_side_by_side(&left, &right);
}

// Full catalog of Zygote Laws (id, title) from ZYGOTE_LAWS.md.
const ZYGOTE_LAWS: &[(&str, &str)] = &[
    ("L001", "Affinity Is Subscription, Not Routing"),
    ("L002", "Broadcast Is Cooperative, Not Competitive"),
    ("L003", "Throughput Is Limited By Bottlenecks"),
    ("L004", "Static Topologies Are Fragile"),
    ("L005", "Pool Order Creates Bias"),
    ("L006", "Fairness Depends On Relative Ordering"),
    ("L007", "The ABI Is Part Of Reality"),
    ("L008", "Directed Signals Bypass Affinity"),
    ("L009", "Cells Can Choose Death"),
    ("L010", "Adaptation Converts Extinction Into Degradation"),
    ("L011", "Population Growth Requires Regulation"),
    ("L012", "Information Processing Has Metabolic Cost"),
    ("L013", "Topology Determines Survival"),
    ("L014", "Workload Creates Lifespan Inequality"),
    ("L015", "Adaptation Is Not Free"),
    ("L016", "Initial Endowment Dominates Efficiency"),
    ("L017", "Food Creates An Ecosystem"),
    ("L018", "Reproduction Costs Position"),
    ("L019", "Reproduction Requires Metabolic Surplus"),
    ("L020", "Heredity Enables Evolution"),
    ("L021", "Tiny Mutations Enable Gradual Adaptation"),
    ("L022", "Trait Frequencies Record Evolutionary History"),
    ("L023", "Pool Order Masks Cost Advantage"),
    ("L024", "Hard Population Caps Prevent Selective Sweeps"),
    ("L025", "Removing One Bottleneck Reveals The Next"),
    ("L026", "Hard Caps Prevent Displacement"),
    ("L027", "Selection Acts Through The Least-Constrained Dimension"),
    ("L028", "Mortality Unfreezes Selection"),
    ("L029", "Reproductive Rate Dominates When Slots Turn Over"),
    ("L030", "Fitness Is Multidimensional"),
    ("L031", "Reproductive Efficiency Has Viability Thresholds"),
    ("L032", "Fitness Is A Gradient, Not A Threshold"),
    ("L033", "Interior Optima Emerge From Tradeoff Balance"),
    ("L034", "Fitness Gradients Enable Hill-Climbing"),
    ("L035", "Robustness Trades Against Efficiency"),
    ("L036", "Fitness Landscapes Are Surfaces, Not Curves"),
    ("L037", "Mutation Sensitivity Defines Landscape Topology"),
    ("L038", "Evolution Prefers Reachable Peaks"),
    ("L039", "Evolutionary Speed Is Itself Evolvable"),
    ("L040", "Convergence Requires Deep Time"),
    ("L041", "Heritable Parameters Are Subject To Selection"),
    ("L042", "Fast Meta-Evolution Wins in Stable Environments"),
    ("L043", "Single-Producer Bottleneck Caps Throughput"),
    ("L044", "Mortality + Earned Reproduction = Adaptability Selection"),
    ("L045", "Environmental Variation Maintains Genetic Diversity"),
    ("L046", "Stress Selects for Adaptability Over Optimization"),
    ("L047", "Over-Optimization Reduces Adaptability"),
    ("L048", "Mutation Rate Evolves to Minimum During Stability"),
    ("L049", "Unpredictability Maintains Broader Distributions"),
    ("L050", "Stochastic Shocks Select for Bet-Hedging"),
    ("L051", "Uncertainty Prevents Tight Convergence"),
    ("L052", "Recovery Time Is Signal-Limited at Capacity"),
    ("L053", "Uncertainty Preserves Variance"),
    ("L054", "Environmental Variation Maintains Diversity"),
    ("L055", "Information Is The Real Currency"),
    ("L056", "Predictive Information Has Fitness Value"),
    ("L057", "Predictive Information Increases Fitness In Partially Predictable Environments"),
    ("L058", "Lineage Crossover Flows Toward the Fittr Strategy"),
    ("L059", "Fixation Reduces Diversity And Creates Systemic Extinction Risk"),
    ("L060", "Communication Converts Individual Information Into Shared Information"),
    ("L061", "A Single Predictor Can Protect A Whole Lineage"),
    ("L062", "Cheap Listeners Free-Ride On Expensive Predictors"),
    ("L063", "Selective Communication Protects Cooperative Information Systems"),
    ("L064", "Information Ecosystems Require A Minimum Producer Fraction"),
];

fn render_law_box(state: &MonitorState) -> Vec<String> {
    let mut content = Vec::new();
    let discovered = state.discovered_laws.len();
    for (id, title) in ZYGOTE_LAWS {
        let marked = state.discovered_laws.contains(*id);
        if marked {
            content.push(format!("● {} {}", id, title));
        } else {
            content.push(format!("  {} {}", id, title));
        }
    }
    let title = format!("Laws Discovered ({}/{})", discovered, ZYGOTE_LAWS.len());
    build_box(&title, &content)
}

// Print two blocks of boxed lines side by side, top-aligned.
fn print_side_by_side(left: &[String], right: &[String]) {
    let gap = 2;
    let max_h = left.len().max(right.len());
    let lw = left.iter().map(|l| l.chars().count()).max().unwrap_or(0);
    for r in 0..max_h {
        let l_raw = left.get(r).map(|s| s.as_str()).unwrap_or("");
        let mut line = format!("{}{}", l_raw, " ".repeat(lw.saturating_sub(l_raw.chars().count())));
        if r < right.len() {
            line.push_str(&" ".repeat(gap));
            line.push_str(&right[r]);
        }
        println!("{}", line);
    }
}

fn bar_line(name: &str, dist: &HashMap<String, usize>) -> String {
    let mut items: Vec<_> = dist.iter().collect();
    items.sort_by_key(|(k, _)| k.parse::<u32>().unwrap_or(0));
    let max_count = items.iter().map(|(_, v)| **v).max().unwrap_or(1).max(1);
    let mut parts = vec![format!("{}:", name)];
    for (k, v) in items.iter().take(4) {
        let bar = "█".repeat(((**v as f64 / max_count as f64) * 12.0) as usize);
        parts.push(format!("{}:{}", k, bar));
    }
    parts.join(" ")
}

fn trunc(s: &str, n: usize) -> String {
    if s.chars().count() <= n {
        s.to_string()
    } else {
        s.chars().take(n).collect()
    }
}

// Right-align text within width (numbers/headers)
fn pad_right(s: &str, w: usize) -> String {
    format!("{:>width$}", s, width = w)
}

// Left-align text within width (names)
fn pad_left(s: &str, w: usize) -> String {
    format!("{:<width$}", s, width = w)
}

mod types {
    use serde::{Deserialize, Serialize};
    use std::collections::HashMap;

    #[derive(Debug, Clone, Serialize, Deserialize)]
    #[serde(tag = "type")]
    pub enum EventRecord {
        #[serde(rename = "tick")]
        Tick(TickEvent),

        #[serde(rename = "birth")]
        Birth(BirthEvent),

        #[serde(rename = "death")]
        Death(DeathEvent),

        #[serde(rename = "signal")]
        Signal(SignalRecord),

        #[serde(rename = "reproduction")]
        Reproduction(ReproductionEvent),

        #[serde(rename = "shock")]
        Shock(ShockEvent),

        #[serde(rename = "recovery")]
        Recovery(RecoveryRecord),

        #[serde(rename = "snapshot")]
        Snapshot(PopulationSnapshot),

        #[serde(rename = "law")]
        Law(LawEvent),
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct TickEvent {
        pub tick: u64,
        pub population: usize,
        pub energy: u64,
        pub signals_emitted: u64,
        pub signals_delivered: u64,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct BirthEvent {
        pub tick: u64,
        pub cell_id: u32,
        pub cell_type: String,
        pub parent_id: Option<u32>,
        pub generation: u32,
        pub reproduction_cost: u32,
        pub reproduction_threshold: u32,
        pub mutation_rate: f64,
        pub initial_energy: u32,
        pub affinities: Vec<u8>,
        pub mutation_step: u32,
        pub decay_rate: f64,
        pub meta_mutation_rate: f64,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct DeathEvent {
        pub tick: u64,
        pub cell_id: u32,
        pub cell_type: String,
        pub cause: String,
        pub age: u64,
        pub generation: u32,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct SignalRecord {
        pub tick: u64,
        pub signal_id: u64,
        pub signal_type: u8,
        pub source_id: u32,
        pub source_type: String,
        pub target_id: u32,
        pub target_type: String,
        pub delivered: bool,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct ReproductionEvent {
        pub tick: u64,
        pub parent_id: u32,
        pub parent_type: String,
        pub child_id: u32,
        pub child_type: String,
        pub parent_generation: u32,
        pub child_generation: u32,
        pub cost: u32,
        pub threshold: u32,
        pub mut_rate: f64,
        pub mutations: Vec<String>,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct ShockEvent {
        pub tick: u64,
        pub death_rate: f64,
        pub duration: u64,
        pub pre_shock_population: usize,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct RecoveryRecord {
        pub shock_start_tick: u64,
        pub recovery_tick: u64,
        pub duration_ticks: u64,
        pub pre_shock_population: usize,
        pub min_population: usize,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct PopulationSnapshot {
        pub tick: u64,
        pub total_cells: usize,
        #[serde(default)]
        pub by_type: HashMap<String, TypeStats>,
        #[serde(default)]
        pub cost_distribution: HashMap<String, usize>,
        #[serde(default)]
        pub mut_distribution: HashMap<String, usize>,
        #[serde(default)]
        pub thresh_distribution: HashMap<String, usize>,
        #[serde(default)]
        pub energy_distribution: HashMap<String, usize>,
        #[serde(default)]
        pub generation_distribution: HashMap<String, usize>,
        #[serde(default)]
        pub lineage_distribution: HashMap<String, usize>,
        #[serde(default)]
        pub producer_fraction: f64,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct LawEvent {
        pub tick: u64,
        pub law_id: String,
        pub title: String,
    }

    #[derive(Debug, Clone, Serialize, Deserialize, Default)]
    pub struct TypeStats {
        pub count: usize,
        pub total_energy: u64,
        pub avg_age: f64,
        pub births: u64,
        pub deaths: u64,
        pub signals_emitted: u64,
        pub signals_received: u64,
    }

    #[derive(Debug, Clone, Serialize, Deserialize, Default)]
    pub struct CellInfo {
        pub id: u32,
        pub cell_type: String,
        pub generation: u32,
        pub birth_tick: u64,
        pub parent_id: Option<u32>,
        pub reproduction_cost: u32,
        pub reproduction_threshold: u32,
        pub mutation_rate: f64,
        pub initial_energy: u32,
        pub affinities: Vec<u8>,
        pub mutation_step: u32,
        pub decay_rate: f64,
        pub meta_mutation_rate: f64,
        pub energy: u32,
        pub signals_emitted: u64,
        pub signals_received: u64,
        pub alive: bool,
        pub death_tick: Option<u64>,
        pub death_cause: Option<String>,
    }
}