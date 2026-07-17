use rand::Rng;
use serde::Deserialize;
use std::collections::HashMap;
use std::path::Path;

#[derive(Debug, Clone, Deserialize)]
pub struct SpawnCondition {
    #[serde(default)]
    pub on_signal: Option<String>,
    #[serde(default)]
    pub probability: f64,
    #[serde(default)]
    pub on_startup: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Gene {
    pub cell_type: String,
    #[serde(default)]
    pub version: u32,
    pub asm_path: String,
    #[serde(default = "default_stack_size")]
    pub stack_size: u32,
    #[serde(default = "default_state_size")]
    pub state_size: u32,
    #[serde(default = "default_timeout_us")]
    pub timeout_us: u64,
    pub spawn_condition: SpawnCondition,
    pub max_instances: u32,
    #[serde(default)]
    pub signal_affinity: Vec<u8>,
    #[serde(default = "default_mutation_rate")]
    pub mutation_rate: f64,
    #[serde(default = "default_energy")]
    pub initial_energy: u32,
    #[serde(default)]
    pub reproduction_threshold: u32,
    #[serde(default)]
    pub reproduction_cost: u32,
    #[serde(default)]
    pub metadata: HashMap<String, String>,
    // Heritable evolutionary parameters
    #[serde(default = "default_mutation_step")]
    pub mutation_step: u32,
    #[serde(default = "default_meta_mutation_rate")]
    pub meta_mutation_rate: f64,
    #[serde(default = "default_decay_rate")]
    pub decay_rate: f64,
    // E035: lineage (line of descent) bookkeeping.
    // `lineage` is the immutable ancestor lineage tag; offspring inherit it
    // so we can track reactive <-> memory descent regardless of cell_type swaps.
    #[serde(default)]
    pub lineage: String,
    // E035: a cell_type that offspring may mutate INTO. When set, a child
    // has a chance to switch lineage (cell_type + asm_path + affinity +
    // is_memory) to this sibling. Enables reactive <-> memory crossover.
    #[serde(default)]
    pub sibling: Option<String>,
    // E035: if true, this gene's cells pay `memory_cost` energy per tick as
    // the price of maintaining memory (the "memory is not free" tax).
    #[serde(default)]
    pub is_memory: bool,
    #[serde(default)]
    pub memory_cost: u32,
    // E037: who should receive this lineage's protective type-50 warning when
    // selective (directed) warning is enabled. "broadcast" = everyone
    // (L#062 public good); "kin" = only same-lineage cells (L#063 antidote
    // to the free-rider tragedy).
    #[serde(default = "default_recipient_policy")]
    pub recipient_policy: String,
}

fn default_recipient_policy() -> String { "broadcast".to_string() }

fn default_mutation_step() -> u32 { 20 }
fn default_meta_mutation_rate() -> f64 { 0.001 }
fn default_decay_rate() -> f64 { 0.95 }

fn default_stack_size() -> u32 { 512 }
fn default_state_size() -> u32 { 256 }
fn default_timeout_us() -> u64 { 100 }
fn default_mutation_rate() -> f64 { 0.01 }
fn default_energy() -> u32 { 100 }

#[derive(Debug, Clone, Deserialize)]
pub struct Genome {
    pub name: String,
    #[serde(default)]
    pub version: u32,
    pub genes: Vec<Gene>,
}

impl Genome {
    pub fn from_file(path: impl AsRef<Path>) -> Result<Self, Box<dyn std::error::Error>> {
        let content = std::fs::read_to_string(path.as_ref())?;
        let genome: Genome = serde_json::from_str(&content)?;
        Ok(genome)
    }

    pub fn get_gene(&self, cell_type: &str) -> Option<&Gene> {
        self.genes.iter().find(|g| g.cell_type == cell_type)
    }

    pub fn startup_genes(&self) -> impl Iterator<Item = &Gene> {
        self.genes.iter().filter(|g| g.spawn_condition.on_startup)
    }
}

impl Gene {
    /// E035: try to switch lineage. If the child carries a `sibling` name
    /// and the `other` gene (the sibling) is provided, with probability
    /// `mutation_rate` the child becomes a member of the sibling lineage: it
    /// adopts the sibling's cell_type, asm_path, affinity, is_memory and
    /// memory_cost. The `lineage` tag is preserved so descent can be tracked
    /// through swaps. Returns true if a switch occurred.
    pub fn maybe_switch_lineage(
        child: &mut Gene,
        other: Option<&Gene>,
        rng: &mut impl rand::Rng,
    ) -> bool {
        let sibling_name = match &child.sibling {
            Some(s) => s.clone(),
            None => return false,
        };
        let other = match other {
            Some(g) if g.cell_type == sibling_name => g,
            _ => return false,
        };
        if child.mutation_rate <= 0.0 {
            return false;
        }
        if rng.gen::<f64>() >= child.mutation_rate {
            return false;
        }
        child.cell_type = other.cell_type.clone();
        child.asm_path = other.asm_path.clone();
        child.signal_affinity = other.signal_affinity.clone();
        child.is_memory = other.is_memory;
        child.memory_cost = other.memory_cost;
        child.metadata.insert("switched_from".to_string(), other.cell_type.clone());
        true
    }

    pub fn create_child(&self, parent_id: u32, rng: &mut impl rand::Rng) -> Gene {
        let mut child = self.clone();
        child.version = self.version + 1;
        child.metadata.insert("parent_id".to_string(), parent_id.to_string());
        child.metadata.insert("generation".to_string(), child.version.to_string());

        // Mutation using heritable parameters
        let mut mutation_rate = self.mutation_rate;
        let mutation_step = self.mutation_step;
        let meta_mutation_rate = self.meta_mutation_rate;
        let decay_rate = self.decay_rate;

        if mutation_rate > 0.0 {
            // Meta-mutation: mutation rate can evolve
            if rng.gen::<f64>() < mutation_rate {
                let change = rng.gen_range(-0.5f64..0.5f64) * 0.1;
                mutation_rate = (mutation_rate + change).clamp(0.0, 1.0);
                child.mutation_rate = mutation_rate;
            }

            // Meta-mutation of evolutionary parameters
            if rng.gen::<f64>() < self.meta_mutation_rate {
                let step_change = rng.gen_range(-5i32..=5);
                child.mutation_step = child.mutation_step.saturating_add_signed(step_change).clamp(1, 50);
            }
            if rng.gen::<f64>() < self.meta_mutation_rate {
                let decay_change = rng.gen_range(-0.05f64..0.05f64);
                child.decay_rate = (child.decay_rate + decay_change).clamp(0.5, 0.99);
            }
            if rng.gen::<f64>() < self.meta_mutation_rate {
                let meta_change = rng.gen_range(-0.001f64..0.001f64);
                child.meta_mutation_rate = (child.meta_mutation_rate + meta_change).clamp(0.0, 0.1);
            }

            // Mutate signal affinity: add/remove ONE type (near existing)
            if rng.gen::<f64>() < mutation_rate && !child.signal_affinity.is_empty() {
                if rng.gen::<bool>() {
                    let base_type = child.signal_affinity[rng.gen_range(0..child.signal_affinity.len())];
                    let new_type = rng.gen_range(base_type.saturating_sub(5)..=base_type.saturating_add(5)).clamp(1, 255);
                    if !child.signal_affinity.contains(&new_type) {
                        child.signal_affinity.push(new_type);
                    }
                } else if child.signal_affinity.len() > 1 {
                    let idx = rng.gen_range(0..child.signal_affinity.len());
                    child.signal_affinity.remove(idx);
                }
            }

            // Mutate energy traits using heritable mutation_step
            if rng.gen::<f64>() < mutation_rate {
                let change = rng.gen_range(-(mutation_step as i32)..=(mutation_step as i32));
                child.initial_energy = child.initial_energy.saturating_add_signed(change).max(1);
            }
            if rng.gen::<f64>() < mutation_rate {
                let change = rng.gen_range(-(mutation_step as i32)..=(mutation_step as i32));
                child.reproduction_threshold = child.reproduction_threshold.saturating_add_signed(change).max(1);
            }
            if rng.gen::<f64>() < mutation_rate {
                let change = rng.gen_range(-(mutation_step as i32)..=(mutation_step as i32));
                child.reproduction_cost = child.reproduction_cost.saturating_add_signed(change).max(1);
            }

            // Use heritable decay_rate
            child.mutation_rate = (mutation_rate * decay_rate).max(0.001);
        }

        child
    }
}