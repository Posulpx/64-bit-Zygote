use std::collections::VecDeque;
use crate::types::*;

pub struct SignalBus {
    pending: VecDeque<Signal>,
}

impl SignalBus {
    pub fn new() -> Self {
        Self { pending: VecDeque::new() }
    }

    pub fn emit(&mut self, signal: Signal) {
        self.pending.push_back(signal);
    }

    pub fn pending_count(&self) -> usize {
        self.pending.len()
    }

    pub fn collect_output(&mut self, cell_id: u32, fifo: &mut RingBuffer) -> Vec<Signal> {
        let mut out = Vec::new();
        while let Some(mut sig) = fifo.pop() {
            sig.source_id = cell_id;
            self.emit(sig);
            out.push(sig);
        }
        out
    }

    pub fn drain(&mut self) -> Vec<Signal> {
        self.pending.drain(..).collect()
    }

    pub fn peek_all(&self) -> Vec<Signal> {
        self.pending.iter().copied().collect()
    }
}
