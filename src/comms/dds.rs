// src/comms/dds.rs

use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ReliabilityQoS {
    BestEffort,
    Reliable,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum DurabilityQoS {
    Volatile,
    TransientLocal,
    Transient,
    Persistent,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum HistoryQoS {
    KeepLast(u32),
    KeepAll,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DDSQoSProfile {
    pub reliability: ReliabilityQoS,
    pub durability: DurabilityQoS,
    pub history: HistoryQoS,
    pub deadline_ms: u32,
    pub liveliness_lease_ms: u32,
}

impl Default for DDSQoSProfile {
    fn default() -> Self {
        Self {
            reliability: ReliabilityQoS::Reliable,
            durability: DurabilityQoS::Volatile,
            history: HistoryQoS::KeepLast(10),
            deadline_ms: 100,
            liveliness_lease_ms: 1000,
        }
    }
}

impl DDSQoSProfile {
    pub fn critical_control() -> Self {
        Self {
            reliability: ReliabilityQoS::Reliable,
            durability: DurabilityQoS::TransientLocal,
            history: HistoryQoS::KeepAll,
            deadline_ms: 5,
            liveliness_lease_ms: 100,
        }
    }
    
    pub fn telemetry() -> Self {
        Self {
            reliability: ReliabilityQoS::BestEffort,
            durability: DurabilityQoS::Volatile,
            history: HistoryQoS::KeepLast(5),
            deadline_ms: 1000,
            liveliness_lease_ms: 5000,
        }
    }
}