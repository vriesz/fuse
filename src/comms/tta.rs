// src/comms/tta.rs

use std::time::{Duration, Instant};
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeSlot {
    pub id: u8,
    pub duration_us: u32,
    pub allocated_node: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TTACycle {
    pub slots: Vec<TimeSlot>,
    pub cycle_time_us: u32,
    pub current_slot: u8,
    #[serde(skip)]
    pub last_cycle_start: Option<Instant>,
}

impl TTACycle {
    pub fn new(cycle_time_us: u32, slot_count: u8) -> Self {
        let slot_duration = cycle_time_us / (slot_count as u32);
        let mut slots = Vec::with_capacity(slot_count as usize);
        
        for i in 0..slot_count {
            slots.push(TimeSlot {
                id: i,
                duration_us: slot_duration,
                allocated_node: format!("node_{}", i),
            });
        }
        
        Self {
            slots,
            cycle_time_us,
            current_slot: 0,
            last_cycle_start: None,
        }
    }
    
    pub fn start_cycle(&mut self) {
        self.last_cycle_start = Some(Instant::now());
        self.current_slot = 0;
    }
    
    pub fn advance_slot(&mut self) -> bool {
        self.current_slot += 1;
        self.current_slot < self.slots.len() as u8
    }
    
    pub fn get_current_slot(&self) -> Option<&TimeSlot> {
        self.slots.get(self.current_slot as usize)
    }
    
    pub fn time_until_next_slot(&self) -> Duration {
        if let Some(start) = self.last_cycle_start {
            let elapsed = start.elapsed();
            let slot_duration = Duration::from_micros(self.slots[self.current_slot as usize].duration_us as u64);
            if elapsed < slot_duration {
                return slot_duration - elapsed;
            }
        }
        Duration::from_micros(0)
    }
}