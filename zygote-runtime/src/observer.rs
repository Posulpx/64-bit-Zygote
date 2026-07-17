use std::collections::HashMap;

pub struct Observer {
    pub cell_emitted: HashMap<u32, u64>,
    pub cell_received: HashMap<u32, u64>,
    pub sigtype_emitted: HashMap<u8, u64>,
    pub sigtype_delivered: HashMap<u8, u64>,
    pub edges: HashMap<(String, u8), HashMap<String, u64>>,
    pub cell_types: HashMap<u32, String>,
}

impl Observer {
    pub fn new() -> Self {
        Self {
            cell_emitted: HashMap::new(),
            cell_received: HashMap::new(),
            sigtype_emitted: HashMap::new(),
            sigtype_delivered: HashMap::new(),
            edges: HashMap::new(),
            cell_types: HashMap::new(),
        }
    }

    pub fn register_cell(&mut self, id: u32, cell_type: &str) {
        self.cell_types.insert(id, cell_type.to_string());
        self.cell_emitted.entry(id).or_insert(0);
        self.cell_received.entry(id).or_insert(0);
    }

    pub fn cell_killed(&mut self, id: u32) {
        self.cell_types.remove(&id);
        self.cell_emitted.remove(&id);
        self.cell_received.remove(&id);
    }

    pub fn observe_emission(&mut self, cell_id: u32, sig_type: u8) {
        *self.cell_emitted.entry(cell_id).or_insert(0) += 1;
        *self.sigtype_emitted.entry(sig_type).or_insert(0) += 1;
    }

    pub fn observe_delivery(&mut self, cell_id: u32, sig_type: u8,
                            _src_cell_id: u32, src_type: &str) {
        *self.cell_received.entry(cell_id).or_insert(0) += 1;
        *self.sigtype_delivered.entry(sig_type).or_insert(0) += 1;

        let dst_type = self.cell_types.get(&cell_id)
            .map(|s| s.as_str()).unwrap_or("?");
        let edge = self.edges
            .entry((src_type.to_string(), sig_type))
            .or_default();
        *edge.entry(dst_type.to_string()).or_insert(0) += 1;
    }

    pub fn total_cells(&self) -> usize {
        self.cell_emitted.len()
    }

    pub fn most_active_cell(&self) -> Option<(u32, u64)> {
        self.cell_emitted.iter()
            .max_by_key(|(_, &c)| c)
            .map(|(&id, &c)| (id, c))
    }

    pub fn least_active_cell(&self) -> Option<(u32, u64)> {
        self.cell_emitted.iter()
            .filter(|(_, &c)| c > 0)
            .min_by_key(|(_, &c)| c)
            .map(|(&id, &c)| (id, c))
    }

    pub fn orphan_cells(&self) -> Vec<u32> {
        let mut ids: Vec<u32> = self.cell_emitted.keys()
            .copied()
            .filter(|id| {
                let e = self.cell_emitted.get(id).copied().unwrap_or(0);
                let r = self.cell_received.get(id).copied().unwrap_or(0);
                e == 0 && r == 0
            })
            .collect();
        ids.sort();
        ids
    }

    pub fn dead_routes(&self) -> Vec<u8> {
        let mut types: Vec<u8> = self.sigtype_emitted.keys()
            .copied()
            .filter(|t| {
                let d = self.sigtype_delivered.get(t).copied().unwrap_or(0);
                d == 0
            })
            .collect();
        types.sort();
        types
    }

    pub fn signal_graph(&self) -> Vec<String> {
        let mut lines: Vec<String> = Vec::new();
        let mut entries: Vec<_> = self.edges.iter()
            .flat_map(|((src_type, sig_type), dsts)| {
                dsts.iter().map(move |(dst_type, count)| {
                    (src_type.clone(), *sig_type, dst_type.clone(), *count)
                })
            })
            .collect();
        entries.sort_by(|a, b| b.3.cmp(&a.3));

        for (src, sig, dst, count) in &entries {
            lines.push(format!("  {} --[{}]--> {}  (x{})", src, sig, dst, count));
        }
        lines
    }

    pub fn report(&self) -> String {
        let mut out = String::new();

        out.push_str(&format!("\n=== Observer Report ===\n"));

        let total = self.total_cells();
        out.push_str(&format!("Total cells tracked: {}\n", total));

        let emitted_total: u64 = self.cell_emitted.values().sum();
        let received_total: u64 = self.cell_received.values().sum();
        out.push_str(&format!("Total signals emitted: {}\n", emitted_total));
        out.push_str(&format!("Total signals received: {}\n", received_total));

        out.push_str("\n--- Per Signal Type ---\n");
        let mut sigtypes: Vec<_> = self.sigtype_emitted.keys().copied().collect();
        sigtypes.sort();
        for t in &sigtypes {
            let e = self.sigtype_emitted.get(t).copied().unwrap_or(0);
            let d = self.sigtype_delivered.get(t).copied().unwrap_or(0);
            let status = if d == 0 { " DEAD ROUTE" } else { "" };
            out.push_str(&format!("  type {}: {} emitted, {} delivered{}\n", t, e, d, status));
        }

        if let Some((id, count)) = self.most_active_cell() {
            let ty = self.cell_types.get(&id).map(|s| s.as_str()).unwrap_or("?");
            out.push_str(&format!("\nMost active cell: {} ({}) — {} signals emitted\n", id, ty, count));
        }
        if let Some((id, count)) = self.least_active_cell() {
            let ty = self.cell_types.get(&id).map(|s| s.as_str()).unwrap_or("?");
            out.push_str(&format!("Least active cell: {} ({}) — {} signals emitted\n", id, ty, count));
        }

        let orphans = self.orphan_cells();
        if !orphans.is_empty() {
            out.push_str(&format!("\nOrphan cells (no I/O): {}\n", orphans.len()));
            for id in &orphans {
                let ty = self.cell_types.get(id).map(|s| s.as_str()).unwrap_or("?");
                out.push_str(&format!("  {} ({})\n", id, ty));
            }
        } else {
            out.push_str("\nOrphan cells: none\n");
        }

        let dead = self.dead_routes();
        if !dead.is_empty() {
            out.push_str(&format!("\nDead routes (emitted but never delivered):\n"));
            for t in &dead {
                let e = self.sigtype_emitted.get(t).copied().unwrap_or(0);
                out.push_str(&format!("  type {} ({} emitted)\n", t, e));
            }
        }

        out.push_str("\n--- Signal Graph (source --[type]--> dest) ---\n");
        let graph_lines = self.signal_graph();
        if graph_lines.is_empty() {
            out.push_str("  (no signal flow recorded)\n");
        } else {
            for line in &graph_lines {
                out.push_str(line);
                out.push('\n');
            }
        }

        out
    }
}
