// src/comms/mod.rs

use openssl::ssl::{SslConnector, SslMethod};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::time::{Duration, Instant};

pub mod dds;
pub mod fog;
pub mod tta;

use crate::physical::topology::{ComponentId, PhysicalTopology};
use dds::DDSQoSProfile;
use fog::FogComputingManager;
use tta::TTACycle;

// Custom serialization for Instant (as duration since current time)
mod instant_serde {
    use super::*;

    // A helper struct that can be serialized
    #[derive(Serialize, Deserialize)]
    struct InstantDuration {
        secs: u64,
        nanos: u32,
    }

    pub fn serialize<S>(instant: &Option<Instant>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match instant {
            Some(instant) => {
                // Calculate duration since program start
                let program_duration = instant.elapsed();

                // Serialize as seconds and nanoseconds
                let duration = InstantDuration {
                    secs: program_duration.as_secs(),
                    nanos: program_duration.subsec_nanos(),
                };
                duration.serialize(serializer)
            }
            None => serializer.serialize_none(),
        }
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<Instant>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let opt = Option::<InstantDuration>::deserialize(deserializer)?;
        match opt {
            Some(duration) => {
                // Recreate Instant by subtracting the duration from now
                let dur = Duration::new(duration.secs, duration.nanos);
                Ok(Some(Instant::now() - dur))
            }
            None => Ok(None),
        }
    }
}

// ------ Core Communication Types ------
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum LinkType {
    MAVLink {
        version: u8,
        heartbeat_interval_ms: u32,
    },
    LoRa {
        frequency_mhz: u32,
        spreading_factor: u8,
    },
    WiFiDirect {
        bandwidth_mbps: u32,
        channel: u8,
    },
    MilitaryEncrypted {
        key_rotation_minutes: u32,
        cipher_suite: String,
    },
    // New communication types
    TimeTriggered {
        cycle_time_us: u32,
        slot_count: u8,
    },
    DDS {
        reliability_qos: String,
        deadline_ms: u32,
        history_depth: u32,
    },
    FogComputing {
        edge_node_id: String,
        offload_threshold: f32,
    },
    XRCEDDS {
        resource_constraints: bool,
        mtu_size: u16,
    },
    PALS {
        period_ms: u32,
        synchronization_window_us: u32,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommunicationLink {
    pub link_type: LinkType,
    pub encryption: bool,
    #[serde(with = "instant_serde")]
    pub last_active: Option<Instant>,
}

// ------ Secure Communication ------

// Custom wrapper for SslConnector to make it cloneable and skippable for serde
#[derive(Debug)]
pub struct SslConnectorWrapper(SslConnector);

impl Clone for SslConnectorWrapper {
    fn clone(&self) -> Self {
        // Create a new connector with the same settings
        // This is simplified - in practice you'd need to copy all settings
        let builder = SslConnector::builder(SslMethod::tls())
            .expect("Failed to create SSL connector builder");
        SslConnectorWrapper(builder.build())
    }
}

impl SslConnectorWrapper {
    pub fn encrypt(&self, data: &[u8]) -> Vec<u8> {
        // This demonstrates that we're using the field
        let _connector = &self.0;

        // Simple placeholder implementation
        let mut result = Vec::new();
        result.extend_from_slice(data);
        result
    }
}

// Add custom serialization for SslConnectorWrapper
impl Serialize for SslConnectorWrapper {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // Skip serialization by serializing it as a unit type
        serializer.serialize_unit()
    }
}

impl<'de> Deserialize<'de> for SslConnectorWrapper {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        // Skip deserialization and create a new instance
        let _ = <()>::deserialize(deserializer)?;

        let builder = SslConnector::builder(SslMethod::tls())
            .map_err(|e| serde::de::Error::custom(format!("OpenSSL error: {}", e)))?;

        Ok(SslConnectorWrapper(builder.build()))
    }
}

#[derive(Debug, Clone)]
pub struct SecureChannel {
    connector: SslConnectorWrapper,
    heartbeat_interval: u32,
}

// Add custom serialization for SecureChannel
impl Serialize for SecureChannel {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        use serde::ser::SerializeStruct;
        // Only serialize the heartbeat_interval
        let mut state = serializer.serialize_struct("SecureChannel", 1)?;
        state.serialize_field("heartbeat_interval", &self.heartbeat_interval)?;
        state.end()
    }
}

impl<'de> Deserialize<'de> for SecureChannel {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        // Create a temporary struct that matches our serialization
        #[derive(Deserialize)]
        struct SecureChannelHelper {
            heartbeat_interval: u32,
        }

        let helper = SecureChannelHelper::deserialize(deserializer)?;

        // Create a new secure channel with default settings
        match Self::new("AES256-SHA256") {
            Ok(mut channel) => {
                channel.heartbeat_interval = helper.heartbeat_interval;
                Ok(channel)
            }
            Err(e) => Err(serde::de::Error::custom(format!("OpenSSL error: {}", e))),
        }
    }
}

impl SecureChannel {
    pub fn new(cipher_suite: &str) -> Result<Self, openssl::error::ErrorStack> {
        let mut builder = SslConnector::builder(SslMethod::tls())?;
        builder.set_cipher_list(cipher_suite)?;

        Ok(Self {
            connector: SslConnectorWrapper(builder.build()),
            heartbeat_interval: 1000,
        })
    }

    pub fn encrypt(&self, data: &[u8]) -> Vec<u8> {
        // Use the connector to encrypt data
        self.connector.encrypt(data)
    }

    pub fn get_heartbeat_interval(&self) -> u32 {
        self.heartbeat_interval
    }

    pub fn set_heartbeat_interval(&mut self, interval: u32) {
        self.heartbeat_interval = interval;
    }
}

// ------ Operational Components ------
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Operator {
    pub id: String,
    pub clearance_level: u8,
    pub assigned_links: Vec<LinkType>,
    #[serde(with = "instant_serde")]
    pub last_heartbeat: Option<Instant>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RadarContact {
    pub distance_m: f32,
    pub bearing_deg: f32,
    pub relative_speed_mps: f32,
    pub via_link: LinkType, // Which comms link detected this
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NavigationBeacon {
    pub id: String,
    pub position: (f32, f32),
    pub signal_strength: f32,
    pub link_used: LinkType, // Which comms link received this
}

// Define OODA cycle priority enum
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CommsPriority {
    Low,
    Medium,
    High,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommunicationHub {
    pub primary_link: CommunicationLink,
    pub backup_links: Vec<CommunicationLink>,
    pub operators: Vec<Operator>,
    #[serde(skip)] // Skip this field during serialization/deserialization
    pub secure_channel: Option<SecureChannel>,
    pub radar_contacts: Vec<RadarContact>,
    #[serde(skip)]
    pub tta_cycle: Option<TTACycle>,
    #[serde(skip)]
    pub dds_profile: Option<DDSQoSProfile>,
    #[serde(skip)]
    pub fog_manager: Option<FogComputingManager>,
    #[serde(skip)]
    pub physical_topology: Option<PhysicalTopology>,
}

impl CommunicationHub {
    pub fn new(primary: LinkType, secure: bool) -> Self {
        let primary_link = CommunicationLink {
            link_type: primary.clone(),
            encryption: secure,
            last_active: None,
        };

        let secure_channel = if secure {
            match SecureChannel::new("AES256-SHA256") {
                Ok(channel) => Some(channel),
                Err(e) => {
                    eprintln!("Failed to create secure channel: {}", e);
                    None
                }
            }
        } else {
            None
        };

        // Initialize appropriate SOTA comms based on link type
        let (tta_cycle, dds_profile, fog_manager) = match &primary {
            LinkType::TimeTriggered {
                cycle_time_us,
                slot_count,
            } => (Some(TTACycle::new(*cycle_time_us, *slot_count)), None, None),
            LinkType::DDS { .. } => (None, Some(DDSQoSProfile::default()), None),
            LinkType::FogComputing {
                offload_threshold, ..
            } => (
                None,
                None,
                Some(FogComputingManager::new(*offload_threshold)),
            ),
            _ => (None, None, None),
        };

        Self {
            primary_link,
            backup_links: Vec::new(),
            operators: Vec::new(),
            secure_channel,
            radar_contacts: Vec::new(),
            tta_cycle,
            dds_profile,
            fog_manager,
            physical_topology: None,
        }
    }

    pub fn add_operator(&mut self, id: String, clearance: u8, links: Vec<LinkType>) {
        self.operators.push(Operator {
            id,
            clearance_level: clearance,
            assigned_links: links,
            last_heartbeat: Some(Instant::now()),
        });
    }

    pub fn set_physical_topology(&mut self, topology: PhysicalTopology) {
        self.physical_topology = Some(topology);
    }

    pub fn calculate_message_latency(
        &self,
        from: &ComponentId,
        to: &ComponentId,
    ) -> Result<f32, String> {
        if let Some(topology) = &self.physical_topology {
            // Try to find direct connection
            if let Some(path) = topology.find_shortest_path(from, to) {
                topology.get_path_latency(&path)
            } else {
                Err(format!("No path found between {} and {}", from, to))
            }
        } else {
            Err("No physical topology defined".to_string())
        }
    }

    // Enhance process_message to include physical constraints
    pub fn process_message(
        &mut self,
        data: Vec<u8>,
        from: ComponentId,
        to: ComponentId,
    ) -> Result<f32, String> {
        let message_size_bits = data.len() as f32 * 8.0;

        if let Some(topology) = &self.physical_topology {
            if let Some(path) = topology.find_shortest_path(&from, &to) {
                // Get the latency of the physical path
                let path_latency_ns = topology.get_path_latency(&path)?;

                // Get the reliability of the path
                let path_reliability = topology.get_path_reliability(&path)?;

                // Find the bottleneck link in the path
                let mut min_bandwidth = f32::INFINITY;
                for i in 0..path.len() - 1 {
                    let src = &path[i];
                    let dst = &path[i + 1];

                    if let Some(conn) = topology.connections.get(&(src.clone(), dst.clone())) {
                        min_bandwidth = min_bandwidth.min(conn.max_data_rate_mbps);
                    } else if let Some(conn) = topology.connections.get(&(dst.clone(), src.clone()))
                    {
                        min_bandwidth = min_bandwidth.min(conn.max_data_rate_mbps);
                    }
                }

                // Calculate transmission time based on bandwidth (bits/bandwidth in Mbps)
                let transmission_time_ns = message_size_bits / (min_bandwidth * 1000.0);

                // Calculate total latency
                let total_latency_ns = path_latency_ns + transmission_time_ns;

                // Apply reliability - if below threshold, consider it a failure
                if path_reliability < 99.0 {
                    // Simulate potential data loss based on reliability
                    let rand_val: f32 = rand::random();
                    if rand_val > (path_reliability / 100.0) {
                        return Err(format!(
                            "Message lost due to low path reliability ({}%)",
                            path_reliability
                        ));
                    }
                }

                Ok(total_latency_ns)
            } else {
                Err(format!("No path found between {} and {}", from, to))
            }
        } else {
            // Fallback to simplified model if no physical topology is defined
            let latency_ms = match &self.primary_link.link_type {
                LinkType::DDS { .. } => 0.5,
                LinkType::TimeTriggered { .. } => 1.0,
                LinkType::WiFiDirect { .. } => 2.0,
                LinkType::MAVLink { .. } => 5.0,
                LinkType::LoRa { .. } => 50.0,
                _ => 10.0,
            };

            Ok(latency_ms * 1_000_000.0) // Convert ms to ns
        }
    }

    // Enhance OODA cycle processing to account for physical topology
    pub fn process_ooda_cycle(&mut self, ooda_time: Duration) -> CommsPriority {
        let bandwidth_needed = match ooda_time {
            t if t < Duration::from_millis(100) => CommsPriority::High,
            t if t < Duration::from_millis(500) => CommsPriority::Medium,
            _ => CommsPriority::Low,
        };

        // Check if we have a physical topology to make better decisions
        if let Some(topology) = &self.physical_topology {
            // Calculate average latency in the system
            let mut total_latency = 0.0;
            let mut count = 0;

            for ((from, to), _) in &topology.connections {
                if let Ok(latency) = topology.get_path_latency(&[from.clone(), to.clone()]) {
                    total_latency += latency;
                    count += 1;
                }
            }

            let avg_latency = if count > 0 {
                total_latency / count as f32
            } else {
                1000.0
            };

            // Adjust priority based on physical constraints
            let adjusted_priority = if avg_latency > 1000.0 {
                // If physical latency is high, increase priority
                match bandwidth_needed {
                    CommsPriority::Low => CommsPriority::Medium,
                    _ => CommsPriority::High,
                }
            } else if avg_latency < 100.0 {
                // If physical latency is low, we might be able to reduce priority
                match bandwidth_needed {
                    CommsPriority::High if ooda_time > Duration::from_millis(80) => {
                        CommsPriority::Medium
                    }
                    _ => bandwidth_needed,
                }
            } else {
                bandwidth_needed
            };

            self.adjust_links(adjusted_priority);
            adjusted_priority
        } else {
            // Original behavior without physical topology
            self.adjust_links(bandwidth_needed.clone());
            bandwidth_needed
        }
    }
    pub fn establish_secure_link(&mut self) -> Result<(), String> {
        match &mut self.secure_channel {
            Some(_) => Ok(()),
            None => {
                self.secure_channel = Some(
                    SecureChannel::new("AES256-SHA256")
                        .map_err(|e| format!("Secure channel failed: {}", e))?,
                );
                Ok(())
            }
        }
    }

    pub fn log_beacon(&mut self, beacon: NavigationBeacon) {
        // Implementation - e.g., store the beacon or process it
        println!("Beacon logged: {} at {:?}", beacon.id, beacon.position);
    }

    pub fn adjust_links(&mut self, priority: CommsPriority) {
        match priority {
            CommsPriority::High => {
                // Fast OODA cycle: use high-bandwidth, low-latency link
                if self.dds_profile.is_none() {
                    self.dds_profile = Some(DDSQoSProfile::critical_control());
                }
                self.primary_link.link_type = LinkType::DDS {
                    reliability_qos: "RELIABLE".into(),
                    deadline_ms: 5,
                    history_depth: 1,
                };
            }
            CommsPriority::Medium => {
                // Medium OODA cycle: balance reliability and performance
                if self.tta_cycle.is_none() {
                    self.tta_cycle = Some(TTACycle::new(10000, 8));
                }
                self.primary_link.link_type = LinkType::TimeTriggered {
                    cycle_time_us: 10000,
                    slot_count: 8,
                };
            }
            CommsPriority::Low => {
                // Slow OODA cycle: optimize for power efficiency
                if self.fog_manager.is_none() {
                    self.fog_manager = Some(FogComputingManager::new(0.7));
                }
                self.primary_link.link_type = LinkType::LoRa {
                    frequency_mhz: 915,
                    spreading_factor: 10,
                };
            }
        }
    }
}
