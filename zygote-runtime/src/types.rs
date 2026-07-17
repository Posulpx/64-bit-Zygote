use std::fmt;
use serde::{Deserialize, Serialize};

pub const SIGNAL_SIZE: usize = 64;
pub const FIFO_CAPACITY: usize = 4;
pub const FIFO_SIZE: usize = SIGNAL_SIZE * FIFO_CAPACITY;
pub const CONTROL_SIZE: usize = 64;
pub const DEFAULT_STATE_SIZE: usize = 256;
pub const DEFAULT_STACK_SIZE: usize = 512;
pub const DEFAULT_TIMEOUT_US: u64 = 100;
pub const KILL_FLAG_OFFSET: usize = 63;

pub const ENERGY_TICK_COST: u32 = 1;
pub const ENERGY_PROCESS_COST: u32 = 0; // process is now a reward, not a cost
pub const ENERGY_PROCESS_REWARD: u32 = 6; // food: energy gained from consuming a signal
pub const ENERGY_EMIT_COST: u32 = 1;
pub const ENERGY_SPAWN_COST: u32 = 10;

pub const BROADCAST_ID: u32 = 0xFFFFFFFF;

/// Mortality discount applied to a "prepared" cell (one that braced by
/// emitting a predictive protective signal) while a shock is active.
pub const PREDICTIVE_DISCOUNT: f64 = 0.25;
/// How many ticks a cell stays "prepared" after emitting a predictive signal.
pub const PREPARED_DURATION: u8 = 6;

#[repr(C, packed)]
#[derive(Clone, Copy)]
pub struct Signal {
    pub source_id: u32,
    pub target_id: u32,
    pub type_id: u8,
    pub payload: [u8; 55],
}

impl Signal {
    pub fn new(type_id: u8, source_id: u32, target_id: u32) -> Self {
        Self { source_id, target_id, type_id, payload: [0u8; 55] }
    }

    pub fn is_broadcast(&self) -> bool {
        self.target_id == BROADCAST_ID
    }
}

impl fmt::Debug for Signal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let source_id = self.source_id;
        let target_id = self.target_id;
        f.debug_struct("Signal")
            .field("type_id", &self.type_id)
            .field("source_id", &source_id)
            .field("target_id", &target_id)
            .field("payload", &&self.payload[..8])
            .finish()
    }
}

pub struct RingBuffer {
    pub buf: [u8; FIFO_SIZE],
    pub head: u16,
    pub tail: u16,
}

impl RingBuffer {
    pub fn new() -> Self {
        Self { buf: [0u8; FIFO_SIZE], head: 0, tail: 0 }
    }

    fn mask(idx: u16) -> u16 {
        idx & (FIFO_CAPACITY as u16 - 1)
    }

    pub fn is_empty(&self) -> bool {
        self.head == self.tail
    }

    pub fn is_full(&self) -> bool {
        Self::mask(self.tail + 1) == Self::mask(self.head)
    }

    pub fn push(&mut self, signal: &Signal) -> bool {
        if self.is_full() {
            return false;
        }
        let offset = (Self::mask(self.tail) as usize) * SIGNAL_SIZE;
        let bytes = unsafe {
            std::slice::from_raw_parts(signal as *const Signal as *const u8, SIGNAL_SIZE)
        };
        self.buf[offset..offset + SIGNAL_SIZE].copy_from_slice(bytes);
        self.tail = Self::mask(self.tail + 1);
        true
    }

    pub fn pop(&mut self) -> Option<Signal> {
        if self.is_empty() {
            return None;
        }
        let offset = (Self::mask(self.head) as usize) * SIGNAL_SIZE;
        let mut signal = Signal::new(0, 0, 0);
        let bytes = unsafe {
            std::slice::from_raw_parts_mut(&mut signal as *mut Signal as *mut u8, SIGNAL_SIZE)
        };
        bytes.copy_from_slice(&self.buf[offset..offset + SIGNAL_SIZE]);
        self.head = Self::mask(self.head + 1);
        Some(signal)
    }
}

pub struct ControlBlock {
    pub buf: [u8; CONTROL_SIZE],
}

impl ControlBlock {
    pub fn new() -> Self {
        Self { buf: [0u8; CONTROL_SIZE] }
    }

    pub fn input_head(&self) -> u16 {
        u16::from_le_bytes([self.buf[0], self.buf[1]])
    }
    pub fn set_input_head(&mut self, v: u16) {
        self.buf[..2].copy_from_slice(&v.to_le_bytes());
    }
    pub fn input_tail(&self) -> u16 {
        u16::from_le_bytes([self.buf[2], self.buf[3]])
    }
    pub fn set_input_tail(&mut self, v: u16) {
        self.buf[2..4].copy_from_slice(&v.to_le_bytes());
    }
    pub fn output_head(&self) -> u16 {
        u16::from_le_bytes([self.buf[4], self.buf[5]])
    }
    pub fn set_output_head(&mut self, v: u16) {
        self.buf[4..6].copy_from_slice(&v.to_le_bytes());
    }
    pub fn output_tail(&self) -> u16 {
        u16::from_le_bytes([self.buf[6], self.buf[7]])
    }
    pub fn set_output_tail(&mut self, v: u16) {
        self.buf[6..8].copy_from_slice(&v.to_le_bytes());
    }

    pub fn kill_flag(&self) -> bool {
        self.buf[KILL_FLAG_OFFSET] != 0
    }
    pub fn set_kill_flag(&mut self, v: bool) {
        self.buf[KILL_FLAG_OFFSET] = if v { 1 } else { 0 };
    }
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
