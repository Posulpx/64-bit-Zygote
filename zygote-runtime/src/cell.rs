use std::fs;
use std::time::Instant;
use crate::genome::Gene;
use crate::sandbox::ExecCode;
use crate::types::*;

pub struct Cell {
    pub id: u32,
    pub gene: Gene,
    pub exec: ExecCode,
    pub state: Vec<u8>,
    pub input_fifo: RingBuffer,
    pub output_fifo: RingBuffer,
    pub control: ControlBlock,
    pub ticks_left: u64,
    pub energy: u32,
    pub alive: bool,
    pub birth_tick: u64,
    pub death_tick: Option<u64>,
    /// Ticks remaining where this cell is "prepared" (braced against an
    /// imminent shock after emitting a predictive protective signal).
    /// While > 0 the cell gets a mortality discount during elevated death.
    pub prepared_ticks: u8,
}

impl Cell {
    pub fn new(id: u32, gene: &Gene) -> Result<Self, Box<dyn std::error::Error>> {
        let code = fs::read(&gene.asm_path)
            .map_err(|e| format!("Failed to load {}: {}", gene.asm_path, e))?;
        let exec = ExecCode::new(&code)?;
        let state_size = gene.state_size as usize;
        Ok(Self {
            id,
            gene: gene.clone(),
            exec,
            state: vec![0u8; state_size],
            input_fifo: RingBuffer::new(),
            output_fifo: RingBuffer::new(),
            control: ControlBlock::new(),
            ticks_left: 1000,
            energy: gene.initial_energy,
            alive: true,
            birth_tick: 0,
            death_tick: None,
            prepared_ticks: 0,
        })
    }

    pub fn sync_before_tick(&mut self) {
        self.control.set_input_tail(self.input_fifo.tail);
        self.control.set_output_head(self.output_fifo.head);
    }

    pub fn sync_after_tick(&mut self) {
        self.input_fifo.head = self.control.input_head();
        self.output_fifo.tail = self.control.output_tail();
    }

    pub fn tick(&mut self) -> CellTickResult {
        let _start = Instant::now();

        let input_head_before = self.input_fifo.head;
        let output_tail_before = self.output_fifo.tail;

        self.sync_before_tick();

        let cell_fn = self.exec.as_fn_ptr();
        let state_ptr = self.state.as_mut_ptr();
        let input_ptr = self.input_fifo.buf.as_mut_ptr();
        let output_ptr = self.output_fifo.buf.as_mut_ptr();
        let control_ptr = self.control.buf.as_mut_ptr();

        unsafe { cell_fn(state_ptr, input_ptr, output_ptr, control_ptr); }

        self.sync_after_tick();

        // E035: memory maintenance tax. Cells of a memory lineage pay a
        // per-tick cost for retaining state ("memory is not free").
        if self.gene.is_memory && self.gene.memory_cost > 0 {
            self.energy = self.energy.saturating_sub(self.gene.memory_cost);
        }

        self.energy = self.energy.saturating_sub(ENERGY_TICK_COST);
        if self.input_fifo.head != input_head_before {
            self.energy = self.energy.saturating_add(ENERGY_PROCESS_REWARD);
            self.energy = self.energy.saturating_sub(ENERGY_PROCESS_COST);
        }
        if self.output_fifo.tail != output_tail_before {
            self.energy = self.energy.saturating_sub(ENERGY_EMIT_COST);
        }

        if self.energy == 0 {
            self.alive = false;
        }

        self.ticks_left = self.ticks_left.saturating_sub(1);
        if self.control.kill_flag() || self.ticks_left == 0 {
            self.alive = false;
        }
        CellTickResult::Ok
    }
}

pub enum CellTickResult {
    Ok,
    Crashed(String),
    Timeout,
}

pub struct CellPool {
    cells: Vec<Cell>,
    next_id: u32,
}

impl CellPool {
    pub fn new() -> Self {
        Self { cells: Vec::new(), next_id: 0 }
    }

    pub fn spawn(&mut self, gene: &Gene, birth_tick: u64) -> Result<u32, Box<dyn std::error::Error>> {
        let id = self.next_id;
        self.next_id += 1;
        let mut cell = Cell::new(id, gene)?;
        cell.birth_tick = birth_tick;
        self.cells.push(cell);
        Ok(id)
    }

    pub fn kill(&mut self, id: u32) {
        if let Some(cell) = self.cells.iter_mut().find(|c| c.id == id) {
            cell.alive = false;
        }
    }

    pub fn sweep(&mut self) -> usize {
        let before = self.cells.len();
        self.cells.retain(|c| c.alive);
        before - self.cells.len()
    }

    pub fn iter(&self) -> impl Iterator<Item = &Cell> {
        self.cells.iter()
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut Cell> {
        self.cells.iter_mut()
    }

    pub fn get(&self, id: u32) -> Option<&Cell> {
        self.cells.iter().find(|c| c.id == id)
    }

    pub fn get_mut(&mut self, id: u32) -> Option<&mut Cell> {
        self.cells.iter_mut().find(|c| c.id == id)
    }

    pub fn len(&self) -> usize {
        self.cells.len()
    }

    pub fn shuffle(&mut self) {
        use rand::seq::SliceRandom;
        let mut rng = rand::thread_rng();
        self.cells.shuffle(&mut rng);
    }

    pub fn rotate_left(&mut self, n: usize) {
        let len = self.cells.len();
        if len > 1 {
            self.cells.rotate_left(n % len);
        }
    }

    pub fn reverse(&mut self) {
        self.cells.reverse();
    }

    pub fn adopt_orphan_affinity(&mut self, orphan_type: u8) -> usize {
        if self.cells.is_empty() {
            return 0;
        }
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let idx = rng.gen_range(0..self.cells.len());
        self.cells[idx].gene.signal_affinity.push(orphan_type);
        1
    }
}
